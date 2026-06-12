import { defineConfig, devices } from "@playwright/test";

// gui-scrutiny runs chromium against an already-running static server
// (python3 -m http.server -d . 8011). Override the base URL with PG_URL.
export default defineConfig({
  testMatch: "gui-scrutiny.spec.mjs",
  fullyParallel: false,
  workers: 1,
  reporter: [["list"]],
  use: {
    baseURL: process.env.PG_URL || "http://localhost:8011/",
    ...devices["Desktop Chrome"],
  },
});
