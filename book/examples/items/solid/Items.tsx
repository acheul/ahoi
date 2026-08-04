import { usePier } from "../../setup/solid/ahoi";

export default function Items() {
    const sphere = usePier();
    const items = sphere.readHail("Items"); // () => number[]
    const [first, setFirst] = sphere.hail({ Item: 0 }); // path-derived, writable

    return (
        <div class="demo">
            <p>
                items: <b id="items">{items().join(", ") || "(empty)"}</b>
            </p>
            <p>
                item 0: <b id="item0">{first() ?? "undefined"}</b>
            </p>
            <button id="push" onClick={() => sphere.tell({ PushItem: items().length * 10 })}>
                push
            </button>
            <button id="pop" onClick={() => sphere.tell("PopItem")}>
                pop
            </button>
            <button id="bump" onClick={() => setFirst((first() ?? 0) + 1)}>
                +1 on item 0
            </button>
        </div>
    );
}
