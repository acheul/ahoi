<script setup lang="ts">
import { useReadHail, useTell } from "../../setup/vue/ahoi";

const count = useReadHail("Count");
const tenTimes = useReadHail("TenTimes"); // number | undefined
const loading = useReadHail("TenTimesLoading");
const tell = useTell();
</script>

<template>
  <div class="demo">
    <p>
      count: <b id="count">{{ count }}</b> · ×10 (async):
      <b id="ten-times">{{ tenTimes ?? "—" }}</b>
      <span id="loading">{{ loading ? "(fetching…)" : "" }}</span>
    </p>
    <button id="bump-1" @click="tell({ Bump: 1 })">+1</button>
  </div>
</template>
