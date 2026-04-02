<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";

const router = useRouter();
const captureTitles = ref(false);
const excludedInput = ref("");
const excludedApps = ref<string[]>([]);

const suggestedExclusions = [
  "1Password",
  "Keychain Access",
  "LastPass",
  "Bitwarden",
  "Messages",
  "WhatsApp",
  "Signal",
  "Telegram",
];

function addExclusion(name: string) {
  if (name && !excludedApps.value.includes(name)) {
    excludedApps.value.push(name);
  }
  excludedInput.value = "";
}

function removeExclusion(name: string) {
  excludedApps.value = excludedApps.value.filter((a) => a !== name);
}

async function finish() {
  await invoke("set_capture_titles", { enabled: captureTitles.value });
  for (const app of excludedApps.value) {
    await invoke("exclude_app", { appName: app });
  }
  await invoke("complete_onboarding");
  router.replace("/logs");
}
</script>

<template>
  <div class="onboarding">
    <div class="content">
      <h1>Welcome to Friction Log</h1>
      <p class="subtitle">
        Before you start, here's what this app does and doesn't do.
      </p>

      <section class="card">
        <h2>What we collect</h2>
        <ul class="check-list">
          <li>Your manually typed friction notes</li>
          <li>The name of the app you're using when you log</li>
          <li>Timestamps</li>
        </ul>
      </section>

      <section class="card">
        <h2>What we never do</h2>
        <ul class="cross-list">
          <li>Take screenshots</li>
          <li>Log keystrokes</li>
          <li>Read clipboard contents</li>
          <li>Scrape app or web page content</li>
          <li>Send any data off your machine</li>
        </ul>
      </section>

      <section class="card">
        <h2>Window titles</h2>
        <p class="description">
          Optionally, we can capture window titles for richer context (e.g.
          "Jira — PROJ-123"). This is off by default.
        </p>
        <label class="toggle-row">
          <input type="checkbox" v-model="captureTitles" />
          <span>Capture window titles</span>
        </label>
      </section>

      <section class="card">
        <h2>Exclude sensitive apps</h2>
        <p class="description">
          These apps will never be recorded, even by name.
        </p>
        <div class="suggestions">
          <button
            v-for="app in suggestedExclusions"
            :key="app"
            :class="{ active: excludedApps.includes(app) }"
            @click="
              excludedApps.includes(app)
                ? removeExclusion(app)
                : addExclusion(app)
            "
          >
            {{ app }}
          </button>
        </div>
        <div class="add-row">
          <input
            v-model="excludedInput"
            placeholder="Add another app name..."
            @keydown.enter.prevent="addExclusion(excludedInput.trim())"
          />
          <button
            class="add-btn"
            @click="addExclusion(excludedInput.trim())"
            :disabled="!excludedInput.trim()"
          >
            Add
          </button>
        </div>
      </section>

      <button class="primary-btn" @click="finish">Start using Friction Log</button>
    </div>
  </div>
</template>

<style scoped>
.onboarding {
  max-width: 560px;
  margin: 0 auto;
  padding: 32px 24px 48px;
}

.content h1 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 4px;
}

.subtitle {
  opacity: 0.5;
  margin-bottom: 24px;
}

.card {
  background: rgba(128, 128, 128, 0.06);
  border-radius: 10px;
  padding: 16px 20px;
  margin-bottom: 16px;
}

.card h2 {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 8px;
}

.description {
  font-size: 14px;
  opacity: 0.6;
  margin-bottom: 10px;
}

.check-list,
.cross-list {
  list-style: none;
  font-size: 14px;
}

.check-list li::before {
  content: "✓ ";
  color: #4caf50;
  font-weight: 600;
}

.cross-list li::before {
  content: "✗ ";
  color: #e57373;
  font-weight: 600;
}

.check-list li,
.cross-list li {
  padding: 2px 0;
}

.toggle-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  cursor: pointer;
}

.toggle-row input[type="checkbox"] {
  width: 16px;
  height: 16px;
}

.suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 10px;
}

.suggestions button {
  font-size: 13px;
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.suggestions button.active {
  background: rgba(229, 115, 115, 0.15);
  border-color: rgba(229, 115, 115, 0.4);
}

.add-row {
  display: flex;
  gap: 6px;
}

.add-row input {
  flex: 1;
  font-size: 13px;
  padding: 6px 10px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  outline: none;
}

.add-btn {
  font-size: 13px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.add-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.primary-btn {
  width: 100%;
  margin-top: 8px;
  padding: 12px;
  font-size: 15px;
  font-weight: 500;
  border: none;
  border-radius: 10px;
  background: #4a90d9;
  color: white;
  cursor: pointer;
}

.primary-btn:hover {
  background: #3a7bc8;
}
</style>
