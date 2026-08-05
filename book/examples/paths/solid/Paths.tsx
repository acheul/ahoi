import { usePier } from "../../setup/solid/bridge";

export default function PathsDemo() {
    const pier = usePier();
    const [first, setFirst] = pier.hail({ Item: 0 }); // writable, path-derived
    const [second, setSecond] = pier.hail({ Item: 1 });
    const watch0 = pier.readHail("Watch0Runs"); // effect watching items[0]
    const watch1 = pier.readHail("Watch1Runs"); // effect watching items[1]

    return (
        <div class="demo">
            <p>
                items[0]: <b id="item0">{first()}</b> · its watcher ran{" "}
                <b id="w0">{watch0()}</b> times
            </p>
            <p>
                items[1]: <b id="item1">{second()}</b> · its watcher ran{" "}
                <b id="w1">{watch1()}</b> times
            </p>
            <button id="bump0" onClick={() => setFirst((first() ?? 0) + 1)}>
                +1 on items[0]
            </button>
            <button id="bump1" onClick={() => setSecond((second() ?? 0) + 1)}>
                +1 on items[1]
            </button>
        </div>
    );
}
