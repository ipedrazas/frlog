<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";

const router = useRouter();

interface WinEntry {
  brief_id: number;
  cluster_title: string;
  category: string;
  resolution_status: string;
  resolved_at: string;
  estimated_savings_mins: number | null;
  outcome_note: string | null;
  exported_at: string | null;
}

interface AutomationBrief {
  id: number;
  exported_at: string | null;
  resolution_status: string | null;
}

const wins = ref<WinEntry[]>([]);
const totalSavings = ref(0);
const pendingFollowups = ref<AutomationBrief[]>([]);

const categoryLabels: Record<string, string> = {
  repetitive_work: "Repetitive Work",
  bottleneck: "Bottleneck",
  coordination_tax: "Coordination Tax",
  context_switching: "Context Switching",
  guilt_pile: "Guilt Pile",
};

const categoryColors: Record<string, string> = {
  bottleneck: "#e57373",
  repetitive_work: "#ffb74d",
  coordination_tax: "#ba68c8",
  context_switching: "#4dd0e1",
  guilt_pile: "#7986cb",
};

const resolvedCount = computed(() =>
  wins.value.filter((w) => w.resolution_status === "resolved").length
);
const reducedCount = computed(() =>
  wins.value.filter((w) => w.resolution_status === "reduced").length
);

function formatSavings(mins: number): string {
  if (mins < 60) return `${mins}min`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

async function load() {
  const [w, s, f] = await Promise.all([
    invoke<WinEntry[]>("get_wins"),
    invoke<number>("get_total_savings_mins"),
    invoke<AutomationBrief[]>("get_briefs_needing_followup"),
  ]);
  wins.value = w;
  totalSavings.value = s;
  pendingFollowups.value = f;
}

onMounted(load);
</script>

<template>
  <div class="wins-container">
    <header class="wins-header">
      <button class="back-btn" @click="router.push('/logs')">&larr; Back</button>
      <h1>Wins</h1>
      <p class="subtitle">Friction you've actually reduced</p>
    </header>

    <!-- Follow-up prompts -->
    <div v-if="pendingFollowups.length" class="followup-section">
      <h2>Pending follow-ups</h2>
      <p class="followup-hint">
        These briefs were exported over a week ago. Did they help?
      </p>
      <div
        v-for="b in pendingFollowups"
        :key="b.id"
        class="followup-card"
        @click="router.push(`/brief/${b.id}`)"
      >
        <span class="followup-text">Brief #{{ b.id }}</span>
        <span class="followup-arrow">&rarr;</span>
      </div>
    </div>

    <!-- Summary stats -->
    <div class="stats-grid" v-if="wins.length">
      <div class="stat-card">
        <span class="stat-value">{{ resolvedCount }}</span>
        <span class="stat-label">Resolved</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ reducedCount }}</span>
        <span class="stat-label">Reduced</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">
          {{ totalSavings > 0 ? formatSavings(totalSavings) : "—" }}
        </span>
        <span class="stat-label">Saved/week</span>
      </div>
    </div>

    <!-- Wins timeline -->
    <div v-if="wins.length" class="timeline">
      <div
        v-for="w in wins"
        :key="w.brief_id"
        :class="['win-card', w.resolution_status]"
        @click="router.push(`/brief/${w.brief_id}`)"
      >
        <div class="win-top">
          <span
            :class="['win-status', w.resolution_status]"
          >
            {{ w.resolution_status }}
          </span>
          <span
            class="win-category"
            :style="{ color: categoryColors[w.category] || '#888' }"
          >
            {{ categoryLabels[w.category] || w.category }}
          </span>
          <span class="win-date">{{ formatDate(w.resolved_at) }}</span>
        </div>
        <h3 class="win-title">{{ w.cluster_title }}</h3>
        <div class="win-details">
          <span v-if="w.estimated_savings_mins" class="win-savings">
            ~{{ formatSavings(w.estimated_savings_mins) }}/week saved
          </span>
          <p v-if="w.outcome_note" class="win-note">{{ w.outcome_note }}</p>
        </div>
        <p
          v-if="w.resolution_status === 'unchanged'"
          class="revisit-hint"
        >
          Worth revisiting?
        </p>
      </div>
    </div>

    <div v-else-if="!pendingFollowups.length" class="empty-state">
      <p>No outcomes recorded yet.</p>
      <p>Generate and export automation briefs, then track whether they helped.</p>
    </div>
  </div>
</template>

<style scoped>
.wins-container {
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
  margin-bottom: 8px;
}

.wins-header h1 {
  font-size: 24px;
  font-weight: 600;
}

.subtitle {
  font-size: 13px;
  opacity: 0.5;
  margin-bottom: 20px;
}

/* Follow-ups */
.followup-section {
  margin-bottom: 24px;
  padding: 14px 16px;
  border-radius: 10px;
  background: rgba(255, 183, 77, 0.06);
  border: 1px solid rgba(255, 183, 77, 0.2);
}

.followup-section h2 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.followup-hint {
  font-size: 12px;
  opacity: 0.5;
  margin-bottom: 10px;
}

.followup-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-radius: 6px;
  background: rgba(128, 128, 128, 0.05);
  cursor: pointer;
  margin-bottom: 4px;
}

.followup-card:hover {
  background: rgba(128, 128, 128, 0.1);
}

.followup-text {
  font-size: 13px;
}

.followup-arrow {
  opacity: 0.3;
}

/* Stats */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
  margin-bottom: 24px;
}

.stat-card {
  background: rgba(128, 128, 128, 0.06);
  border-radius: 10px;
  padding: 14px 16px;
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 2px;
}

.stat-label {
  font-size: 12px;
  opacity: 0.5;
}

/* Timeline */
.timeline {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.win-card {
  padding: 14px 16px;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.1);
  cursor: pointer;
  transition: background 0.15s;
}

.win-card:hover {
  background: rgba(128, 128, 128, 0.04);
}

.win-card.unchanged {
  opacity: 0.6;
}

.win-top {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.win-status {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 600;
  text-transform: capitalize;
}

.win-status.resolved { background: rgba(76,175,80,0.15); color: #4caf50; }
.win-status.reduced { background: rgba(74,144,217,0.15); color: #4a90d9; }
.win-status.unchanged { background: rgba(255,183,77,0.15); color: #ffb74d; }

.win-category {
  font-size: 11px;
  font-weight: 500;
}

.win-date {
  font-size: 11px;
  opacity: 0.4;
  margin-left: auto;
}

.win-title {
  font-size: 15px;
  font-weight: 500;
}

.win-details {
  margin-top: 4px;
}

.win-savings {
  font-size: 13px;
  color: #4caf50;
  font-weight: 500;
}

.win-note {
  font-size: 13px;
  opacity: 0.5;
  margin-top: 2px;
}

.revisit-hint {
  font-size: 12px;
  color: #ffb74d;
  margin-top: 6px;
  font-style: italic;
}

.empty-state {
  text-align: center;
  padding: 64px 0;
  opacity: 0.5;
}

.empty-state p + p {
  margin-top: 4px;
}
</style>
