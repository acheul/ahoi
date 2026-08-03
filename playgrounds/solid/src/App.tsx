/**
 * Each section exercises one bridge feature — see the table in
 * `playgrounds/ahoi-wasm/src/lib.rs`.
 */
import { For, Show, createSignal } from "solid-js";
import { PierProvider, usePier } from "./ahoi";
import type { Fruit } from "../../ahoi-wasm/bindings/Fruit";

function Counter() {
    const sphere = usePier();
    const [count, setCount] = sphere.hail("Count"); // writable
    const doubled = sphere.readHail("Doubled"); // memo
    const countX10 = sphere.readHail("CountX10"); // async resource
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
            <button id="tell-increase" onClick={() => setReturned(sphere.tell("Increase"))}>
                +1 (tell Increase)
            </button>
            <button id="add-count" onClick={() => sphere.tell({ AddCount: 5 })}>
                +5 (callback)
            </button>
            <button id="start-ticker" onClick={() => sphere.tell({ StartTicker: 1 })}>
                start ticker (+1/s)
            </button>
            <button id="stop-ticker" onClick={() => sphere.tell("StopTicker")}>
                stop ticker
            </button>
            <Show when={returned() !== undefined}>
                <span> → tell returned: <b id="tell-returned">{returned()}</b></span>
            </Show>
        </section>
    );
}

function Items() {
    const sphere = usePier();
    const items = sphere.readHail("Items");
    const [item1, setItem1] = sphere.hail({ Item: 1 }); // writable path-derived
    const [popped, setPopped] = createSignal<number>();

    return (
        <section>
            <h3>Items</h3>
            <p>
                items: <b id="items">[{items().join(", ")}]</b> · items[1]:{" "}
                <b id="item-1">{item1() ?? "-"}</b>
            </p>
            <button id="push-item" onClick={() => sphere.tell({ PushItem: (items().length + 1) * 10 })}>
                push
            </button>
            <button id="pop-item" onClick={() => setPopped(sphere.tell("PopItem"))}>
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
    const sphere = usePier();
    const lastFruit = sphere.readHail("LastFruit"); // enum on the wire
    const fruitCounts = sphere.readHail("FruitCounts"); // JS Map on the wire

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
            <button id="set-apple" onClick={() => sphere.tell({ SetFruit: "Apple" })}>
                Apple
            </button>
            <button id="set-banana" onClick={() => sphere.tell({ SetFruit: { Banana: "yellow" } })}>
                Banana
            </button>
        </section>
    );
}

/** Lives under its own nested pier; unmounting must clear its spheres. */
function Panel() {
    const sphere = usePier();
    const [info, setInfo] = sphere.hail("PanelInfo");

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
        </PierProvider>
    );
}
