<script setup lang="ts">
/**
 * Each section exercises one bridge feature — see the table in
 * `playgrounds/ahoi-wasm/src/lib.rs`. Mirrors the solid/react playgrounds
 * (same element ids).
 */
import { ref } from "vue";
import { PierProvider } from "./ahoi";
import Counter from "./Counter.vue";
import Items from "./Items.vue";
import Fruits from "./Fruits.vue";
import Panel from "./Panel.vue";
import PanicDemo from "./PanicDemo.vue";

const showPanel = ref(false);
</script>

<template>
    <PierProvider pier="Top">
        <h1>ahoi × vue</h1>
        <Counter />
        <Items />
        <Fruits />
        <section>
            <button id="toggle-panel" @click="showPanel = !showPanel">
                {{ showPanel ? "unmount panel" : "mount panel" }}
            </button>
            <PierProvider v-if="showPanel" pier="Panel">
                <Panel />
            </PierProvider>
        </section>
        <PanicDemo />
    </PierProvider>
</template>
