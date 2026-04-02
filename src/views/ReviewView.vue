<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";

const router = useRouter();

interface CategoryCount {
  category: string;
  count: number;
}

interface ReviewStats {
  total_logs: number;
  logs_with_category: number;
  total_waiting_secs: number;
  category_counts: CategoryCount[];
  period_start: string;
  period_end: string;
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

const stats = ref<ReviewStats | null>(null);
const clusters = ref<FrictionCluster[]>([]);
const showIgnored = ref(false);
const cadence = ref<"week" | "day">("week");

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

function getDateRange(): { since: string; until: string; label: string } {
  const now = new Date();
  const until = new Date(now);
  until.setDate(until.getDate() + 1);
  until.setHours(0, 0, 0, 0);

  const since = new Date(until);
  if (cadence.value === "week") {
    since.setDate(since.getDate() - 7);
  } else {
    since.setDate(since.getDate() - 1);
  }

  const fmt = (d: Date) =>
    d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  const label =
    cadence.value === "week"
      ? `${fmt(since)} – ${fmt(now)}`
      : fmt(now);

  return {
    since: since.toISOString().slice(0, 23),
    until: until.toISOString().slice(0, 23),
    label,
  };
}

const dateLabel = ref("");

async function loadReview() {
  const range = getDateRange();
  dateLabel.value = range.label;

  const [s, c] = await Promise.all([
    invoke<ReviewStats>("get_review_stats", {
      since: range.since,
      until: range.until,
    }),
    invoke<FrictionCluster[]>("get_clusters_for_review", {
      showIgnored: showIgnored.value,
    }),
  ]);
  stats.value = s;
  clusters.value = c;
}

const topClusters = computed(() => clusters.value.slice(0, 3));
const remainingClusters = computed(() => clusters.value.slice(3));

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.round(secs / 60)}m`;
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

async function setStatus(clusterId: number, status: string) {
  await invoke("update_cluster_status", { clusterId, status });
  await loadReview();
}

async function toggleCadence() {
  cadence.value = cadence.value === "week" ? "day" : "week";
  await loadReview();
}

async function toggleShowIgnored() {
  showIgnored.value = !showIgnored.value;
  await loadReview();
}

onMounted(loadReview);
</script>

<template>
  <div class="review-container">
    <header class="review-header">
      <button class="back-btn" @click="router.push('/logs')">&larr; Back</button>
      <div class="header-top">
        <h1>Review</h1>
        <button class="cadence-btn" @click="toggleCadence">
          {{ cadence === "week" ? "Weekly" : "Daily" }}
        </button>
      </div>
      <p class="date-range">{{ dateLabel }}</p>
    </header>

    <!-- Aggregate stats -->
    <div class="stats-grid" v-if="stats">
      <div class="stat-card">
        <span class="stat-value">{{ stats.total_logs }}</span>
        <span class="stat-label">Friction logs</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ stats.logs_with_category }}</span>
        <span class="stat-label">Categorized</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{
          stats.total_waiting_secs > 0
            ? formatDuration(stats.total_waiting_secs)
            : "—"
        }}</span>
        <span class="stat-label">Waiting time</span>
      </div>
    </div>

    <!-- Category breakdown -->
    <div class="category-breakdown" v-if="stats && stats.category_counts.length">
      <h2>Top friction categories</h2>
      <div class="category-bars">
        <div
          v-for="cc in stats.category_counts"
          :key="cc.category"
          class="category-row"
        >
          <span
            class="cat-label"
            :style="{ color: categoryColors[cc.category] || '#888' }"
          >
            {{ categoryLabels[cc.category] || cc.category }}
          </span>
          <div class="bar-track">
            <div
              class="bar-fill"
              :style="{
                width: `${Math.min(100, (cc.count / stats.total_logs) * 100)}%`,
                background: categoryColors[cc.category] || '#888',
              }"
            />
          </div>
          <span class="cat-count">{{ cc.count }}</span>
        </div>
      </div>
    </div>

    <!-- Top 3 clusters -->
    <div class="top-section" v-if="topClusters.length">
      <h2>Top opportunities</h2>
      <p class="section-hint">These patterns appear most often. Worth fixing?</p>

      <div
        v-for="(c, i) in topClusters"
        :key="c.id"
        :class="['review-cluster', c.status]"
      >
        <div class="cluster-rank">#{{ i + 1 }}</div>
        <div class="cluster-body" @click="router.push(`/cluster/${c.id}`)">
          <div class="cluster-top">
            <span
              class="cluster-category"
              :style="{ color: categoryColors[c.category] || '#888' }"
            >
              {{ categoryLabels[c.category] || c.category }}
            </span>
            <span class="cluster-count">{{ c.log_count }} logs</span>
            <span class="cluster-confidence">
              {{ (c.confidence * 100).toFixed(0) }}%
            </span>
          </div>
          <h3 class="cluster-title">{{ c.title }}</h3>
          <p class="cluster-summary">{{ c.summary }}</p>
        </div>
        <div class="triage-controls">
          <button
            :class="['tri-btn', { active: c.status === 'ignore' }]"
            @click.stop="setStatus(c.id, 'ignore')"
            title="Ignore"
          >
            Ignore
          </button>
          <button
            :class="['tri-btn', { active: c.status === 'watch' }]"
            @click.stop="setStatus(c.id, 'watch')"
            title="Watch"
          >
            Watch
          </button>
          <button
            :class="['tri-btn fix', { active: c.status === 'worth_fixing' }]"
            @click.stop="setStatus(c.id, 'worth_fixing')"
            title="Worth Fixing"
          >
            Worth Fixing
          </button>
        </div>
      </div>
    </div>

    <!-- Remaining clusters -->
    <div class="remaining-section" v-if="remainingClusters.length">
      <h2>Other patterns</h2>
      <div
        v-for="c in remainingClusters"
        :key="c.id"
        :class="['review-cluster small', c.status]"
      >
        <div class="cluster-body" @click="router.push(`/cluster/${c.id}`)">
          <div class="cluster-top">
            <span
              class="cluster-category"
              :style="{ color: categoryColors[c.category] || '#888' }"
            >
              {{ categoryLabels[c.category] || c.category }}
            </span>
            <span class="cluster-count">{{ c.log_count }} logs</span>
          </div>
          <h3 class="cluster-title">{{ c.title }}</h3>
        </div>
        <div class="triage-controls">
          <button
            :class="['tri-btn', { active: c.status === 'ignore' }]"
            @click.stop="setStatus(c.id, 'ignore')"
          >
            Ignore
          </button>
          <button
            :class="['tri-btn', { active: c.status === 'watch' }]"
            @click.stop="setStatus(c.id, 'watch')"
          >
            Watch
          </button>
          <button
            :class="['tri-btn fix', { active: c.status === 'worth_fixing' }]"
            @click.stop="setStatus(c.id, 'worth_fixing')"
          >
            Worth Fixing
          </button>
        </div>
      </div>
    </div>

    <!-- Show ignored toggle -->
    <div class="ignored-toggle">
      <button class="toggle-ignored-btn" @click="toggleShowIgnored">
        {{ showIgnored ? "Hide ignored" : "Show ignored patterns" }}
      </button>
    </div>

    <!-- Empty state -->
    <div
      v-if="!topClusters.length && !remainingClusters.length"
      class="empty-state"
    >
      <p>No patterns to review yet.</p>
      <p>
        Log some friction notes and use "Recompute patterns" from the Patterns
        tab.
      </p>
    </div>
  </div>
</template>

<style scoped>
.review-container {
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

.header-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.review-header h1 {
  font-size: 24px;
  font-weight: 600;
}

.cadence-btn {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.date-range {
  font-size: 13px;
  opacity: 0.5;
  margin-top: 2px;
  margin-bottom: 20px;
}

/* Stats grid */
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

/* Category breakdown */
.category-breakdown {
  margin-bottom: 28px;
}

.category-breakdown h2 {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 10px;
}

.category-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.cat-label {
  font-size: 12px;
  font-weight: 500;
  width: 120px;
  flex-shrink: 0;
}

.bar-track {
  flex: 1;
  height: 6px;
  background: rgba(128, 128, 128, 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 3px;
  opacity: 0.7;
}

.cat-count {
  font-size: 12px;
  opacity: 0.5;
  width: 30px;
  text-align: right;
}

/* Sections */
.top-section,
.remaining-section {
  margin-bottom: 24px;
}

.top-section h2,
.remaining-section h2 {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 4px;
}

.section-hint {
  font-size: 13px;
  opacity: 0.45;
  margin-bottom: 12px;
}

/* Review cluster card */
.review-cluster {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 0;
  border-bottom: 1px solid rgba(128, 128, 128, 0.1);
}

.review-cluster:last-child {
  border-bottom: none;
}

.review-cluster.ignore {
  opacity: 0.4;
}

.cluster-rank {
  font-size: 20px;
  font-weight: 700;
  opacity: 0.15;
  width: 28px;
  flex-shrink: 0;
  padding-top: 4px;
}

.cluster-body {
  flex: 1;
  cursor: pointer;
}

.cluster-top {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
}

.cluster-category {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.cluster-count,
.cluster-confidence {
  font-size: 11px;
  opacity: 0.4;
}

.cluster-title {
  font-size: 15px;
  font-weight: 500;
}

.cluster-summary {
  font-size: 13px;
  opacity: 0.5;
  margin-top: 2px;
}

/* Small variant */
.review-cluster.small {
  padding: 10px 0;
}

.review-cluster.small .cluster-title {
  font-size: 14px;
}

/* Triage controls */
.triage-controls {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.tri-btn {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 4px;
  border: 1px solid rgba(128, 128, 128, 0.15);
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.5;
  white-space: nowrap;
}

.tri-btn:hover {
  opacity: 0.8;
}

.tri-btn.active {
  opacity: 1;
  background: rgba(128, 128, 128, 0.1);
  border-color: rgba(128, 128, 128, 0.3);
}

.tri-btn.fix.active {
  background: rgba(76, 175, 80, 0.1);
  border-color: rgba(76, 175, 80, 0.3);
  color: #4caf50;
}

.ignored-toggle {
  text-align: center;
  padding: 8px 0;
}

.toggle-ignored-btn {
  font-size: 12px;
  background: none;
  border: none;
  color: inherit;
  opacity: 0.4;
  cursor: pointer;
}

.toggle-ignored-btn:hover {
  opacity: 0.7;
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
