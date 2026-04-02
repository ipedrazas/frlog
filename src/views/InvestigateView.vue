<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRoute, useRouter } from "vue-router";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

const route = useRoute();
const router = useRouter();

interface ImportResult {
  patterns_imported: number;
  logs_created: number;
}

const prompt = ref("");
const reportText = ref("");
const copyFeedback = ref("");
const importing = ref(false);
const importResult = ref<ImportResult | null>(null);
const importError = ref("");
const step = ref<"prompt" | "import">("prompt");

async function loadPrompt() {
  const clusterId = route.query.cluster
    ? Number(route.query.cluster)
    : null;
  prompt.value = await invoke("generate_investigation_prompt", { clusterId });
}

async function copyPrompt() {
  try {
    await writeText(prompt.value);
  } catch {
    await navigator.clipboard.writeText(prompt.value);
  }
  copyFeedback.value = "Copied!";
  setTimeout(() => (copyFeedback.value = ""), 2000);
}

async function importReport() {
  if (!reportText.value.trim()) return;
  importing.value = true;
  importError.value = "";
  importResult.value = null;

  try {
    importResult.value = await invoke("import_investigation_report", {
      reportText: reportText.value,
    });
  } catch (e) {
    importError.value = String(e);
  }
  importing.value = false;
}

onMounted(loadPrompt);
</script>

<template>
  <div class="investigate-container">
    <header class="investigate-header">
      <button class="back-btn" @click="router.back()">&larr; Back</button>
      <h1>Investigate</h1>
      <p class="subtitle">
        Use an AI agent to analyze your email and calendar for friction patterns
      </p>
    </header>

    <!-- Step tabs -->
    <div class="step-tabs">
      <button
        :class="['step-tab', { active: step === 'prompt' }]"
        @click="step = 'prompt'"
      >
        1. Copy prompt
      </button>
      <span class="step-arrow">&rarr;</span>
      <button
        :class="['step-tab', { active: step === 'import' }]"
        @click="step = 'import'"
      >
        2. Import report
      </button>
    </div>

    <!-- Step 1: Prompt -->
    <div v-if="step === 'prompt'" class="step-content">
      <div class="instructions">
        <h2>Copy this prompt to your AI agent</h2>
        <p>
          Paste it into Claude Cowork, Copilot, ChatGPT, or any AI agent that
          has access to your email and calendar. The prompt asks the agent to
          analyze your data and return a structured report that frlog can
          import.
        </p>
      </div>

      <div class="prompt-actions">
        <button class="copy-btn" @click="copyPrompt">
          {{ copyFeedback || "Copy prompt to clipboard" }}
        </button>
        <button class="next-btn" @click="step = 'import'">
          Next: Import report &rarr;
        </button>
      </div>

      <pre class="prompt-display">{{ prompt }}</pre>
    </div>

    <!-- Step 2: Import -->
    <div v-if="step === 'import'" class="step-content">
      <div class="instructions">
        <h2>Paste the AI agent's response</h2>
        <p>
          After the AI agent analyzes your email/calendar, paste its JSON
          response here. frlog will import the discovered patterns as new
          friction log entries.
        </p>
      </div>

      <textarea
        v-model="reportText"
        class="report-input"
        :rows="12"
        placeholder='Paste the JSON report here...

{
  "patterns": [
    {
      "description": "...",
      "category": "repetitive_work",
      ...
    }
  ]
}'
        :disabled="importing"
      />

      <div class="import-actions">
        <button
          class="import-btn"
          @click="importReport"
          :disabled="importing || !reportText.trim()"
        >
          {{ importing ? "Importing..." : "Import report" }}
        </button>
      </div>

      <!-- Success -->
      <div v-if="importResult" class="import-success">
        <h3>Import complete</h3>
        <p>
          {{ importResult.patterns_imported }} patterns found,
          {{ importResult.logs_created }} log entries created.
        </p>
        <p class="import-hint">
          Go to the Patterns tab and click "Recompute patterns" to see how
          these findings cluster with your existing logs.
        </p>
        <button class="go-logs-btn" @click="router.push('/logs')">
          View logs
        </button>
      </div>

      <!-- Error -->
      <div v-if="importError" class="import-error">
        <h3>Import failed</h3>
        <p>{{ importError }}</p>
        <p class="error-hint">
          Make sure the AI agent returned valid JSON matching the expected
          schema. The JSON should start with <code>{"patterns": [...]}</code>.
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.investigate-container {
  max-width: 720px;
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

.investigate-header h1 {
  font-size: 24px;
  font-weight: 600;
}

.subtitle {
  font-size: 13px;
  opacity: 0.5;
  margin-bottom: 20px;
}

/* Step tabs */
.step-tabs {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 20px;
}

.step-tab {
  font-size: 14px;
  padding: 8px 16px;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.15);
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.5;
}

.step-tab.active {
  opacity: 1;
  background: rgba(74, 144, 217, 0.08);
  border-color: rgba(74, 144, 217, 0.3);
  color: #4a90d9;
}

.step-arrow {
  opacity: 0.2;
}

/* Instructions */
.instructions {
  margin-bottom: 16px;
}

.instructions h2 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
}

.instructions p {
  font-size: 14px;
  opacity: 0.6;
  line-height: 1.5;
}

/* Prompt display */
.prompt-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.copy-btn {
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 6px;
  border: none;
  background: #4a90d9;
  color: white;
  cursor: pointer;
}

.copy-btn:hover {
  background: #3a7bc8;
}

.next-btn {
  padding: 8px 16px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.prompt-display {
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: inherit;
  padding: 16px;
  border-radius: 10px;
  background: rgba(128, 128, 128, 0.04);
  border: 1px solid rgba(128, 128, 128, 0.1);
  max-height: 500px;
  overflow-y: auto;
  margin: 0;
}

/* Report input */
.report-input {
  width: 100%;
  font-size: 13px;
  line-height: 1.5;
  padding: 12px;
  border-radius: 10px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  font-family: monospace;
  resize: vertical;
  outline: none;
  margin-bottom: 12px;
}

.report-input:focus {
  border-color: rgba(74, 144, 217, 0.4);
}

.import-actions {
  margin-bottom: 16px;
}

.import-btn {
  padding: 8px 20px;
  font-size: 14px;
  font-weight: 500;
  border-radius: 6px;
  border: none;
  background: #4a90d9;
  color: white;
  cursor: pointer;
}

.import-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

/* Results */
.import-success {
  padding: 16px;
  border-radius: 10px;
  background: rgba(76, 175, 80, 0.06);
  border: 1px solid rgba(76, 175, 80, 0.2);
}

.import-success h3 {
  font-size: 15px;
  font-weight: 600;
  color: #4caf50;
  margin-bottom: 4px;
}

.import-success p {
  font-size: 14px;
}

.import-hint {
  opacity: 0.6;
  margin-top: 4px;
}

.go-logs-btn {
  margin-top: 10px;
  padding: 6px 14px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid rgba(76, 175, 80, 0.3);
  background: transparent;
  color: #4caf50;
  cursor: pointer;
}

.import-error {
  padding: 16px;
  border-radius: 10px;
  background: rgba(229, 115, 115, 0.06);
  border: 1px solid rgba(229, 115, 115, 0.2);
}

.import-error h3 {
  font-size: 15px;
  font-weight: 600;
  color: #e57373;
  margin-bottom: 4px;
}

.import-error p {
  font-size: 14px;
}

.error-hint {
  opacity: 0.6;
  margin-top: 4px;
  font-size: 13px;
}

.error-hint code {
  background: rgba(128, 128, 128, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 12px;
}
</style>
