import { useHail, useReadHail } from "../../setup/react/bridge";

export default function PathsDemo() {
    const [first, setFirst] = useHail({ Item: 0 }); // writable, path-derived
    const [second, setSecond] = useHail({ Item: 1 });
    const watch0 = useReadHail("Watch0Runs"); // effect watching items[0]
    const watch1 = useReadHail("Watch1Runs"); // effect watching items[1]

    return (
        <div className="demo">
            <p>
                items[0]: <b id="item0">{first}</b> · its watcher ran <b id="w0">{watch0}</b>{" "}
                times
            </p>
            <p>
                items[1]: <b id="item1">{second}</b> · its watcher ran <b id="w1">{watch1}</b>{" "}
                times
            </p>
            <button id="bump0" onClick={() => setFirst((first ?? 0) + 1)}>
                +1 on items[0]
            </button>
            <button id="bump1" onClick={() => setSecond((second ?? 0) + 1)}>
                +1 on items[1]
            </button>
        </div>
    );
}
