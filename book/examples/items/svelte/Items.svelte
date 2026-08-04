<script lang="ts">
    import { useHail, useReadHail, useTell } from "../../setup/svelte/ahoi";

    const items = useReadHail("Items"); // Readable<number[]>
    const first = useHail({ Item: 0 }); // path-derived, writable
    const tell = useTell();
</script>

<div class="demo">
    <p>
        items: <b id="items">{$items.join(", ") || "(empty)"}</b>
    </p>
    <p>
        item 0: <b id="item0">{$first ?? "undefined"}</b>
    </p>
    <button id="push" on:click={() => tell({ PushItem: $items.length * 10 })}>push</button>
    <button id="pop" on:click={() => tell("PopItem")}>pop</button>
    <button id="bump" on:click={() => ($first = ($first ?? 0) + 1)}>+1 on item 0</button>
</div>
