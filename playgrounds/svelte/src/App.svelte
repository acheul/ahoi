<script lang="ts">
    /**
     * Each section exercises one bridge feature — see the table in
     * `playgrounds/ahoi-wasm/src/lib.rs`. Mirrors the other playgrounds
     * (same element ids).
     *
     * Svelte has no provider component here: `providePier` sets the pier for
     * this component and everything below it.
     */
    import { providePier } from "./ahoi";
    import Counter from "./Counter.svelte";
    import Items from "./Items.svelte";
    import Fruits from "./Fruits.svelte";
    import Panel from "./Panel.svelte";
    import PanicDemo from "./PanicDemo.svelte";

    providePier("Top");
    let showPanel = $state(false);
</script>

<h1>ahoi × svelte</h1>
<Counter />
<Items />
<Fruits />
<section>
    <button id="toggle-panel" onclick={() => (showPanel = !showPanel)}>
        {showPanel ? "unmount panel" : "mount panel"}
    </button>
    {#if showPanel}
        <Panel />
    {/if}
</section>
<PanicDemo />
