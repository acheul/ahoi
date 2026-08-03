<script setup lang="ts">
// Lives under its own nested pier; unmounting must clear its spheres.
import { useHail } from "./ahoi";

const info = useHail("PanelInfo");
</script>

<template>
    <section>
        <h3>Panel (nested pier)</h3>
        <p>info: <b id="panel-info">{{ info }}</b></p>
        <input id="panel-input" v-model="info" />
    </section>
</template>
