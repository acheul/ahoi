import { usePier } from "../../setup/solid/ahoi";

export default function Counter() {
    const sphere = usePier();
    const [count, setCount] = sphere.hail("Count"); // writable
    const doubled = sphere.readHail("Doubled"); // read-only memo

    return (
        <div class="demo">
            <p>
                count: <b id="count">{count()}</b> · doubled: <b id="doubled">{doubled()}</b>
            </p>
            <button id="write-count" onClick={() => setCount(count() + 1)}>
                +1 (write)
            </button>
            <button id="tell-increase" onClick={() => sphere.tell("Increase")}>
                +1 (tell)
            </button>
        </div>
    );
}
