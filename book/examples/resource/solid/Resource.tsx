import { usePier } from "../../setup/solid/bridge";

export default function ResourceDemo() {
    const pier = usePier();
    const count = pier.readHail("Count");
    const tenTimes = pier.readHail("TenTimes"); // number | undefined
    const loading = pier.readHail("TenTimesLoading");
    const running = pier.readHail("TickerRunning"); // Action state

    return (
        <div class="demo">
            <p>
                count: <b id="count">{count()}</b> · ×10 (async):{" "}
                <b id="ten-times">{tenTimes() ?? "—"}</b>{" "}
                <span id="loading">{loading() ? "(fetching…)" : ""}</span>
            </p>
            <button id="bump-1" onClick={() => pier.tell({ Bump: 1 })}>
                +1
            </button>
            <button id="start" onClick={() => pier.tell({ StartTicker: 1 })}>
                start ticker (+1/s)
            </button>
            <button id="stop" onClick={() => pier.tell("StopTicker")}>
                stop
            </button>
            <span id="running">{running() ? " ticking…" : ""}</span>
        </div>
    );
}
