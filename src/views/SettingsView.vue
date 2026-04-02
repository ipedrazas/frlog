<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";

const router = useRouter();

const trackingPaused = ref(false);
const captureTitles = ref(false);
const excludedApps = ref<string[]>([]);
const excludeInput = ref("");
const focusEventCount = ref(0);
const confirmingDelete = ref(false);

async function loadSettings() {
  const settings = await invoke<{
    tracking_paused: boolean;
    capture_titles: boolean;
    onboarding_completed: boolean;
    poll_interval_secs: number;
  }>("get_settings");
  trackingPaused.value = settings.tracking_paused;
  captureTitles.value = settings.capture_titles;
}

async function loadExclusions() {
  excludedApps.value = await invoke("get_excluded_apps");
}

async function loadStats() {
  focusEventCount.value = await invoke("get_focus_event_count");
}

async function togglePause() {
  trackingPaused.value = !trackingPaused.value;
  await invoke("set_tracking_paused", { paused: trackingPaused.value });
}

async function toggleTitles() {
  captureTitles.value = !captureTitles.value;
  await invoke("set_capture_titles", { enabled: captureTitles.value });
}

async function addExclusion() {
  const name = excludeInput.value.trim();
  if (!name || excludedApps.value.includes(name)) return;
  await invoke("exclude_app", { appName: name });
  excludeInput.value = "";
  await loadExclusions();
}

async function removeExclusion(name: string) {
  await invoke("remove_app_exclusion", { appName: name });
  await loadExclusions();
}

async function deleteAllData() {
  if (!confirmingDelete.value) {
    confirmingDelete.value = true;
    return;
  }
  await invoke("delete_all_data");
  confirmingDelete.value = false;
  await loadStats();
}

onMounted(async () => {
  await Promise.all([loadSettings(), loadExclusions(), loadStats()]);
});
</script>

<template>
  <div class="settings-container">
    <header class="settings-header">
      <button class="back-btn" @click="router.push('/logs')">&larr; Back</button>
      <h1>Settings</h1>
    </header>

    <section class="card">
      <h2>Tracking</h2>
      <label class="toggle-row" @click.prevent="togglePause">
        <span class="toggle-label">
          {{ trackingPaused ? "Tracking paused" : "Tracking active" }}
        </span>
        <span :class="['toggle-switch', { active: !trackingPaused }]">
          <span class="toggle-knob" />
        </span>
      </label>
      <p class="stat">{{ focusEventCount }} focus events recorded</p>
    </section>

    <section class="card">
      <h2>Window titles</h2>
      <p class="description">
        When enabled, window titles are recorded alongside app names for richer
        context. Off by default for privacy.
      </p>
      <label class="toggle-row" @click.prevent="toggleTitles">
        <span class="toggle-label">Capture window titles</span>
        <span :class="['toggle-switch', { active: captureTitles }]">
          <span class="toggle-knob" />
        </span>
      </label>
    </section>

    <section class="card">
      <h2>Excluded apps</h2>
      <p class="description">
        These apps are never recorded, not even by name.
      </p>
      <div v-if="excludedApps.length" class="exclusion-list">
        <div v-for="app in excludedApps" :key="app" class="exclusion-item">
          <span>{{ app }}</span>
          <button class="remove-btn" @click="removeExclusion(app)">Remove</button>
        </div>
      </div>
      <div v-else class="empty-exclusions">No apps excluded</div>
      <div class="add-row">
        <input
          v-model="excludeInput"
          placeholder="App name to exclude..."
          @keydown.enter.prevent="addExclusion"
        />
        <button
          class="add-btn"
          @click="addExclusion"
          :disabled="!excludeInput.trim()"
        >
          Add
        </button>
      </div>
    </section>

    <section class="card danger">
      <h2>Data</h2>
      <p class="description">
        Delete all captured data including friction logs and focus events. This
        cannot be undone.
      </p>
      <button
        :class="['delete-btn', { confirming: confirmingDelete }]"
        @click="deleteAllData"
      >
        {{ confirmingDelete ? "Click again to confirm deletion" : "Delete all data" }}
      </button>
    </section>
  </div>
</template>

<style scoped>
.settings-container {
  max-width: 560px;
  margin: 0 auto;
  padding: 32px 24px;
}

.settings-header {
  margin-bottom: 24px;
}

.back-btn {
  font-size: 13px;
  background: none;
  border: none;
  color: #4a90d9;
  cursor: pointer;
  padding: 0;
  margin-bottom: 8px;
}

.settings-header h1 {
  font-size: 24px;
  font-weight: 600;
  text-align: left;
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
  font-size: 13px;
  opacity: 0.5;
  margin-bottom: 12px;
}

.stat {
  font-size: 12px;
  opacity: 0.4;
  margin-top: 8px;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  padding: 4px 0;
}

.toggle-label {
  font-size: 14px;
}

.toggle-switch {
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: rgba(128, 128, 128, 0.3);
  position: relative;
  transition: background 0.2s;
}

.toggle-switch.active {
  background: #4a90d9;
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 9px;
  background: white;
  transition: transform 0.2s;
}

.toggle-switch.active .toggle-knob {
  transform: translateX(18px);
}

.exclusion-list {
  margin-bottom: 10px;
}

.exclusion-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  font-size: 14px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.1);
}

.exclusion-item:last-child {
  border-bottom: none;
}

.remove-btn {
  font-size: 12px;
  color: #e57373;
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 6px;
}

.empty-exclusions {
  font-size: 13px;
  opacity: 0.4;
  margin-bottom: 10px;
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

.card.danger {
  border: 1px solid rgba(229, 115, 115, 0.2);
}

.delete-btn {
  width: 100%;
  padding: 8px;
  font-size: 13px;
  border: 1px solid rgba(229, 115, 115, 0.3);
  border-radius: 6px;
  background: transparent;
  color: #e57373;
  cursor: pointer;
}

.delete-btn.confirming {
  background: rgba(229, 115, 115, 0.1);
  border-color: #e57373;
}
</style>
