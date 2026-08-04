/**
 * Runtime tests for the framework-agnostic core.
 *
 * Runs against `dist/`, so it exercises what actually ships. `tests/inference.ts`
 * covers the type layer; this covers behaviour the type layer cannot see —
 * chiefly that the hail cache is scoped per pier, which decides whether two
 * providers on one page share state by accident.
 */
import assert from "node:assert/strict";
import { AhoiStorage } from "../dist/index.js";

let failures = 0;
function test(name, fn) {
    try {
        fn();
        console.log(`  ok   ${name}`);
    } catch (err) {
        failures++;
        console.log(`  FAIL ${name}\n       ${err.message.split("\n")[0]}`);
    }
}

/** A storage wired to a fake wasm module, plus recorders for what it asked for. */
function harness() {
    let next = 100;
    const enrolled = [];
    const cleared = [];

    const storage = new AhoiStorage(
        {
            _enrol_pier: () => next++,
            _enrol_hail: (par, key) => {
                const id = next++;
                enrolled.push({ id, par, key });
                return [id, 0];
            },
            _clear_sphere: (id) => cleared.push(id),
            _write_hail: () => {},
        },
        (fn) => fn(),
    );

    // Minimal signal: the storage only needs a getter and a setter back.
    const hat = (v) => {
        let cur = v;
        return [() => cur, (n) => (cur = n)];
    };

    const bucket = () => {
        const fns = [];
        return { fns, reg: (fn) => (fns.push(fn), fn), run: () => fns.forEach((f) => f()) };
    };

    return { storage, enrolled, cleared, hat, bucket };
}

/** Enrols every key in one pier and reports how many spheres were created. */
function spheresFor(keys) {
    const { storage, enrolled, hat, bucket } = harness();
    const b = bucket();
    const pier = storage._enrol_pier(undefined, "Top", b.reg);
    for (const key of keys) storage._enrol_read_hail(pier, key, hat, b.reg);
    return enrolled.length;
}

console.log("AhoiStorage: key identity");

test("the same key resolves to one sphere however it is spelled", () => {
    // Property order carries no meaning, so these are the same hail.
    assert.equal(spheresFor([{ Cell: { row: 1, col: 2 } }, { Cell: { col: 2, row: 1 } }]), 1);
});

test("keys that differ in any way get their own sphere", () => {
    const keys = [
        "Item",
        { Item: 3 },
        { Item: 4 },
        { Item: "3" }, // a string payload is not the number payload
        { Cell: { row: 1, col: 2 } },
        { Cell: { row: 2, col: 1 } },
    ];
    assert.equal(spheresFor(keys), keys.length);
});

console.log("AhoiStorage: hail cache");

test("one pier reuses a sphere for a repeated key", () => {
    const { storage, enrolled, hat, bucket } = harness();
    const b = bucket();
    const pier = storage._enrol_pier(undefined, "Top", b.reg);
    storage._enrol_read_hail(pier, "Count", hat, b.reg);
    storage._enrol_read_hail(pier, "Count", hat, b.reg);
    assert.equal(enrolled.filter((e) => e.key === "Count").length, 1);
});

test("two piers get separate spheres for the same key", () => {
    const { storage, enrolled, hat, bucket } = harness();
    const a = bucket();
    const b = bucket();
    const pierA = storage._enrol_pier(undefined, "Top", a.reg);
    const pierB = storage._enrol_pier(undefined, "Top", b.reg);
    storage._enrol_read_hail(pierA, "Count", hat, a.reg);
    storage._enrol_read_hail(pierB, "Count", hat, b.reg);

    const counts = enrolled.filter((e) => e.key === "Count");
    assert.equal(counts.length, 2, "expected one enrolment per pier");
    assert.notEqual(counts[0].id, counts[1].id);
    assert.notEqual(counts[0].par, counts[1].par);
});

test("object keys are separated per pier too", () => {
    const { storage, enrolled, hat, bucket } = harness();
    const a = bucket();
    const b = bucket();
    const pierA = storage._enrol_pier(undefined, "Top", a.reg);
    const pierB = storage._enrol_pier(undefined, "Top", b.reg);
    storage._enrol_read_hail(pierA, { Item: 0 }, hat, a.reg);
    storage._enrol_read_hail(pierB, { Item: 0 }, hat, b.reg);
    assert.equal(enrolled.filter((e) => typeof e.key === "object").length, 2);
});

test("adapters get a distinct accessor per pier", () => {
    // The Svelte adapter hangs its store off the accessor identity, so two
    // piers sharing one accessor would show pier A's value inside pier B.
    const { storage, hat, bucket } = harness();
    const a = bucket();
    const b = bucket();
    const pierA = storage._enrol_pier(undefined, "Top", a.reg);
    const pierB = storage._enrol_pier(undefined, "Top", b.reg);
    const readA = storage._enrol_read_hail(pierA, "Count", hat, a.reg);
    const readB = storage._enrol_read_hail(pierB, "Count", hat, b.reg);
    assert.notEqual(readA, readB);
});

console.log("AhoiStorage: teardown");

test("releasing one pier leaves the other's hails alone", () => {
    const { storage, enrolled, cleared, hat, bucket } = harness();
    const a = bucket();
    const b = bucket();
    const pierA = storage._enrol_pier(undefined, "Top", a.reg);
    const pierB = storage._enrol_pier(undefined, "Top", b.reg);
    storage._enrol_read_hail(pierA, "Count", hat, a.reg);
    storage._enrol_read_hail(pierB, "Count", hat, b.reg);

    a.run();

    const bSpheres = enrolled.filter((e) => e.par === pierB).map((e) => e.id);
    assert.ok(bSpheres.length > 0);
    for (const id of bSpheres) assert.ok(!cleared.includes(id), `sphere ${id} was cleared`);
});

test("a shared key is only cleared once its last holder releases", () => {
    const { storage, enrolled, cleared, hat, bucket } = harness();
    const b = bucket();
    const pier = storage._enrol_pier(undefined, "Top", b.reg);
    storage._enrol_read_hail(pier, "Count", hat, b.reg);
    storage._enrol_read_hail(pier, "Count", hat, b.reg);

    const hailId = enrolled.find((e) => e.key === "Count").id;
    b.fns[1](); // first holder releases
    assert.ok(!cleared.includes(hailId), "cleared while still referenced");
    b.fns[2](); // second holder releases
    assert.ok(cleared.includes(hailId), "not cleared after last release");
});

test("tearing a pier down leaves no empty key map behind", () => {
    const { storage, hat, bucket } = harness();
    const a = bucket();
    const pier = storage._enrol_pier(undefined, "Top", a.reg);
    storage._enrol_read_hail(pier, "Count", hat, a.reg);

    a.run();

    const maps = storage["_hail_keys"];
    assert.ok(!maps.has(pier), "per-pier map outlived its pier");
});

console.log();
if (failures > 0) {
    console.log(`${failures} test(s) failed`);
    process.exit(1);
}
console.log("all tests passed");
