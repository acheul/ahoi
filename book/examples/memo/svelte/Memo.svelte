<script lang="ts">
    import { useReadHail, useTell } from "../../setup/svelte/bridge";

    const count = useReadHail("Count");
    const parity = useReadHail("Parity"); // count % 2
    const label = useReadHail("Label"); // memo over parity
    const runs = useReadHail("LabelRuns"); // times the label memo ran
    const tell = useTell();
</script>

<div class="demo">
    <p>
        count: <b id="count">{$count}</b> · parity: <b id="parity">{$parity}</b> ·
        label: <b id="label">{$label}</b>
    </p>
    <p>
        label memo has run <b id="runs">{$runs}</b> times
    </p>
    <button id="bump-1" on:click={() => tell({ Bump: 1 })}>+1</button>
    <button id="bump-2" on:click={() => tell({ Bump: 2 })}>+2</button>
</div>
