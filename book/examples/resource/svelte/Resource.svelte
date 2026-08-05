<script lang="ts">
    import { useReadHail, useTell } from "../../setup/svelte/bridge";

    const count = useReadHail("Count");
    const tenTimes = useReadHail("TenTimes"); // number | undefined
    const loading = useReadHail("TenTimesLoading");
    const running = useReadHail("TickerRunning"); // Action state
    const tell = useTell();
</script>

<div class="demo">
    <p>
        count: <b id="count">{$count}</b> · ×10 (async):
        <b id="ten-times">{$tenTimes ?? "—"}</b>
        <span id="loading">{$loading ? "(fetching…)" : ""}</span>
    </p>
    <button id="bump-1" on:click={() => tell({ Bump: 1 })}>+1</button>
    <button id="start" on:click={() => tell({ StartTicker: 1 })}>start ticker (+1/s)</button>
    <button id="stop" on:click={() => tell("StopTicker")}>stop</button>
    <span id="running">{$running ? " ticking…" : ""}</span>
</div>
