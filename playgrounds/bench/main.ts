/**
 * Reactivity bench harness.
 *
 * Drives `AhoiStorage` directly with a trivial signal implementation, so the
 * numbers isolate the ahoi core + wasm bridge (no UI framework in the loop).
 * All bridge operations are synchronous, so `performance.now()` around an
 * operation captures the full JS→Rust→propagation→dispatch→JS round-trip.
 *
 * Method: per sample, run K ops and divide (dodges the ~100µs timer
 * resolution); warmup samples are discarded; stats over S samples.
 * Every scenario also asserts how many hail dispatches it expects per op —
 * a "fast" regression that silently stops propagating shows up as ✗.
 */
import wasmInit, { abi_version, clear, hail, pier, set_panic_hook, tell, write } from "./pkg/bench_wasm";
import { AhoiStorage, type SphereId } from "@acheul/ahoi-js";

await wasmInit();
set_panic_hook();

// ── storage with a counting signal impl ─────────────────────────────────────

let dispatches = 0;

const hat = <T>(v: T): [() => T, (nv: T) => void] => {
    let cur = v;
    return [() => cur, (nv: T) => { cur = nv; dispatches++; }];
};

/** collects cleanups so scenarios can release their hails */
const makeScope = () => {
    const fns: (() => any)[] = [];
    const on_clean_up = <F extends () => any>(fn: F): F => { fns.push(fn); return fn; };
    return { on_clean_up, release: () => { fns.forEach((f) => f()); fns.length = 0; } };
};

const storage = new AhoiStorage<any, any>(
    {
        _enrol_pier: pier,
        _enrol_hail: (p, k) => hail(p, k) as [SphereId, any],
        _clear_sphere: clear,
        _write_hail: write,
        _abi_version: abi_version,
    },
    (fn) => fn(),
);

const pierScope = makeScope(); // never released
const pierId = storage._enrol_pier(undefined, "Bench", pierScope.on_clean_up);

// ── measurement ─────────────────────────────────────────────────────────────

const WARMUP = 5;
const SAMPLES = 30;

interface Scenario {
    name: string;
    /** ops per sample */
    k: number;
    /** expected hail dispatches per op */
    expect: number;
    setup?: () => void;
    op: (i: number) => void;
    teardown?: () => void;
}

interface Result {
    name: string;
    median: number;
    p95: number;
    mean: number;
    dispatchOk: boolean;
}

function runScenario(s: Scenario): Result {
    s.setup?.();
    const times: number[] = [];
    let ok = true;
    for (let sample = 0; sample < WARMUP + SAMPLES; sample++) {
        dispatches = 0;
        const t0 = performance.now();
        for (let i = 0; i < s.k; i++) s.op(sample * s.k + i);
        const t1 = performance.now();
        if (dispatches !== s.expect * s.k) ok = false;
        if (sample >= WARMUP) times.push(((t1 - t0) / s.k) * 1000); // µs/op
    }
    s.teardown?.();
    times.sort((a, b) => a - b);
    const median = times[Math.floor(times.length / 2)];
    const p95 = times[Math.floor(times.length * 0.95)];
    const mean = times.reduce((a, b) => a + b, 0) / times.length;
    return { name: s.name, median, p95, mean, dispatchOk: ok };
}

// ── scenarios ───────────────────────────────────────────────────────────────

const FANOUT = 200;
const CHAIN_DEPTH = 100;
const ENROL_N = 200;

function scenarios(): Scenario[] {
    let scope = makeScope();
    let writer: (v: number) => void;
    const fanoutScope = makeScope();
    const chainScope = makeScope();

    return [
        {
            name: "tell noop (boundary only)",
            k: 500,
            expect: 0,
            op: () => tell(pierId, "Noop"),
        },
        {
            name: "write hail round-trip (1 cell)",
            k: 500,
            expect: 1,
            setup: () => {
                scope = makeScope();
                const [, w] = storage._enrol_hail<number>(pierId, { Cell: 0 }, hat, scope.on_clean_up);
                writer = w;
            },
            op: (i) => writer(i + 1),
            teardown: () => scope.release(),
        },
        {
            name: "tell round-trip (bump 1 cell)",
            k: 500,
            expect: 1,
            setup: () => {
                scope = makeScope();
                storage._enrol_hail<number>(pierId, { Cell: 1 }, hat, scope.on_clean_up);
            },
            op: () => tell(pierId, { Bump: 1 }),
            teardown: () => scope.release(),
        },
        {
            name: `fan-out (1 write → ${FANOUT} hails)`,
            k: 20,
            expect: FANOUT,
            setup: () => {
                for (let i = 0; i < FANOUT; i++) {
                    storage._enrol_hail<number>(pierId, { Cell: i }, hat, fanoutScope.on_clean_up);
                }
            },
            op: () => tell(pierId, "WriteAll"),
            teardown: () => fanoutScope.release(),
        },
        {
            name: `memo chain (depth ${CHAIN_DEPTH})`,
            k: 100,
            expect: 1,
            setup: () => {
                storage._enrol_read_hail<number>(pierId, { Chain: CHAIN_DEPTH }, hat, chainScope.on_clean_up);
            },
            op: (i) => tell(pierId, { SetSrc: i + 1 }),
            teardown: () => chainScope.release(),
        },
        {
            name: `enrol + clear (${ENROL_N} hails)`,
            k: 3,
            expect: 0,
            op: () => {
                const s = makeScope();
                for (let i = 0; i < ENROL_N; i++) {
                    storage._enrol_read_hail<number>(pierId, { Cell: i }, hat, s.on_clean_up);
                }
                s.release();
            },
        },
    ];
}

// ── UI ──────────────────────────────────────────────────────────────────────

const BASELINE_KEY = "ahoi-bench-baseline";
const $ = (id: string) => document.getElementById(id)!;
const tbody = document.querySelector<HTMLTableSectionElement>("#results tbody")!;

const fmt = (us: number) => (us >= 100 ? us.toFixed(0) : us.toFixed(2));

function render(results: Result[]) {
    const baseline: Record<string, number> = JSON.parse(localStorage.getItem(BASELINE_KEY) ?? "{}");
    tbody.innerHTML = "";
    for (const r of results) {
        const base = baseline[r.name];
        const delta =
            base === undefined ? "—" : `${(((r.median - base) / base) * 100).toFixed(1)}% (base ${fmt(base)})`;
        const row = tbody.insertRow();
        [r.name, fmt(r.median), fmt(r.p95), fmt(r.mean), delta, r.dispatchOk ? "✓" : "✗"].forEach(
            (text) => (row.insertCell().textContent = String(text)),
        );
    }
    $("json").textContent = JSON.stringify(
        Object.fromEntries(results.map((r) => [r.name, Number(r.median.toFixed(3))])),
        null,
        2,
    );
}

let lastResults: Result[] = [];

$("run").onclick = () => {
    $("status").textContent = " running…";
    // let the status paint before the sync bench loop blocks the thread
    setTimeout(() => {
        lastResults = scenarios().map(runScenario);
        render(lastResults);
        $("status").textContent = " done";
    }, 20);
};

$("save-baseline").onclick = () => {
    if (lastResults.length === 0) return;
    localStorage.setItem(
        BASELINE_KEY,
        JSON.stringify(Object.fromEntries(lastResults.map((r) => [r.name, r.median]))),
    );
    render(lastResults);
    $("status").textContent = " baseline saved";
};

$("clear-baseline").onclick = () => {
    localStorage.removeItem(BASELINE_KEY);
    if (lastResults.length > 0) render(lastResults);
    $("status").textContent = " baseline cleared";
};
