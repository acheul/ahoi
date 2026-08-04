<script lang="ts">
    import { useReadHail, useTell } from "../../setup/svelte/ahoi";

    const count = useReadHail("Count");
    const tenTimes = useReadHail("TenTimes"); // number | undefined
    const loading = useReadHail("TenTimesLoading");
    const tell = useTell();
</script>

<div class="demo">
    <p>
        count: <b id="count">{$count}</b> · ×10 (async):
        <b id="ten-times">{$tenTimes ?? "—"}</b>
        <span id="loading">{$loading ? "(fetching…)" : ""}</span>
    </p>
    <button id="bump-1" on:click={() => tell({ Bump: 1 })}>+1</button>
</div>
