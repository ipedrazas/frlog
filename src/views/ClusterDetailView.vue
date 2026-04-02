<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRoute, useRouter } from "vue-router";

const route = useRoute();
const router = useRouter();

interface LogEntry {
  id: number;
  raw_text: string;
  note_text: string;
  parsed_command: string | null;
  category: string | null;
  confidence: number | null;
  app_context: string | null;
  created_at: string;
}

interface ClusterDetail {
  cluster: {
    id: number;
    title: string;
    summary: string;
    category: string;
    confidence: number;
    status: string;
    log_count: number;
    created_at: string;
    updated_at: string;
  };
  logs: LogEntry[];
  app_contexts: string[];
}

interface AutomationBrief {
  id: number;
  cluster_id: number;
}

const detail = ref<ClusterDetail | null>(null);
const existingBrief = ref<AutomationBrief | null>(null);
const generatingBrief = ref(false);
const editingTitle = ref(false);
const editTitle = ref("");
const editingCategory = ref(false);
const editCategory = ref("");

const categories = [
  { value: "repetitive_work", label: "Repetitive Work" },
  { value: "bottleneck", label: "Bottleneck" },
  { value: "coordination_tax", label: "Coordination Tax" },
  { value: "context_switching", label: "Context Switching" },
  { value: "guilt_pile", label: "Guilt Pile" },
];

const categoryLabel = computed(() => {
  if (!detail.value) return "";
  return (
    categories.find((c) => c.value === detail.value!.cluster.category)?.label ||
    detail.value.cluster.category
  );
});

const confidenceLabel = computed(() => {
  if (!detail.value) return "";
  const c = detail.value.cluster.confidence;
  if (c >= 0.8) return "High";
  if (c >= 0.6) return "Medium";
  return "Low";
});

async function load() {
  const id = Number(route.params.id);
  detail.value = await invoke("get_cluster_detail", { clusterId: id });
  existingBrief.value = await invoke("get_brief_by_cluster", { clusterId: id });
}

async function generateBrief() {
  if (!detail.value) return;
  generatingBrief.value = true;
  const brief: AutomationBrief = await invoke("generate_brief", {
    clusterId: detail.value.cluster.id,
  });
  generatingBrief.value = false;
  router.push(`/brief/${brief.id}`);
}

async function setStatus(status: string) {
  if (!detail.value) return;
  await invoke("update_cluster_status", {
    clusterId: detail.value.cluster.id,
    status,
  });
  await load();
}

function startEditTitle() {
  if (!detail.value) return;
  editTitle.value = detail.value.cluster.title;
  editingTitle.value = true;
}

async function saveTitle() {
  if (!detail.value) return;
  await invoke("update_cluster_title", {
    clusterId: detail.value.cluster.id,
    title: editTitle.value,
  });
  editingTitle.value = false;
  await load();
}

function startEditCategory() {
  if (!detail.value) return;
  editCategory.value = detail.value.cluster.category;
  editingCategory.value = true;
}

async function saveCategory() {
  if (!detail.value) return;
  await invoke("update_cluster_category", {
    clusterId: detail.value.cluster.id,
    category: editCategory.value,
  });
  editingCategory.value = false;
  await load();
}

async function updateLogCategory(logId: number, category: string) {
  await invoke("update_log_category", { logId, category });
  await load();
}

function formatTime(iso: string): string {
  return new Date(iso).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

onMounted(load);
</script>

<template>
  <div class="cluster-container" v-if="detail">
    <header class="cluster-header">
      <button class="back-btn" @click="router.back()">&larr; Back</button>

      <div class="title-row" v-if="!editingTitle">
        <h1>{{ detail.cluster.title }}</h1>
        <button class="edit-btn" @click="startEditTitle">Edit</button>
      </div>
      <div class="title-edit" v-else>
        <input
          v-model="editTitle"
          @keydown.enter="saveTitle"
          @keydown.escape="editingTitle = false"
          autofocus
        />
        <button class="save-btn" @click="saveTitle">Save</button>
        <button class="cancel-btn" @click="editingTitle = false">Cancel</button>
      </div>

      <div class="meta-row">
        <span class="category-badge" v-if="!editingCategory" @click="startEditCategory">
          {{ categoryLabel }}
        </span>
        <select
          v-else
          v-model="editCategory"
          @change="saveCategory"
          @blur="editingCategory = false"
        >
          <option v-for="c in categories" :key="c.value" :value="c.value">
            {{ c.label }}
          </option>
        </select>

        <span class="confidence">
          Confidence: {{ confidenceLabel }}
          ({{ (detail.cluster.confidence * 100).toFixed(0) }}%)
        </span>

        <span class="log-count">{{ detail.cluster.log_count }} logs</span>

        <span :class="['status-badge', detail.cluster.status]">
          {{ detail.cluster.status }}
        </span>
      </div>
    </header>

    <p class="summary">{{ detail.cluster.summary }}</p>

    <!-- Triage actions -->
    <div class="triage-bar">
      <button
        :class="['triage-btn', { active: detail.cluster.status === 'ignore' }]"
        @click="setStatus('ignore')"
      >
        Ignore
      </button>
      <button
        :class="['triage-btn', { active: detail.cluster.status === 'watch' }]"
        @click="setStatus('watch')"
      >
        Watch
      </button>
      <button
        :class="[
          'triage-btn worth-fixing',
          { active: detail.cluster.status === 'worth_fixing' },
        ]"
        @click="setStatus('worth_fixing')"
      >
        Worth Fixing
      </button>
    </div>

    <!-- Generate Brief -->
    <div class="brief-action">
      <button
        v-if="existingBrief"
        class="brief-btn"
        @click="router.push(`/brief/${existingBrief.id}`)"
      >
        View Brief
      </button>
      <button
        v-else
        class="brief-btn generate"
        @click="generateBrief"
        :disabled="generatingBrief"
      >
        {{ generatingBrief ? "Generating..." : "Generate Automation Brief" }}
      </button>
    </div>

    <!-- App contexts -->
    <div v-if="detail.app_contexts.length" class="apps-section">
      <h3>Apps involved</h3>
      <div class="app-tags">
        <span v-for="app in detail.app_contexts" :key="app" class="app-tag">
          {{ app }}
        </span>
      </div>
    </div>

    <!-- Contributing logs -->
    <div class="logs-section">
      <h3>Contributing examples ({{ detail.logs.length }})</h3>
      <ul class="log-list">
        <li v-for="log in detail.logs" :key="log.id" class="log-entry">
          <div class="log-meta">
            <time>{{ formatTime(log.created_at) }}</time>
            <span v-if="log.app_context" class="log-app">{{
              log.app_context
            }}</span>
            <span v-if="log.confidence != null" class="log-confidence">
              {{ (log.confidence * 100).toFixed(0) }}%
            </span>
          </div>
          <p class="log-text">{{ log.note_text || log.raw_text }}</p>
          <div class="log-actions">
            <select
              class="category-select"
              :value="log.category || ''"
              @change="
                updateLogCategory(
                  log.id,
                  ($event.target as HTMLSelectElement).value
                )
              "
            >
              <option value="">No category</option>
              <option v-for="c in categories" :key="c.value" :value="c.value">
                {{ c.label }}
              </option>
            </select>
          </div>
        </li>
      </ul>
    </div>
  </div>

  <div v-else class="loading">Loading...</div>
</template>

<style scoped>
.cluster-container {
  max-width: 640px;
  margin: 0 auto;
  padding: 32px 24px;
}

.back-btn {
  font-size: 13px;
  background: none;
  border: none;
  color: #4a90d9;
  cursor: pointer;
  padding: 0;
  margin-bottom: 12px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-row h1 {
  font-size: 22px;
  font-weight: 600;
}

.edit-btn,
.save-btn,
.cancel-btn {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.title-edit {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 8px;
}

.title-edit input {
  flex: 1;
  font-size: 18px;
  font-weight: 600;
  padding: 4px 8px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  border-radius: 6px;
  background: transparent;
  color: inherit;
  outline: none;
}

.meta-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 6px;
  flex-wrap: wrap;
}

.category-badge {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  background: rgba(74, 144, 217, 0.1);
  color: #4a90d9;
  cursor: pointer;
}

.meta-row select {
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: transparent;
  color: inherit;
}

.confidence,
.log-count {
  font-size: 12px;
  opacity: 0.5;
}

.status-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  font-weight: 500;
}

.status-badge.new {
  background: rgba(128, 128, 128, 0.1);
  color: inherit;
  opacity: 0.5;
}

.status-badge.watch {
  background: rgba(255, 183, 77, 0.15);
  color: #ffb74d;
}

.status-badge.ignore {
  background: rgba(128, 128, 128, 0.1);
  color: inherit;
  opacity: 0.4;
}

.status-badge.worth_fixing {
  background: rgba(76, 175, 80, 0.15);
  color: #4caf50;
}

.summary {
  margin: 16px 0;
  font-size: 14px;
  opacity: 0.6;
}

.triage-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
}

.triage-btn {
  padding: 6px 14px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.triage-btn.active {
  border-color: rgba(128, 128, 128, 0.4);
  background: rgba(128, 128, 128, 0.1);
}

.triage-btn.worth-fixing.active {
  border-color: rgba(76, 175, 80, 0.4);
  background: rgba(76, 175, 80, 0.1);
  color: #4caf50;
}

.brief-action {
  margin-bottom: 20px;
}

.brief-btn {
  width: 100%;
  padding: 10px;
  font-size: 14px;
  font-weight: 500;
  border-radius: 8px;
  border: 1px solid rgba(74, 144, 217, 0.3);
  background: rgba(74, 144, 217, 0.06);
  color: #4a90d9;
  cursor: pointer;
}

.brief-btn.generate {
  background: #4a90d9;
  color: white;
  border: none;
}

.brief-btn.generate:hover {
  background: #3a7bc8;
}

.brief-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.apps-section {
  margin-bottom: 20px;
}

.apps-section h3,
.logs-section h3 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 8px;
  opacity: 0.7;
}

.app-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.app-tag {
  font-size: 12px;
  padding: 3px 8px;
  border-radius: 4px;
  background: rgba(74, 144, 217, 0.1);
  color: #4a90d9;
}

.log-list {
  list-style: none;
}

.log-entry {
  padding: 10px 0;
  border-bottom: 1px solid rgba(128, 128, 128, 0.1);
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

.log-meta time {
  font-size: 12px;
  opacity: 0.4;
}

.log-app {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(74, 144, 217, 0.1);
  color: #4a90d9;
}

.log-confidence {
  font-size: 11px;
  opacity: 0.4;
}

.log-text {
  font-size: 14px;
  line-height: 1.5;
}

.log-actions {
  margin-top: 4px;
}

.category-select {
  font-size: 11px;
  padding: 2px 4px;
  border-radius: 4px;
  border: 1px solid rgba(128, 128, 128, 0.15);
  background: transparent;
  color: inherit;
  opacity: 0.6;
}

.loading {
  text-align: center;
  padding: 64px;
  opacity: 0.5;
}
</style>
