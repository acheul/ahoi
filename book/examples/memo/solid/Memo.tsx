import { usePier } from "../../setup/solid/ahoi";

export default function MemoDemo() {
    const sphere = usePier();
    const count = sphere.readHail("Count");
    const parity = sphere.readHail("Parity"); // count % 2
    const label = sphere.readHail("Label"); // memo over parity
    const runs = sphere.readHail("LabelRuns"); // times the label memo ran

    return (
        <div class="demo">
            <p>
                count: <b id="count">{count()}</b> · parity: <b id="parity">{parity()}</b> ·
                label: <b id="label">{label()}</b>
            </p>
            <p>
                label memo has run <b id="runs">{runs()}</b> times
            </p>
            <button id="bump-1" onClick={() => sphere.tell({ Bump: 1 })}>
                +1
            </button>
            <button id="bump-2" onClick={() => sphere.tell({ Bump: 2 })}>
                +2
            </button>
        </div>
    );
}
