import { usePier } from "../../setup/solid/ahoi";

export default function ResourceDemo() {
    const sphere = usePier();
    const count = sphere.readHail("Count");
    const tenTimes = sphere.readHail("TenTimes"); // number | undefined
    const loading = sphere.readHail("TenTimesLoading");
    const running = sphere.readHail("TickerRunning"); // Action state

    return (
        <div class="demo">
            <p>
                count: <b id="count">{count()}</b> · ×10 (async):{" "}
                <b id="ten-times">{tenTimes() ?? "—"}</b>{" "}
                <span id="loading">{loading() ? "(fetching…)" : ""}</span>
            </p>
            <button id="bump-1" onClick={() => sphere.tell({ Bump: 1 })}>
                +1
            </button>
            <button id="start" onClick={() => sphere.tell({ StartTicker: 1 })}>
                start ticker (+1/s)
            </button>
            <button id="stop" onClick={() => sphere.tell("StopTicker")}>
                stop
            </button>
            <span id="running">{running() ? " ticking…" : ""}</span>
        </div>
    );
}
