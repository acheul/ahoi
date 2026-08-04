<script setup lang="ts">
import { useHail, useReadHail, useTell } from "../../setup/vue/ahoi";

const count = useHail("Count"); // writable ref
const doubled = useReadHail("Doubled"); // read-only memo
const tell = useTell();
</script>

<template>
  <div class="demo">
    <p>
      count: <b id="count">{{ count }}</b> · doubled: <b id="doubled">{{ doubled }}</b>
    </p>
    <button id="write-count" @click="count++">+1 (write)</button>
    <button id="tell-increase" @click="tell('Increase')">+1 (tell)</button>
  </div>
</template>
