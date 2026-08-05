import { useHail, useReadHail, useTell } from "../../setup/react/bridge";

export default function Counter() {
    const [count, setCount] = useHail("Count"); // writable
    const doubled = useReadHail("Doubled"); // read-only memo
    const tell = useTell();

    return (
        <div className="demo">
            <p>
                count: <b id="count">{count}</b> · doubled: <b id="doubled">{doubled}</b>
            </p>
            <button id="write-count" onClick={() => setCount(count + 1)}>
                +1 (write)
            </button>
            <button id="tell-increase" onClick={() => tell("Increase")}>
                +1 (tell)
            </button>
        </div>
    );
}
