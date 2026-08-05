<script lang="ts">
    import { useHail, useReadHail, useTell } from "../../setup/svelte/bridge";

    const count = useHail("Count"); // writable store
    const doubled = useReadHail("Doubled"); // read-only memo
    const tell = useTell();
</script>

<div class="demo">
    <p>
        count: <b id="count">{$count}</b> · doubled: <b id="doubled">{$doubled}</b>
    </p>
    <button id="write-count" on:click={() => ($count += 1)}>+1 (write)</button>
    <button id="tell-increase" on:click={() => tell("Increase")}>+1 (tell)</button>
</div>
