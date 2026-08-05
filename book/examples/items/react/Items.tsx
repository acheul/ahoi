import { useHail, useReadHail, useTell } from "../../setup/react/bridge";

export default function Items() {
    const items = useReadHail("Items"); // number[]
    const [first, setFirst] = useHail({ Item: 0 }); // path-derived, writable
    const tell = useTell();

    return (
        <div className="demo">
            <p>
                items: <b id="items">{items.join(", ") || "(empty)"}</b>
            </p>
            <p>
                item 0: <b id="item0">{first ?? "undefined"}</b>
            </p>
            <button id="push" onClick={() => tell({ PushItem: items.length * 10 })}>
                push
            </button>
            <button id="pop" onClick={() => tell("PopItem")}>
                pop
            </button>
            <button id="bump" onClick={() => setFirst((first ?? 0) + 1)}>
                +1 on item 0
            </button>
        </div>
    );
}
