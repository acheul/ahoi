import { useReadHail, useTell } from "../../setup/react/ahoi";

export default function MemoDemo() {
    const count = useReadHail("Count");
    const parity = useReadHail("Parity"); // count % 2
    const label = useReadHail("Label"); // memo over parity
    const runs = useReadHail("LabelRuns"); // times the label memo ran
    const tell = useTell();

    return (
        <div className="demo">
            <p>
                count: <b id="count">{count}</b> · parity: <b id="parity">{parity}</b> · label:{" "}
                <b id="label">{label}</b>
            </p>
            <p>
                label memo has run <b id="runs">{runs}</b> times
            </p>
            <button id="bump-1" onClick={() => tell({ Bump: 1 })}>
                +1
            </button>
            <button id="bump-2" onClick={() => tell({ Bump: 2 })}>
                +2
            </button>
        </div>
    );
}
