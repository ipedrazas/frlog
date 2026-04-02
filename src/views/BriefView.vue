<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRoute, useRouter } from "vue-router";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

const route = useRoute();
const router = useRouter();

interface AutomationBrief {
  id: number;
  cluster_id: number;
  problem: string;
  trigger: string;
  current_workflow: string;
  apps_involved: string;
  frequency: string;
  estimated_time_cost: string;
  emotional_cost: string;
  dependencies: string;
  example_instances: string;
  desired_outcome: string;
  constraints: string;
  candidate_approaches: string;
  agent_spec: string;
  exported_at: string | null;
  resolution_status: string | null;
  resolved_at: string | null;
  estimated_savings_mins: number | null;
  outcome_note: string | null;
  created_at: string;
}

const brief = ref<AutomationBrief | null>(null);
const editing = ref<string | null>(null);
const editValue = ref("");
const copyFeedback = ref("");
const loading = ref(true);
const showOutcomeForm = ref(false);
const outcomeStatus = ref("");
const outcomeSavings = ref<string>("");
const outcomeNote = ref("");

const sections = [
  { key: "problem", label: "Problem" },
  { key: "trigger", label: "Trigger" },
  { key: "current_workflow", label: "Current Workflow" },
  { key: "apps_involved", label: "Apps Involved" },
  { key: "frequency", label: "Frequency" },
  { key: "estimated_time_cost", label: "Estimated Time Cost" },
  { key: "emotional_cost", label: "Emotional Cost" },
  { key: "dependencies", label: "Dependencies / Blockers" },
  { key: "example_instances", label: "Example Instances" },
  { key: "desired_outcome", label: "Desired Outcome" },
  { key: "constraints", label: "Constraints" },
  { key: "candidate_approaches", label: "Candidate Automation Approaches" },
  { key: "agent_spec", label: "Agent-Ready Spec" },
] as const;

async function load() {
  loading.value = true;
  const briefId = Number(route.params.id);
  brief.value = await invoke("get_brief", { briefId });
  loading.value = false;
}

function startEdit(key: string) {
  if (!brief.value) return;
  editing.value = key;
  editValue.value = (brief.value as any)[key] || "";
}

async function saveEdit() {
  if (!brief.value || !editing.value) return;
  (brief.value as any)[editing.value] = editValue.value;
  await invoke("update_brief", { brief: brief.value });
  editing.value = null;
}

function cancelEdit() {
  editing.value = null;
}

function formatAsPlainText(): string {
  if (!brief.value) return "";
  return sections
    .map((s) => `${s.label}:\n${(brief.value as any)[s.key] || "—"}`)
    .join("\n\n");
}

function formatAsMarkdown(): string {
  if (!brief.value) return "";
  return (
    "# Automation Brief\n\n" +
    sections
      .map((s) => `## ${s.label}\n\n${(brief.value as any)[s.key] || "—"}`)
      .join("\n\n")
  );
}

async function copyAs(format: "plain" | "markdown" | "agent") {
  if (!brief.value) return;

  let text: string;
  if (format === "plain") {
    text = formatAsPlainText();
  } else if (format === "markdown") {
    text = formatAsMarkdown();
  } else {
    text = brief.value.agent_spec;
  }

  try {
    await writeText(text);
    await invoke("mark_brief_exported", { briefId: brief.value.id });
    brief.value.exported_at = new Date().toISOString();

    copyFeedback.value =
      format === "plain"
        ? "Copied as plain text"
        : format === "markdown"
          ? "Copied as Markdown"
          : "Copied agent spec";
    setTimeout(() => (copyFeedback.value = ""), 2000);
  } catch {
    // Fallback: use navigator clipboard
    await navigator.clipboard.writeText(text);
    await invoke("mark_brief_exported", { briefId: brief.value.id });
    brief.value.exported_at = new Date().toISOString();
    copyFeedback.value = "Copied!";
    setTimeout(() => (copyFeedback.value = ""), 2000);
  }
}

function startOutcome(status: string) {
  outcomeStatus.value = status;
  outcomeSavings.value = "";
  outcomeNote.value = "";
  showOutcomeForm.value = true;
}

async function submitOutcome() {
  if (!brief.value || !outcomeStatus.value) return;
  const savings = outcomeSavings.value ? parseInt(outcomeSavings.value) : null;
  await invoke("update_brief_outcome", {
    briefId: brief.value.id,
    status: outcomeStatus.value,
    estimatedSavingsMins: savings,
    outcomeNote: outcomeNote.value || null,
  });
  showOutcomeForm.value = false;
  await load();
}

const outcomeLabels: Record<string, string> = {
  resolved: "Resolved",
  reduced: "Reduced",
  unchanged: "Unchanged",
};

onMounted(load);
</script>

<template>
  <div class="brief-container" v-if="brief">
    <header class="brief-header">
      <button class="back-btn" @click="router.back()">&larr; Back</button>
      <h1>Automation Brief</h1>
      <p class="brief-meta">
        <span v-if="brief.exported_at" class="exported-badge">Exported</span>
        <span class="brief-date">
          Created {{ new Date(brief.created_at).toLocaleDateString() }}
        </span>
      </p>
    </header>

    <!-- Export bar -->
    <div class="export-bar">
      <button class="export-btn" @click="copyAs('plain')">
        Copy as Plain Text
      </button>
      <button class="export-btn" @click="copyAs('markdown')">
        Copy as Markdown
      </button>
      <button class="export-btn primary" @click="copyAs('agent')">
        Copy Agent Spec
      </button>
      <span v-if="copyFeedback" class="copy-feedback">{{ copyFeedback }}</span>
    </div>

    <!-- Outcome tracking -->
    <div class="outcome-section">
      <div v-if="brief.resolution_status" class="outcome-result">
        <div class="outcome-header">
          <span :class="['outcome-badge', brief.resolution_status]">
            {{ outcomeLabels[brief.resolution_status] || brief.resolution_status }}
          </span>
          <span v-if="brief.estimated_savings_mins" class="outcome-savings">
            ~{{ brief.estimated_savings_mins }}min/week saved
          </span>
          <span v-if="brief.resolved_at" class="outcome-date">
            {{ new Date(brief.resolved_at).toLocaleDateString() }}
          </span>
        </div>
        <p v-if="brief.outcome_note" class="outcome-note">{{ brief.outcome_note }}</p>
        <button class="update-outcome-btn" @click="showOutcomeForm = true">
          Update outcome
        </button>
      </div>

      <div v-else-if="!showOutcomeForm" class="outcome-prompt">
        <p>Did this help?</p>
        <div class="outcome-buttons">
          <button class="outcome-btn resolved" @click="startOutcome('resolved')">Resolved</button>
          <button class="outcome-btn reduced" @click="startOutcome('reduced')">Reduced</button>
          <button class="outcome-btn unchanged" @click="startOutcome('unchanged')">Unchanged</button>
        </div>
      </div>

      <div v-if="showOutcomeForm" class="outcome-form">
        <div class="outcome-status-select">
          <button
            v-for="s in ['resolved', 'reduced', 'unchanged']"
            :key="s"
            :class="['outcome-btn', s, { active: outcomeStatus === s }]"
            @click="outcomeStatus = s"
          >
            {{ outcomeLabels[s] }}
          </button>
        </div>
        <label class="form-field">
          <span>Estimated time saved per week (minutes, optional)</span>
          <input
            v-model="outcomeSavings"
            type="number"
            placeholder="e.g. 30"
            min="0"
          />
        </label>
        <label class="form-field">
          <span>Notes (optional)</span>
          <input
            v-model="outcomeNote"
            placeholder="What worked? What didn't?"
          />
        </label>
        <div class="form-actions">
          <button class="submit-btn" @click="submitOutcome" :disabled="!outcomeStatus">
            Save outcome
          </button>
          <button class="cancel-form-btn" @click="showOutcomeForm = false">Cancel</button>
        </div>
      </div>
    </div>

    <!-- Brief sections -->
    <div class="sections">
      <div
        v-for="s in sections"
        :key="s.key"
        class="section"
        :class="{ 'agent-section': s.key === 'agent_spec' }"
      >
        <div class="section-header">
          <h3>{{ s.label }}</h3>
          <button
            v-if="editing !== s.key"
            class="edit-btn"
            @click="startEdit(s.key)"
          >
            Edit
          </button>
        </div>

        <div v-if="editing === s.key" class="section-edit">
          <textarea
            v-model="editValue"
            rows="4"
            @keydown.escape="cancelEdit"
          />
          <div class="edit-actions">
            <button class="save-btn" @click="saveEdit">Save</button>
            <button class="cancel-btn" @click="cancelEdit">Cancel</button>
          </div>
        </div>

        <pre
          v-else
          class="section-content"
        >{{ (brief as any)[s.key] || "—" }}</pre>
      </div>
    </div>
  </div>

  <div v-else-if="loading" class="loading">Loading brief...</div>
  <div v-else class="loading">Brief not found.</div>
</template>

<style scoped>
.brief-container {
  max-width: 680px;
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
  margin-bottom: 8px;
}

.brief-header h1 {
  font-size: 22px;
  font-weight: 600;
}

.brief-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  margin-bottom: 16px;
}

.exported-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: rgba(76, 175, 80, 0.15);
  color: #4caf50;
  font-weight: 500;
}

.brief-date {
  font-size: 12px;
  opacity: 0.4;
}

/* Export bar */
.export-bar {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 24px;
  padding: 12px 16px;
  background: rgba(128, 128, 128, 0.04);
  border-radius: 10px;
  border: 1px solid rgba(128, 128, 128, 0.1);
}

.export-btn {
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.export-btn:hover {
  background: rgba(128, 128, 128, 0.1);
}

.export-btn.primary {
  border-color: rgba(74, 144, 217, 0.4);
  background: rgba(74, 144, 217, 0.08);
  color: #4a90d9;
  font-weight: 500;
}

.export-btn.primary:hover {
  background: rgba(74, 144, 217, 0.15);
}

.copy-feedback {
  font-size: 12px;
  color: #4caf50;
  font-weight: 500;
  margin-left: auto;
}

/* Sections */
.section {
  margin-bottom: 16px;
  padding: 12px 16px;
  background: rgba(128, 128, 128, 0.03);
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.06);
}

.section.agent-section {
  border-color: rgba(74, 144, 217, 0.2);
  background: rgba(74, 144, 217, 0.03);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.section-header h3 {
  font-size: 13px;
  font-weight: 600;
  opacity: 0.6;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.edit-btn {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid rgba(128, 128, 128, 0.15);
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.4;
}

.edit-btn:hover {
  opacity: 0.8;
}

.section-content {
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: inherit;
  margin: 0;
}

.section-edit textarea {
  width: 100%;
  font-size: 14px;
  line-height: 1.5;
  padding: 8px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: transparent;
  color: inherit;
  font-family: inherit;
  resize: vertical;
  outline: none;
}

.edit-actions {
  display: flex;
  gap: 6px;
  margin-top: 6px;
}

.save-btn,
.cancel-btn {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 4px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.save-btn {
  border-color: rgba(74, 144, 217, 0.4);
  color: #4a90d9;
}

/* Outcome section */
.outcome-section {
  margin-bottom: 24px;
  padding: 16px;
  border-radius: 10px;
  background: rgba(128, 128, 128, 0.04);
  border: 1px solid rgba(128, 128, 128, 0.1);
}

.outcome-prompt p {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 10px;
}

.outcome-buttons,
.outcome-status-select {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.outcome-btn {
  padding: 6px 14px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.outcome-btn.resolved { border-color: rgba(76,175,80,0.3); color: #4caf50; }
.outcome-btn.reduced { border-color: rgba(74,144,217,0.3); color: #4a90d9; }
.outcome-btn.unchanged { border-color: rgba(255,183,77,0.3); color: #ffb74d; }

.outcome-btn.active {
  font-weight: 600;
}

.outcome-btn.resolved.active { background: rgba(76,175,80,0.1); }
.outcome-btn.reduced.active { background: rgba(74,144,217,0.1); }
.outcome-btn.unchanged.active { background: rgba(255,183,77,0.1); }

.outcome-result {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.outcome-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.outcome-badge {
  font-size: 12px;
  padding: 3px 10px;
  border-radius: 5px;
  font-weight: 600;
  text-transform: capitalize;
}

.outcome-badge.resolved { background: rgba(76,175,80,0.15); color: #4caf50; }
.outcome-badge.reduced { background: rgba(74,144,217,0.15); color: #4a90d9; }
.outcome-badge.unchanged { background: rgba(255,183,77,0.15); color: #ffb74d; }

.outcome-savings {
  font-size: 13px;
  font-weight: 500;
  color: #4caf50;
}

.outcome-date {
  font-size: 12px;
  opacity: 0.4;
}

.outcome-note {
  font-size: 13px;
  opacity: 0.6;
}

.update-outcome-btn {
  font-size: 12px;
  background: none;
  border: none;
  color: #4a90d9;
  cursor: pointer;
  padding: 0;
  align-self: flex-start;
}

.form-field {
  display: block;
  margin-bottom: 10px;
}

.form-field span {
  display: block;
  font-size: 12px;
  opacity: 0.5;
  margin-bottom: 4px;
}

.form-field input {
  width: 100%;
  padding: 6px 10px;
  font-size: 14px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  outline: none;
  font-family: inherit;
}

.form-actions {
  display: flex;
  gap: 8px;
}

.submit-btn {
  padding: 6px 16px;
  font-size: 13px;
  border-radius: 6px;
  border: none;
  background: #4a90d9;
  color: white;
  cursor: pointer;
}

.submit-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

.cancel-form-btn {
  padding: 6px 12px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.loading {
  text-align: center;
  padding: 64px;
  opacity: 0.5;
}
</style>
