import { defineConfig } from "@playwright/test";
export default defineConfig({
    testDir: "./e2e",
    testMatch: "*.e2e.ts",
    timeout: 180_000,
    workers: 1,
    use: {
        channel: "chrome",
        viewport: { width: 1600, height: 1000 },
        launchOptions: { args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"] },
        screenshot: "only-on-failure",
        trace: "retain-on-failure",
    },
});

