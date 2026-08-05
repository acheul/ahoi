/**
 * Reduced feature set — this playground verifies Tsify as the type exporter,
 * not the bridge itself. See the table in
 * `playgrounds/ahoi-wasm-tsify/src/lib.rs`.
 */
import { Show, createSignal } from "solid-js";
import { PierProvider, usePier } from "./bridge";
import type { Fruit } from "../../ahoi-wasm-tsify/pkg/ahoi_wasm_tsify";

function Counter() {
    const pier = usePier();
    const [count, setCount] = pier.hail("Count"); // writable
    const doubled = pier.readHail("Doubled"); // memo
    const [returned, setReturned] = createSignal<number>();

    return (
        <section>
            <h3>Counter</h3>
            <p>
                count: <b id="count">{count()}</b> · doubled (memo):{" "}
                <b id="doubled">{doubled()}</b>
            </p>
            <button id="write-count" onClick={() => setCount(count() + 1)}>
                +1 (write hail)
            </button>
            <button id="tell-increase" onClick={() => setReturned(pier.tell("Increase"))}>
                +1 (tell Increase)
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
    const [echoed, setEchoed] = createSignal<Fruit>();

    const label = (fruit: Fruit | undefined) =>
        fruit === undefined ? "-" : typeof fruit === "string" ? fruit : `Banana(${fruit.Banana})`;

    return (
        <section>
            <h3>Fruits</h3>
            <p>
                last: <b id="last-fruit">{label(lastFruit())}</b> · tell echoed back:{" "}
                <b id="fruit-echoed">{label(echoed())}</b>
            </p>
            <button id="set-apple" onClick={() => setEchoed(pier.tell({ SetFruit: "Apple" }))}>
                Apple
            </button>
            <button
                id="set-banana"
                onClick={() => setEchoed(pier.tell({ SetFruit: { Banana: "yellow" } }))}
            >
                Banana
            </button>
        </section>
    );
}

export function App() {
    return (
        <PierProvider pier="Top">
            <h1>ahoi × solid × tsify</h1>
            <Counter />
            <Items />
            <Fruits />
        </PierProvider>
    );
}
