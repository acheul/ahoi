<script setup lang="ts">
/**
 * Panic diagnostics. `Tell::PanicDemo` holds a write guard and then reads the
 * same value — the classic double-borrow bug.
 *
 * What to look for: the Rust panic in the console should point at the `read()`
 * line in `playgrounds/ahoi-wasm-tsrs/src/lib.rs`, not at a line inside ahoi-core.
 * Those locations only exist in a `--dev` wasm build; release builds compile
 * them out.
 */
import { ref } from "vue";
import { useTell } from "./bridge";

const tell = useTell();
const fired = ref(false);

const trigger = () => {
  fired.value = true;
  tell("PanicDemo");
};
</script>

<template>
  <section>
    <h3>Panic diagnostics</h3>
    <p>
      Triggers a double-borrow panic on purpose. Open the console: the blame
      should land on the <code>read()</code> call in
      <code>ahoi-wasm-tsrs/src/lib.rs</code>.
    </p>
    <p>
      <b>Note:</b> wasm panics abort the module — everything above stops working
      afterwards. Reload the page to continue.
    </p>
    <button id="panic-demo" @click="trigger">
      trigger panic (double borrow)
    </button>
    <span v-if="fired" id="panic-fired">
      → check the console; reload to recover</span
    >
  </section>
</template>
