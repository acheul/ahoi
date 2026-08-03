<script setup lang="ts">
import { ref } from "vue";
import { useHail, useReadHail, useTell } from "./ahoi";

const count = useHail("Count"); // writable
const doubled = useReadHail("Doubled"); // memo
const countX10 = useReadHail("CountX10"); // async resource
const tell = useTell();
const returned = ref<number>();
</script>

<template>
    <section>
        <h3>Counter</h3>
        <p>
            count: <b id="count">{{ count }}</b> · doubled (memo):
            <b id="doubled">{{ doubled }}</b> · ×10 (async resource):
            <b id="count-x10">{{ countX10 ?? "loading…" }}</b>
        </p>
        <button id="write-count" @click="count++">+1 (write hail)</button>
        <button id="tell-increase" @click="returned = tell('Increase')">
            +1 (tell Increase)
        </button>
        <button id="add-count" @click="tell({ AddCount: 5 })">+5 (callback)</button>
        <button id="start-ticker" @click="tell({ StartTicker: 1 })">
            start ticker (+1/s)
        </button>
        <button id="stop-ticker" @click="tell('StopTicker')">stop ticker</button>
        <span v-if="returned !== undefined">
            → tell returned: <b id="tell-returned">{{ returned }}</b>
        </span>
    </section>
</template>
