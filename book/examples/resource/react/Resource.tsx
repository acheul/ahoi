import { useReadHail, useTell } from "../../setup/react/ahoi";

export default function ResourceDemo() {
    const count = useReadHail("Count");
    const tenTimes = useReadHail("TenTimes"); // number | undefined
    const loading = useReadHail("TenTimesLoading");
    const tell = useTell();

    return (
        <div className="demo">
            <p>
                count: <b id="count">{count}</b> · ×10 (async):{" "}
                <b id="ten-times">{tenTimes ?? "—"}</b>{" "}
                <span id="loading">{loading ? "(fetching…)" : ""}</span>
            </p>
            <button id="bump-1" onClick={() => tell({ Bump: 1 })}>
                +1
            </button>
        </div>
    );
}
