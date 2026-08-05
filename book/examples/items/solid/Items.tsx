import { usePier } from "../../setup/solid/bridge";

export default function Items() {
    const pier = usePier();
    const items = pier.readHail("Items"); // () => number[]
    const [first, setFirst] = pier.hail({ Item: 0 }); // path-derived, writable

    return (
        <div class="demo">
            <p>
                items: <b id="items">{items().join(", ") || "(empty)"}</b>
            </p>
            <p>
                item 0: <b id="item0">{first() ?? "undefined"}</b>
            </p>
            <button id="push" onClick={() => pier.tell({ PushItem: items().length * 10 })}>
                push
            </button>
            <button id="pop" onClick={() => pier.tell("PopItem")}>
                pop
            </button>
            <button id="bump" onClick={() => setFirst((first() ?? 0) + 1)}>
                +1 on item 0
            </button>
        </div>
    );
}
