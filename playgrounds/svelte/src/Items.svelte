<script lang="ts">
    import { useHail, useReadHail, useTell } from "./ahoi";

    const items = useReadHail("Items");
    const item1 = useHail({ Item: 1 }); // writable path-derived
    const tell = useTell();
    let popped: number | undefined = $state();
</script>

<section>
    <h3>Items</h3>
    <p>
        items: <b id="items">[{$items.join(", ")}]</b> · items[1]:
        <b id="item-1">{$item1 ?? "-"}</b>
    </p>
    <button id="push-item" onclick={() => tell({ PushItem: ($items.length + 1) * 10 })}>
        push
    </button>
    <button id="pop-item" onclick={() => (popped = tell("PopItem"))}>pop</button>
    <button id="write-item" onclick={() => ($item1 = ($item1 ?? 0) + 1)}>
        items[1] += 1 (write derived)
    </button>
    <span> last popped: <b id="popped">{popped ?? "-"}</b></span>
</section>
