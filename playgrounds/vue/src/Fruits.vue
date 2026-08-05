<script setup lang="ts">
import { useReadHail, useTell } from "./bridge";
import type { Fruit } from "../../ahoi-wasm/bindings/Fruit";

const lastFruit = useReadHail("LastFruit"); // enum on the wire
const fruitCounts = useReadHail("FruitCounts"); // JS Map on the wire
const tell = useTell();

const label = (fruit: Fruit | undefined) =>
  fruit === undefined
    ? "-"
    : typeof fruit === "string"
      ? fruit
      : `Banana(${fruit.Banana})`;
</script>

<template>
  <section>
    <h3>Fruits</h3>
    <p>
      last: <b id="last-fruit">{{ label(lastFruit) }}</b> · counts:
      <b id="fruit-counts">
        <span v-for="[name, n] in fruitCounts" :key="name">{{
          `${name}×${n} `
        }}</span>
      </b>
    </p>
    <button id="set-apple" @click="tell({ SetFruit: 'Apple' })">Apple</button>
    <button id="set-banana" @click="tell({ SetFruit: { Banana: 'yellow' } })">
      Banana
    </button>
  </section>
</template>
