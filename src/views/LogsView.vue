<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useRouter } from "vue-router";

const router = useRouter();

interface LogEntry {
  id: number;
  raw_text: string;
  note_text: string;
  parsed_command: string | null;
  category: string | null;
  app_context: string | null;
  source: string;
  created_at: string;
}

interface WaitingPeriod {
  id: number;
  note: string;
  direction: string;
  start_time: string;
  end_time: string | null;
  duration_secs: number | null;
}

interface FrictionCluster {
  id: number;
  title: string;
  summary: string;
  category: string;
  confidence: number;
  status: string;
  log_count: number;
}

const logs = ref<LogEntry[]>([]);
const activeWaiting = ref<WaitingPeriod[]>([]);
const clusters = ref<FrictionCluster[]>([]);
const activeFilter = ref<string>("all");
const activeTab = ref<"logs" | "clusters">("logs");
const clusterLoading = ref(false);
const editingLogId = ref<number | null>(null);
const editLogText = ref("");
let unlisten: UnlistenFn | null = null;

const filters = [
  { value: "all", label: "All" },
  { value: "cmd:waiting", label: "Waiting" },
  { value: "cmd:blocking", label: "Blocking" },
  { value: "cmd:repetitive", label: "Repetitive" },
  { value: "cmd:coordination", label: "Coordination" },
  { value: "cmd:guilt_pile", label: "Guilt Pile" },
];

async function fetchLogs() {
  const filter = activeFilter.value === "all" ? null : activeFilter.value;
  logs.value = await invoke("get_logs", { filter });
}

async function fetchActiveWaiting() {
  activeWaiting.value = await invoke("get_active_waiting_periods");
}

async function fetchClusters() {
  clusters.value = await invoke("get_clusters");
}

async function recomputeClusters() {
  clusterLoading.value = true;
  clusters.value = await invoke("recompute_clusters");
  clusterLoading.value = false;
}

async function endWaiting(direction: string) {
  await invoke("end_waiting", { direction });
  await Promise.all([fetchLogs(), fetchActiveWaiting()]);
}

function startEditLog(log: LogEntry) {
  editingLogId.value = log.id;
  editLogText.value = log.raw_text;
}

async function saveEditLog() {
  if (editingLogId.value === null) return;
  await invoke("update_log_text", {
    logId: editingLogId.value,
    rawText: editLogText.value,
  });
  editingLogId.value = null;
  await fetchLogs();
}

function cancelEditLog() {
  editingLogId.value = null;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.round(secs / 60)}m`;
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function elapsedSince(iso: string): string {
  const secs = Math.floor((Date.now() - new Date(iso).getTime()) / 1000);
  return formatDuration(secs);
}

const commandLabels: Record<string, string> = {
  waiting: "Waiting",
  blocking: "Blocking",
  repetitive: "Repetitive",
  coordination: "Coordination",
  guilt_pile: "Guilt Pile",
};

const categoryColors: Record<string, string> = {
  bottleneck: "#e57373",
  repetitive_work: "#ffb74d",
  coordination_tax: "#ba68c8",
  context_switching: "#4dd0e1",
  guilt_pile: "#7986cb",
};

const categoryLabels: Record<string, string> = {
  repetitive_work: "Repetitive Work",
  bottleneck: "Bottleneck",
  coordination_tax: "Coordination Tax",
  context_switching: "Context Switching",
  guilt_pile: "Guilt Pile",
};

watch(activeFilter, () => fetchLogs());

onMounted(async () => {
  await Promise.all([fetchLogs(), fetchActiveWaiting(), fetchClusters()]);
  unlisten = await listen("log-saved", () => {
    fetchLogs();
    fetchActiveWaiting();
  });
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="logs-container">
    <header class="logs-header">
      <div class="header-top">
        <h1>Friction Logs</h1>
        <div class="header-actions">
          <button class="review-btn" @click="router.push('/review')">
            Review
          </button>
          <button class="wins-btn" @click="router.push('/wins')">
            Wins
          </button>
          <button class="settings-btn" @click="router.push('/settings')">
            Settings
          </button>
        </div>
      </div>
      <p class="shortcut-hint">
        Press <kbd>Cmd+Shift+L</kbd> to capture a friction note
      </p>
    </header>

    <!-- Active waiting/blocking states -->
    <div v-if="activeWaiting.length > 0" class="active-waiting">
      <div
        v-for="wp in activeWaiting"
        :key="wp.id"
        :class="['waiting-card', wp.direction]"
      >
        <div class="waiting-info">
          <span class="waiting-direction">{{
            wp.direction === "waiting" ? "Waiting" : "Blocking others"
          }}</span>
          <span class="waiting-note">{{ wp.note }}</span>
          <span class="waiting-elapsed">{{ elapsedSince(wp.start_time) }}</span>
        </div>
        <button class="resolve-btn" @click="endWaiting(wp.direction)">
          Resolve
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="tab-bar">
      <button
        :class="['tab-btn', { active: activeTab === 'logs' }]"
        @click="activeTab = 'logs'"
      >
        Logs
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'clusters' }]"
        @click="activeTab = 'clusters'"
      >
        Patterns
        <span v-if="clusters.length" class="tab-count">{{ clusters.length }}</span>
      </button>
    </div>

    <!-- === Clusters tab === -->
    <template v-if="activeTab === 'clusters'">
      <div class="cluster-actions">
        <button class="recompute-btn" @click="recomputeClusters" :disabled="clusterLoading">
          {{ clusterLoading ? "Analyzing..." : "Recompute patterns" }}
        </button>
      </div>

      <div v-if="clusters.length === 0" class="empty-state">
        <p>No patterns detected yet.</p>
        <p>Log more friction notes, then recompute.</p>
      </div>

      <ul v-else class="cluster-list">
        <li
          v-for="c in clusters"
          :key="c.id"
          class="cluster-card"
          @click="router.push(`/cluster/${c.id}`)"
        >
          <div class="cluster-top">
            <span
              class="cluster-category"
              :style="{ color: categoryColors[c.category] || '#888' }"
            >
              {{ categoryLabels[c.category] || c.category }}
            </span>
            <span class="cluster-confidence">
              {{ (c.confidence * 100).toFixed(0) }}%
            </span>
            <span :class="['cluster-status', c.status]">{{ c.status }}</span>
          </div>
          <h3 class="cluster-title">{{ c.title }}</h3>
          <p class="cluster-summary">{{ c.summary }}</p>
        </li>
      </ul>
    </template>

    <!-- === Logs tab === -->
    <template v-if="activeTab === 'logs'">
      <!-- Filters -->
      <div class="filter-bar">
        <button
          v-for="f in filters"
          :key="f.value"
          :class="['filter-btn', { active: activeFilter === f.value }]"
          @click="activeFilter = f.value"
        >
          {{ f.label }}
        </button>
      </div>

      <div v-if="logs.length === 0" class="empty-state">
        <p v-if="activeFilter === 'all'">No logs yet.</p>
        <p v-else>No logs matching this filter.</p>
      </div>

    <ul v-else class="log-list">
      <li v-for="log in logs" :key="log.id" class="log-entry">
        <div class="log-meta">
          <time class="log-time">{{ formatTime(log.created_at) }}</time>
          <span
            v-if="log.parsed_command"
            class="log-badge"
            :style="{
              background: categoryColors[log.category || ''] + '20' || 'rgba(128,128,128,0.1)',
              color: categoryColors[log.category || ''] || 'inherit',
              borderColor: categoryColors[log.category || ''] + '40' || 'rgba(128,128,128,0.2)',
            }"
          >
            {{ commandLabels[log.parsed_command] || log.parsed_command }}
          </span>
          <span v-if="log.app_context" class="log-app">{{
            log.app_context
          }}</span>
          <span v-if="log.source && log.source !== 'manual'" class="log-source">
            {{ log.source === 'investigation' ? 'imported' : log.source }}
          </span>
        </div>

        <!-- Editing mode -->
        <div v-if="editingLogId === log.id" class="log-edit">
          <input
            v-model="editLogText"
            class="log-edit-input"
            @keydown.enter.prevent="saveEditLog"
            @keydown.escape="cancelEditLog"
            autofocus
          />
          <div class="log-edit-actions">
            <button class="log-save-btn" @click="saveEditLog">Save</button>
            <button class="log-cancel-btn" @click="cancelEditLog">Cancel</button>
          </div>
        </div>

        <!-- Display mode -->
        <p
          v-else
          class="log-text"
          @dblclick="startEditLog(log)"
          title="Double-click to edit"
        >
          {{ log.note_text || log.raw_text }}
        </p>
      </li>
    </ul>
    </template>
  </div>
</template>

<style scoped>
.logs-container {
  max-width: 640px;
  margin: 0 auto;
  padding: 32px 24px;
}

.logs-header {
  margin-bottom: 24px;
}

.header-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.logs-header h1 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 4px;
}

.header-actions {
  display: flex;
  gap: 6px;
}

.review-btn,
.wins-btn,
.settings-btn {
  font-size: 13px;
  padding: 4px 12px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.review-btn {
  border-color: rgba(74, 144, 217, 0.3);
  color: #4a90d9;
}

.wins-btn {
  border-color: rgba(76, 175, 80, 0.3);
  color: #4caf50;
}

.review-btn:hover,
.settings-btn:hover {
  background: rgba(128, 128, 128, 0.1);
}

.shortcut-hint {
  font-size: 13px;
  opacity: 0.5;
}

kbd {
  font-family: inherit;
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: rgba(128, 128, 128, 0.1);
}

/* Active waiting states */
.active-waiting {
  margin-bottom: 16px;
}

.waiting-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-radius: 8px;
  margin-bottom: 8px;
}

.waiting-card.waiting {
  background: rgba(229, 115, 115, 0.08);
  border: 1px solid rgba(229, 115, 115, 0.2);
}

.waiting-card.blocking {
  background: rgba(255, 183, 77, 0.08);
  border: 1px solid rgba(255, 183, 77, 0.2);
}

.waiting-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.waiting-direction {
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.waiting-card.waiting .waiting-direction {
  color: #e57373;
}

.waiting-card.blocking .waiting-direction {
  color: #ffb74d;
}

.waiting-note {
  opacity: 0.7;
}

.waiting-elapsed {
  opacity: 0.4;
  font-size: 12px;
}

.resolve-btn {
  font-size: 12px;
  padding: 3px 10px;
  border-radius: 5px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.resolve-btn:hover {
  background: rgba(128, 128, 128, 0.1);
}

/* Filter bar */
.filter-bar {
  display: flex;
  gap: 4px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.filter-btn {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.15);
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.6;
}

.filter-btn.active {
  opacity: 1;
  background: rgba(128, 128, 128, 0.1);
  border-color: rgba(128, 128, 128, 0.3);
}

.filter-btn:hover {
  opacity: 0.9;
}

/* Log list */
.empty-state {
  text-align: center;
  padding: 64px 0;
  opacity: 0.5;
}

.log-list {
  list-style: none;
}

.log-entry {
  padding: 12px 0;
  border-bottom: 1px solid rgba(128, 128, 128, 0.15);
}

.log-entry:last-child {
  border-bottom: none;
}

.log-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
}

.log-time {
  font-size: 12px;
  opacity: 0.45;
}

.log-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid;
  font-weight: 500;
}

.log-app {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(74, 144, 217, 0.1);
  color: #4a90d9;
}

.log-text {
  font-size: 15px;
  line-height: 1.5;
  cursor: default;
}

.log-text:hover {
  background: rgba(128, 128, 128, 0.04);
  border-radius: 4px;
  margin: -2px -4px;
  padding: 2px 4px;
}

.log-edit {
  margin-top: 4px;
}

.log-edit-input {
  width: 100%;
  font-size: 15px;
  line-height: 1.5;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid rgba(74, 144, 217, 0.4);
  background: transparent;
  color: inherit;
  font-family: inherit;
  outline: none;
}

.log-edit-actions {
  display: flex;
  gap: 6px;
  margin-top: 4px;
}

.log-save-btn,
.log-cancel-btn {
  font-size: 12px;
  padding: 2px 10px;
  border-radius: 4px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.log-save-btn {
  border-color: rgba(74, 144, 217, 0.4);
  color: #4a90d9;
}

.log-source {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(156, 39, 176, 0.1);
  color: #9c27b0;
  font-weight: 500;
}

/* Tabs */
.tab-bar {
  display: flex;
  gap: 0;
  margin-bottom: 16px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.15);
}

.tab-btn {
  font-size: 14px;
  padding: 8px 16px;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.5;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
}

.tab-btn.active {
  opacity: 1;
  border-bottom-color: #4a90d9;
}

.tab-count {
  font-size: 11px;
  background: rgba(128, 128, 128, 0.15);
  padding: 1px 6px;
  border-radius: 8px;
  margin-left: 4px;
}

/* Clusters */
.cluster-actions {
  margin-bottom: 16px;
}

.recompute-btn {
  font-size: 13px;
  padding: 6px 14px;
  border-radius: 6px;
  border: 1px solid rgba(74, 144, 217, 0.3);
  background: rgba(74, 144, 217, 0.08);
  color: #4a90d9;
  cursor: pointer;
}

.recompute-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.cluster-list {
  list-style: none;
}

.cluster-card {
  padding: 14px 16px;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.1);
  margin-bottom: 10px;
  cursor: pointer;
  transition: background 0.15s;
}

.cluster-card:hover {
  background: rgba(128, 128, 128, 0.05);
}

.cluster-top {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.cluster-category {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.cluster-confidence {
  font-size: 11px;
  opacity: 0.4;
}

.cluster-status {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.cluster-status.new { background: rgba(128,128,128,0.1); opacity: 0.5; }
.cluster-status.watch { background: rgba(255,183,77,0.15); color: #ffb74d; }
.cluster-status.ignore { background: rgba(128,128,128,0.1); opacity: 0.4; }
.cluster-status.worth_fixing { background: rgba(76,175,80,0.15); color: #4caf50; }

.cluster-title {
  font-size: 15px;
  font-weight: 500;
  margin-bottom: 2px;
}

.cluster-summary {
  font-size: 13px;
  opacity: 0.5;
}
</style>
