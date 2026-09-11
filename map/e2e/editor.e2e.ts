import { test, expect, type Page, type APIRequestContext } from "@playwright/test";
import { spawn, type ChildProcess } from "node:child_process";
import { mkdtemp, mkdir, copyFile, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "node:net";

const root = fileURLToPath(new URL("../../", import.meta.url));
let directory: string;
let server: ChildProcess;
let base: string;
let logs: string;
let initial: string;
let binary: string;
const environment = () => {
    const env = { ...process.env, SCRYR_SQLITE_PATH: resolve(directory, "scryr.db"), AUTH_MODE: "local" };
    for (const key of ["DATABASE_URL", "TURSO_DATABASE_URL", "TURSO_AUTH_TOKEN", "CLERK_SECRET_KEY", "SCRYR_TOKEN", "SCRYR_API_TOKEN", "SCRYR_ENDPOINT", "SCRYR_GRAPHQL_URL", "SCRYR_CLERK_ORG_ID"]) delete env[key as keyof typeof env];
    return env;
};
async function port() {
    const listener = createServer();
    await new Promise<void>(resolve => listener.listen(0, "127.0.0.1", resolve));
    const address = listener.address();
    if (!address || typeof address === "string") throw new Error("No port");
    await new Promise<void>((resolve, reject) => listener.close(error => error ? reject(error) : resolve()));
    return address.port;
}
async function launch(request: APIRequestContext, serverOnly = false, watch = false, entrypoint = "index.scry") {
    const bindPort = await port();
    base = `http://127.0.0.1:${bindPort}`;
    logs = "";
    server = spawn(binary, ["serve", "--host", "127.0.0.1", "--port", String(bindPort),
        ...(serverOnly ? ["--server-only"] : ["--no-open", "--path", entrypoint, ...(watch ? ["--watch"] : [])])],
        { cwd: directory, env: environment(), stdio: ["ignore", "pipe", "pipe"] });
    server.stdout?.on("data", data => logs += data);
    server.stderr?.on("data", data => logs += data);
    await expect.poll(async () => {
        try { return (await request.get(`${base}/ready`)).status(); } catch { return 0; }
    }, { timeout: 60_000, message: "standalone server starts" }).toBe(200);
    await expect.poll(async () => {
        const result = await gql(request, "{ scryrMaps { scryIdentifier } }");
        return result.data?.scryrMaps?.length ?? 0;
    }, { timeout: 90_000, message: "serve publishes the initial file" }).toBeGreaterThan(0);
}
async function stop() {
    if (!server || server.exitCode !== null) return;
    await new Promise<void>(resolve => { server.once("exit", () => resolve()); server.kill("SIGTERM"); });
}
async function gql(request: APIRequestContext, query: string, variables = {}) {
    return (await request.post(`${base}/graphql`, {
        headers: { origin: base }, data: { query, variables },
    })).json();
}
async function openEditor(page: Page) {
    await page.goto(base);
    await page.getByRole("button", { name: "Show editor", exact: true }).click();
    await expect(page.locator(".cm-content")).toContainText("from scryr");
}
async function run(page: Page, code: string, location: "disk" | "cloud") {
    await page.locator(".cm-content").fill(code);
    await page.getByRole("button", { name: "Save and Run", exact: true }).click();
    await expect(page.getByRole("status").filter({ hasText: /Saved to .*Diagram updated|Run failed/ })).toBeVisible({ timeout: 120_000 });
    expect(await page.getByRole("log").innerText()).not.toContain("Error:");
    await expect(page.getByRole("status").filter({ hasText: `Saved to ${location} · Diagram updated` })).toBeVisible();
}
test.beforeEach(async ({ page }) => {
    page.on("dialog", dialog => dialog.accept());
    directory = await mkdtemp(resolve(tmpdir(), "scryr-editor-e2e-"));
    binary = resolve(directory, process.platform === "win32" ? "scryr.exe" : "scryr");
    await copyFile(resolve(root, "crystal/target/release/scryr"), binary);
    initial = await readFile(resolve(root, "manifest/tests/samples/mern/index.scry"), "utf8");
    await writeFile(resolve(directory, "index.scry"), initial);
});
test.afterEach(async ({}, info) => {
    await stop();
    if (info.status !== info.expectedStatus) await info.attach("server.log", { body: logs, contentType: "text/plain" });
    await rm(directory, { recursive: true, force: true });
});
test("standalone serve edits disk, refreshes fresh blocks, survives restart, and protects invalid/conflicting drafts", async ({ page, request }) => {
    await launch(request);
    await openEditor(page);
    const changed = initial.replaceAll("React it all yo", "Console Web");
    await run(page, changed, "disk");
    expect(await readFile(resolve(directory, "index.scry"), "utf8")).toContain("Console Web");
    const blocks = await gql(request, '{ blocks { name } }');
    expect(blocks.errors).toBeUndefined();
    expect(blocks.data.blocks.some((block: {name: string}) => block.name === "Console Web")).toBe(true);
    await run(page, changed.replaceAll("Console Web", "Console Web Again"), "disk");
    await page.locator(".cm-content").fill(`${changed}\ndef invalid_native() -> None:\n    unused = 1\n`);
    await page.getByRole("button", { name: "Save and Run", exact: true }).click();
    await expect(page.getByRole("status").filter({ hasText: "Run failed" })).toBeVisible({ timeout: 120_000 });
    await expect(page.getByRole("log")).toContainText("F841");
    expect(await readFile(resolve(directory, "index.scry"), "utf8")).toContain("Console Web Again");
    await page.locator(".cm-content").fill("while True:\n    pass\n");
    await page.getByRole("button", { name: "Save and Run", exact: true }).click();
    await page.getByRole("button", { name: "Cancel run", exact: true }).click();
    await expect(page.getByRole("log")).toContainText("Execution cancelled");
    await page.locator(".cm-content").fill("this is invalid Python !!!");
    await page.getByRole("button", { name: "Save and Run", exact: true }).click();
    await expect(page.getByRole("status").filter({ hasText: "Run failed" })).toBeVisible({ timeout: 120_000 });
    expect(await readFile(resolve(directory, "index.scry"), "utf8")).toContain("Console Web Again");
    await writeFile(resolve(directory, "index.scry"), changed.replaceAll("Console Web", "External Web"));
    await expect(page.getByRole("alert")).toContainText("Source changed elsewhere");
    await expect(page.getByRole("button", { name: "Save and Run", exact: true })).toBeDisabled();
    await stop();
    await launch(request);
    await page.goto(base);
    await openEditor(page);
    await expect(page.locator(".cm-content")).toContainText("External Web");
    await expect(page.locator("canvas").first()).toBeVisible();
    await page.screenshot({ path: test.info().outputPath("standalone-editor.png") });
});
test("stored-source editing through server-only saves all source without touching disk", async ({ page, request }) => {
    await writeFile(resolve(directory, "labels.scry"), 'SUFFIX = ""\n');
    initial = `from labels import SUFFIX\n${initial.replace('"React it all yo"', '"React it all yo" + SUFFIX')}`;
    await writeFile(resolve(directory, "index.scry"), initial);
    await launch(request); // seed a source snapshot using the real CLI
    await stop();
    const diskBefore = await readFile(resolve(directory, "index.scry"), "utf8");
    await launch(request, true);
    await openEditor(page);
    await run(page, initial.replaceAll("React it all yo", "Stored Web"), "cloud");
    expect(await readFile(resolve(directory, "index.scry"), "utf8")).toBe(diskBefore);
    await page.reload();
    await page.getByRole("button", { name: "Show editor", exact: true }).click();
    await expect(page.locator(".cm-content")).toContainText("Stored Web");
    const denied = await request.post(`${base}/graphql`, {
        headers: { origin: "https://untrusted.example" },
        data: { query: 'query { manifestDocument(identifier: "mern_diagram") }' },
    });
    expect((await denied.json()).errors).toBeTruthy();
});
test("serve watch reloads external source and preserves a dirty browser draft", async ({ page, request }) => {
    await launch(request, false, true);
    await openEditor(page);
    await writeFile(resolve(directory, "index.scry"), initial.replaceAll("React it all yo", "Watched Web"));
    await expect(page.locator(".cm-content")).toContainText("Watched Web", { timeout: 60_000 });
    await run(page, initial.replaceAll("React it all yo", "Browser Web"), "disk");
    await page.waitForTimeout(2000);
    expect(await readFile(resolve(directory, "index.scry"), "utf8")).toContain("Browser Web");
});

test("nested entrypoint preserves project tooling configuration and only writes the registered file", async ({ page, request }) => {
    await mkdir(resolve(directory, "services"));
    await writeFile(resolve(directory, "services/index.scry"), initial);
    await writeFile(resolve(directory, "pyproject.toml"), '[tool.ruff]\nline-length = 120\n');
    await launch(request, false, false, "services/index.scry");
    await openEditor(page);
    await run(page, initial.replaceAll("React it all yo", "Nested Web"), "disk");
    expect(await readFile(resolve(directory, "services/index.scry"), "utf8")).toContain("Nested Web");
    expect(await readFile(resolve(directory, "index.scry"), "utf8")).toBe(initial);
    const source = await gql(request, '{ manifestDocument(identifier: "mern_diagram") }');
    expect(source.data.manifestDocument.folderPath).toBe("services");
    expect(source.data.manifestDocument.local).toBe(true);
});

test("console follows tray brightness and keeps source controls in the bottom toolbar", async ({ page, request }) => {
    await launch(request);
    await openEditor(page);
    const panel = page.getByRole("region", { name: "Manifest source editor" });
    const footer = panel.locator("footer");
    await expect(panel.getByRole("button", { name: "Save and Run", exact: true })).toBeVisible();
    await expect(footer.getByRole("checkbox", { name: "Follow selected block" })).toBeVisible();
    await expect(footer.getByRole("button", { name: "Reload source" })).toBeVisible();
    await expect(panel.getByText(/Run saves/)).toHaveCount(0);
    await expect(page.locator(".cm-editor")).toHaveCSS("color", "rgb(15, 23, 42)");
    const palette = page.locator("button").filter({ has: page.locator("svg.lucide-palette") });
    const tray = palette.locator("../..");
    expect(await panel.evaluate(el => getComputedStyle(el).backgroundColor)).toBe(await tray.evaluate(el => getComputedStyle(el).backgroundColor));
    await page.screenshot({ path: test.info().outputPath("console-light.png") });
    await palette.click();
    await Promise.all([
        page.waitForEvent("load"),
        page.getByRole("switch", { name: "Switch to dark mode" }).click(),
    ]);
    await page.getByRole("button", { name: "Show editor", exact: true }).click();
    await expect(page.locator(".cm-editor")).toHaveCSS("color", "rgb(226, 232, 240)");
    await page.screenshot({ path: test.info().outputPath("console-dark.png") });
});
