<script setup lang="ts">
import { useHail, useReadHail, useTell } from "../../setup/vue/ahoi";

const items = useReadHail("Items"); // ComputedRef<number[]>
const first = useHail({ Item: 0 }); // path-derived, writable
const tell = useTell();
</script>

<template>
  <div class="demo">
    <p>
      items: <b id="items">{{ items.join(", ") || "(empty)" }}</b>
    </p>
    <p>
      item 0: <b id="item0">{{ first ?? "undefined" }}</b>
    </p>
    <button id="push" @click="tell({ PushItem: items.length * 10 })">push</button>
    <button id="pop" @click="tell('PopItem')">pop</button>
    <button id="bump" @click="first = (first ?? 0) + 1">+1 on item 0</button>
  </div>
</template>
