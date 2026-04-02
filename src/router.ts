import { createRouter, createWebHistory } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import LogsView from "./views/LogsView.vue";
import CaptureView from "./views/CaptureView.vue";
import SettingsView from "./views/SettingsView.vue";
import OnboardingView from "./views/OnboardingView.vue";
import ClusterDetailView from "./views/ClusterDetailView.vue";
import ReviewView from "./views/ReviewView.vue";
import BriefView from "./views/BriefView.vue";
import WinsView from "./views/WinsView.vue";
import InvestigateView from "./views/InvestigateView.vue";

const routes = [
  { path: "/", redirect: "/logs" },
  { path: "/logs", component: LogsView },
  { path: "/capture", component: CaptureView },
  { path: "/settings", component: SettingsView },
  { path: "/onboarding", component: OnboardingView },
  { path: "/cluster/:id", component: ClusterDetailView },
  { path: "/review", component: ReviewView },
  { path: "/brief/:id", component: BriefView },
  { path: "/wins", component: WinsView },
  { path: "/investigate", component: InvestigateView },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});

// Redirect to onboarding on first launch (only for main window, not capture)
router.beforeEach(async (to) => {
  if (to.path === "/capture" || to.path === "/onboarding") return;

  try {
    const settings = await invoke<{ onboarding_completed: boolean }>(
      "get_settings"
    );
    if (!settings.onboarding_completed) {
      return "/onboarding";
    }
  } catch {
    // If settings can't be read, proceed normally
  }
});
