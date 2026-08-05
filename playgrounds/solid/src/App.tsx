/**
 * Each section exercises one bridge feature — see the table in
 * `playgrounds/ahoi-wasm/src/lib.rs`.
 */
import { For, Show, createSignal } from "solid-js";
import { PierProvider, usePier } from "./bridge";
import type { Fruit } from "../../ahoi-wasm/bindings/Fruit";

function Counter() {
    const pier = usePier();
    const [count, setCount] = pier.hail("Count"); // writable
    const doubled = pier.readHail("Doubled"); // memo
    const countX10 = pier.readHail("CountX10"); // async resource
    const [returned, setReturned] = createSignal<number>();

    return (
        <section>
            <h3>Counter</h3>
            <p>
                count: <b id="count">{count()}</b> · doubled (memo):{" "}
                <b id="doubled">{doubled()}</b> · ×10 (async resource):{" "}
                <b id="count-x10">{countX10() ?? "loading…"}</b>
            </p>
            <button id="write-count" onClick={() => setCount(count() + 1)}>
                +1 (write hail)
            </button>
            <button id="tell-increase" onClick={() => setReturned(pier.tell("Increase"))}>
                +1 (tell Increase)
            </button>
            <button id="add-count" onClick={() => pier.tell({ AddCount: 5 })}>
                +5 (callback)
            </button>
            <button id="start-ticker" onClick={() => pier.tell({ StartTicker: 1 })}>
                start ticker (+1/s)
            </button>
            <button id="stop-ticker" onClick={() => pier.tell("StopTicker")}>
                stop ticker
            </button>
            <Show when={returned() !== undefined}>
                <span> → tell returned: <b id="tell-returned">{returned()}</b></span>
            </Show>
        </section>
    );
}

function Items() {
    const pier = usePier();
    const items = pier.readHail("Items");
    const [item1, setItem1] = pier.hail({ Item: 1 }); // writable path-derived
    const [popped, setPopped] = createSignal<number>();

    return (
        <section>
            <h3>Items</h3>
            <p>
                items: <b id="items">[{items().join(", ")}]</b> · items[1]:{" "}
                <b id="item-1">{item1() ?? "-"}</b>
            </p>
            <button id="push-item" onClick={() => pier.tell({ PushItem: (items().length + 1) * 10 })}>
                push
            </button>
            <button id="pop-item" onClick={() => setPopped(pier.tell("PopItem"))}>
                pop
            </button>
            <button id="write-item" onClick={() => setItem1((item1() ?? 0) + 1)}>
                items[1] += 1 (write derived)
            </button>
            <span> last popped: <b id="popped">{popped() ?? "-"}</b></span>
        </section>
    );
}

function Fruits() {
    const pier = usePier();
    const lastFruit = pier.readHail("LastFruit"); // enum on the wire
    const fruitCounts = pier.readHail("FruitCounts"); // JS Map on the wire

    const label = (fruit: Fruit | undefined) =>
        fruit === undefined ? "-" : typeof fruit === "string" ? fruit : `Banana(${fruit.Banana})`;

    return (
        <section>
            <h3>Fruits</h3>
            <p>
                last: <b id="last-fruit">{label(lastFruit())}</b> · counts:{" "}
                <b id="fruit-counts">
                    <For each={[...fruitCounts().entries()]}>
                        {([name, n]) => <span>{`${name}×${n} `}</span>}
                    </For>
                </b>
            </p>
            <button id="set-apple" onClick={() => pier.tell({ SetFruit: "Apple" })}>
                Apple
            </button>
            <button id="set-banana" onClick={() => pier.tell({ SetFruit: { Banana: "yellow" } })}>
                Banana
            </button>
        </section>
    );
}

/**
 * Panic diagnostics. `Tell::PanicDemo` holds a write guard and then reads the
 * same value — the classic double-borrow bug.
 *
 * What to look for: the Rust panic in the console should point at the `read()`
 * line in `playgrounds/ahoi-wasm/src/lib.rs`, not at a line inside ahoi-core.
 * Those locations only exist in a `--dev` wasm build; release builds compile
 * them out.
 */
function PanicDemo() {
    const pier = usePier();
    const [fired, setFired] = createSignal(false);

    return (
        <section>
            <h3>Panic diagnostics</h3>
            <p>
                Triggers a double-borrow panic on purpose. Open the console: the
                blame should land on the <code>read()</code> call in{" "}
                <code>ahoi-wasm/src/lib.rs</code>.
            </p>
            <p>
                <b>Note:</b> wasm panics abort the module — everything above stops
                working afterwards. Reload the page to continue.
            </p>
            <button
                id="panic-demo"
                onClick={() => {
                    setFired(true);
                    pier.tell("PanicDemo");
                }}
            >
                trigger panic (double borrow)
            </button>
            <Show when={fired()}>
                <span id="panic-fired"> → check the console; reload to recover</span>
            </Show>
        </section>
    );
}

/** Lives under its own nested pier; unmounting must clear its spheres. */
function Panel() {
    const pier = usePier();
    const [info, setInfo] = pier.hail("PanelInfo");

    return (
        <section>
            <h3>Panel (nested pier)</h3>
            <p>info: <b id="panel-info">{info()}</b></p>
            <input
                id="panel-input"
                value={info()}
                onInput={(e) => setInfo(e.currentTarget.value)}
            />
        </section>
    );
}

export function App() {
    const [showPanel, setShowPanel] = createSignal(false);

    return (
        <PierProvider pier="Top">
            <h1>ahoi × solid</h1>
            <Counter />
            <Items />
            <Fruits />
            <section>
                <button id="toggle-panel" onClick={() => setShowPanel(!showPanel())}>
                    {showPanel() ? "unmount panel" : "mount panel"}
                </button>
                <Show when={showPanel()}>
                    <PierProvider pier="Panel">
                        <Panel />
                    </PierProvider>
                </Show>
            </section>
            <PanicDemo />
        </PierProvider>
    );
}
