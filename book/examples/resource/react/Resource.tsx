import { useReadHail, useTell } from "../../setup/react/bridge";

export default function ResourceDemo() {
    const count = useReadHail("Count");
    const tenTimes = useReadHail("TenTimes"); // number | undefined
    const loading = useReadHail("TenTimesLoading");
    const running = useReadHail("TickerRunning"); // Action state
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
            <button id="start" onClick={() => tell({ StartTicker: 1 })}>
                start ticker (+1/s)
            </button>
            <button id="stop" onClick={() => tell("StopTicker")}>
                stop
            </button>
            <span id="running">{running ? " ticking…" : ""}</span>
        </div>
    );
}
