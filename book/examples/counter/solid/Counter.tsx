import { usePier } from "../../setup/solid/bridge";

export default function Counter() {
    const pier = usePier();
    const [count, setCount] = pier.hail("Count"); // writable
    const doubled = pier.readHail("Doubled"); // read-only memo

    return (
        <div class="demo">
            <p>
                count: <b id="count">{count()}</b> · doubled: <b id="doubled">{doubled()}</b>
            </p>
            <button id="write-count" onClick={() => setCount(count() + 1)}>
                +1 (write)
            </button>
            <button id="tell-increase" onClick={() => pier.tell("Increase")}>
                +1 (tell)
            </button>
        </div>
    );
}
