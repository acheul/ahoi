import { usePier } from "../../setup/solid/bridge";

export default function MemoDemo() {
    const pier = usePier();
    const count = pier.readHail("Count");
    const parity = pier.readHail("Parity"); // count % 2
    const label = pier.readHail("Label"); // memo over parity
    const runs = pier.readHail("LabelRuns"); // times the label memo ran

    return (
        <div class="demo">
            <p>
                count: <b id="count">{count()}</b> · parity: <b id="parity">{parity()}</b> ·
                label: <b id="label">{label()}</b>
            </p>
            <p>
                label memo has run <b id="runs">{runs()}</b> times
            </p>
            <button id="bump-1" onClick={() => pier.tell({ Bump: 1 })}>
                +1
            </button>
            <button id="bump-2" onClick={() => pier.tell({ Bump: 2 })}>
                +2
            </button>
        </div>
    );
}
