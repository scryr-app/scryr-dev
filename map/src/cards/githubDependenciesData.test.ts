import { describe, expect, it } from "vitest";
import type { Block } from "@/graphql/generated";
import { getBlockCardData } from "./blockCardData";
import {
	DEPENDENCY_STALE_AFTER_MS,
	githubDependenciesData,
} from "./githubDependenciesData";

const observedAt = "2026-09-17T12:00:00Z";
const now = Date.parse(observedAt) + 300_000;
const repository = "acme/api";
const inventory = {
	observedAt,
	packages: [{ id: "one", name: "widget", version: "1.0", license: "MIT" }],
};
const alert = {
	number: 1,
	package: "widget",
	ecosystem: "npm",
	manifestPath: "package-lock.json",
	severity: "high",
	url: "https://github.com/acme/api/security/dependabot/1",
};
function raw(github: object = {}, sync: object = {}, source: object = {}) {
	return {
		github: { repoUrl: "https://github.com/acme/api.git" },
		dependencies: {
			source: {
				provider: "github",
				inventory: true,
				security: true,
				...source,
			},
			github: { repository, ...github },
		},
		providerSync: sync,
	};
}
function sync(error?: string, repo = repository) {
	return {
		lastAttemptAt: observedAt,
		lastSuccessAt: observedAt,
		error,
		context: { repository: repo },
	};
}

describe("GitHub dependency observations", () => {
	it("only creates native dependency cards for native configuration or snapshots", () => {
		expect(githubDependenciesData(undefined)).toBeUndefined();
		expect(
			githubDependenciesData({ dependencies: { totalDeps: 3, reports: {} } }),
		).toBeUndefined();
		expect(githubDependenciesData(raw(), now)).toMatchObject({
			repository,
			inventory: { state: "unknown" },
			security: { state: "unknown" },
		});
	});
	it("distinguishes a verified empty security snapshot from missing or malformed data", () => {
		expect(
			githubDependenciesData(raw({ security: { observedAt, alerts: [] } }), now)
				?.security,
		).toMatchObject({
			state: "current",
			snapshot: { vulnerablePackages: 0, highestSeverity: "none", alerts: [] },
		});
		for (const security of [
			undefined,
			{},
			{ observedAt, alerts: null },
			{ observedAt: "invalid", alerts: [] },
			{ observedAt, alerts: [{ bad: "payload" }] },
		]) {
			expect(
				githubDependenciesData(raw({ security }), now)?.security,
			).toMatchObject({ state: "unknown", snapshot: undefined });
		}
	});
	it("keeps inventory failure independent of a verified zero-alert scan", () => {
		const result = githubDependenciesData(
			raw(
				{ security: { observedAt, alerts: [] } },
				{
					github_dependencies_inventory: sync(
						"GitHub inventory unavailable (404)",
					),
					github_dependencies_security: sync(),
				},
			),
			now,
		);
		expect(result?.inventory.state).toBe("unknown");
		expect(result?.inventory.sync?.error).toContain("404");
		expect(result?.security.state).toBe("current");
		expect(result?.security.snapshot?.alerts).toHaveLength(0);
	});
	it("keeps inventory data while a security permission failure means unknown", () => {
		const result = githubDependenciesData(
			raw(
				{ inventory },
				{ github_dependencies_security: sync("Requires security permission") },
			),
			now,
		);
		expect(result?.inventory.snapshot?.packages).toHaveLength(1);
		expect(result?.inventory.state).toBe("current");
		expect(result?.security.state).toBe("unknown");
		expect(result?.security.snapshot).toBeUndefined();
	});
	it("marks previously empty security results last-known after a failed poll and recovers on success", () => {
		const snapshots = { inventory, security: { observedAt, alerts: [] } };
		const failed = githubDependenciesData(
			raw(snapshots, { github_dependencies_security: sync("Rate limited") }),
			now,
		);
		expect(failed?.security).toMatchObject({
			state: "unknown",
			stale: true,
			snapshot: { vulnerablePackages: 0 },
		});
		expect(failed?.inventory.state).toBe("current");
		const recovered = githubDependenciesData(
			raw(snapshots, { github_dependencies_security: sync() }),
			now,
		);
		expect(recovered?.security).toMatchObject({
			state: "current",
			stale: false,
		});
	});
	it("marks observations older than two hours stale even without a reported failure", () => {
		const result = githubDependenciesData(
			raw({ inventory, security: { observedAt, alerts: [] } }),
			Date.parse(observedAt) + DEPENDENCY_STALE_AFTER_MS + 1,
		);
		expect(result?.inventory).toMatchObject({ state: "unknown", stale: true });
		expect(result?.security).toMatchObject({ state: "unknown", stale: true });
	});
	it("groups vulnerable packages by ecosystem and name, counts alerts separately, and sorts severity", () => {
		const alerts = [
			alert,
			{ ...alert, number: 2 },
			{ ...alert, number: 3, ecosystem: "pip", severity: "critical" },
			{ ...alert, number: 4, package: "other", severity: "low" },
			{ ...alert, number: 5, severity: "medium" },
		];
		const result = githubDependenciesData(
			raw({ security: { observedAt, alerts } }),
			now,
		)?.security.snapshot;
		expect(result?.vulnerablePackages).toBe(3);
		expect(result?.alerts).toHaveLength(5);
		expect(result?.highestSeverity).toBe("critical");
		expect(result?.severityCounts).toEqual({
			critical: 1,
			high: 2,
			medium: 1,
			low: 1,
		});
		expect(result?.alerts[0].number).toBe(3);
	});
	it("does not double-count duplicate provider alert IDs", () => {
		const result = githubDependenciesData(
			raw({ security: { observedAt, alerts: [alert, alert] } }),
			now,
		)?.security.snapshot;
		expect(result?.alerts).toHaveLength(1);
		expect(result?.severityCounts.high).toBe(1);
	});
	it("does not convert an unknown severity into a clean scan", () => {
		expect(
			githubDependenciesData(
				raw({
					security: {
						observedAt,
						alerts: [{ ...alert, severity: "unexpected" }],
					},
				}),
				now,
			)?.security.state,
		).toBe("unknown");
	});
	it("removes old snapshots and statuses when the declared repository changes", () => {
		const input = raw(
			{ inventory, security: { observedAt, alerts: [alert] } },
			{ github_dependencies_security: sync("Old repository error") },
		);
		input.github.repoUrl = "https://github.com/acme/new-api";
		const result = githubDependenciesData(input, now);
		expect(result?.repository).toBe("acme/new-api");
		expect(result?.inventory.snapshot).toBeUndefined();
		expect(result?.security.snapshot).toBeUndefined();
		expect(result?.security.sync).toBeUndefined();
	});
	it("matches GitHub repository names case-insensitively but rejects invalid URLs", () => {
		const input = raw({ inventory });
		input.github.repoUrl = "https://github.com/ACME/API/";
		expect(githubDependenciesData(input, now)?.inventory.state).toBe("current");
		input.github.repoUrl = "not a repository";
		expect(
			githubDependenciesData(input, now)?.inventory.snapshot,
		).toBeUndefined();
	});
	it("hides stale observations and errors for explicitly disabled parts", () => {
		const result = githubDependenciesData(
			raw(
				{ inventory, security: { observedAt, alerts: [alert] } },
				{ github_dependencies_security: sync("offline") },
				{ inventory: false, security: false },
			),
			now,
		);
		expect(result?.inventory).toEqual({ state: "disabled", stale: false });
		expect(result?.security).toEqual({ state: "disabled", stale: false });
	});
	it("enables both parts by default and supports standalone native snapshots", () => {
		const input = raw({ inventory });
		input.dependencies.source = {
			provider: "github",
		} as typeof input.dependencies.source;
		expect(githubDependenciesData(input, now)?.security.state).toBe("unknown");
		expect(
			githubDependenciesData(
				{ dependencies: { github: { repository, inventory } } },
				now,
			)?.inventory.state,
		).toBe("current");
	});
	it("preserves explicit dependency counts without inventing outdated counts or compliance", () => {
		const result = githubDependenciesData(
			raw({ inventory: { ...inventory, directDeps: 0, transitiveDeps: 1 } }),
			now,
		)?.inventory.snapshot;
		expect(result).toMatchObject({ directDeps: 0, transitiveDeps: 1 });
		expect(result).not.toHaveProperty("outdatedDeps");
		expect(result).not.toHaveProperty("licenseCompliance");
		expect(
			githubDependenciesData(raw({ inventory }), now)?.inventory.snapshot
				?.directDeps,
		).toBeUndefined();
	});
	it.each([
		"javascript:alert(1)",
		"data:text/html,malicious",
		"//evil.example/path",
		"https://user:password@evil.example",
	])("rejects unsafe remediation URL %s while retaining the alert", (url) => {
		const result = githubDependenciesData(
			raw({ security: { observedAt, alerts: [{ ...alert, url }] } }),
			now,
		)?.security.snapshot;
		expect(result?.alerts).toHaveLength(1);
		expect(result?.alerts[0].url).toBeUndefined();
	});
	it("wires native dependency configuration through block raw JSON", () => {
		const block = {
			links: [],
			connections: [],
			frameworks: [],
			rawJsonString: JSON.stringify(raw()),
		} as unknown as Block;
		expect(getBlockCardData(block).githubDependencies?.security.state).toBe(
			"unknown",
		);
	});
});
