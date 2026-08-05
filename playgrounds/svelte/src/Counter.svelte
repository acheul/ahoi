<script lang="ts">
    import { useHail, useReadHail, useTell } from "./bridge";

    const count = useHail("Count"); // writable
    const doubled = useReadHail("Doubled"); // memo
    const countX10 = useReadHail("CountX10"); // async resource
    const tell = useTell();
    let returned: number | undefined = $state();
</script>

<section>
    <h3>Counter</h3>
    <p>
        count: <b id="count">{$count}</b> · doubled (memo):
        <b id="doubled">{$doubled}</b> · ×10 (async resource):
        <b id="count-x10">{$countX10 ?? "loading…"}</b>
    </p>
    <button id="write-count" onclick={() => ($count += 1)}>+1 (write hail)</button>
    <button id="tell-increase" onclick={() => (returned = tell("Increase"))}>
        +1 (tell Increase)
    </button>
    <button id="add-count" onclick={() => tell({ AddCount: 5 })}>+5 (callback)</button>
    <button id="start-ticker" onclick={() => tell({ StartTicker: 1 })}>
        start ticker (+1/s)
    </button>
    <button id="stop-ticker" onclick={() => tell("StopTicker")}>stop ticker</button>
    {#if returned !== undefined}
        <span> → tell returned: <b id="tell-returned">{returned}</b></span>
    {/if}
</section>
