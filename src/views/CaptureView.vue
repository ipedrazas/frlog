<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";

const note = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

onMounted(async () => {
  await nextTick();
  inputRef.value?.focus();
});

function submit() {
  const text = note.value.trim();
  if (!text) return;
  // Fire and forget — Rust hides the window before writing to DB
  invoke("save_log", { rawText: text });
  note.value = "";
}

function cancel() {
  note.value = "";
  invoke("close_capture");
}
</script>

<template>
  <div class="capture-overlay" @mousedown.self="cancel">
    <input
      ref="inputRef"
      v-model="note"
      class="capture-input"
      placeholder="What's frustrating you right now?"
      @keydown.enter.prevent="submit"
      @keydown.escape="cancel"
      autofocus
      spellcheck="false"
    />
  </div>
</template>

<style scoped>
.capture-overlay {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  background: transparent;
}

.capture-input {
  width: 100%;
  height: 100%;
  border: none;
  outline: none;
  font-size: 18px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  padding: 0 16px;
  border-radius: 12px;
  background: rgba(30, 30, 30, 0.95);
  color: #f0f0f0;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.capture-input::placeholder {
  color: rgba(255, 255, 255, 0.4);
}
</style>
