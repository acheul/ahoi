<script lang="ts">
    // Owns its own nested pier; destroying this component clears its spheres.
    import { providePier, useHail } from "./bridge";

    providePier("Panel");
    const info = useHail("PanelInfo");
</script>

<section>
    <h3>Panel (nested pier)</h3>
    <p>info: <b id="panel-info">{$info}</b></p>
    <input id="panel-input" bind:value={$info} />
</section>
