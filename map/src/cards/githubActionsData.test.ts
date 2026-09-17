import { describe, expect, it } from "vitest";
import { githubActionsData } from "./githubActionsData";

function run(overrides: Record<string, unknown> = {}) {
	return {
		host: "github.com",
		repositoryId: 1,
		repository: "acme/api",
		workflowId: 10,
		workflowName: "CI",
		runId: 100,
		runAttempt: 1,
		headBranch: "main",
		status: "completed",
		conclusion: "success",
		createdAt: "2026-09-17T12:00:00Z",
		updatedAt: "2026-09-17T12:01:00Z",
		htmlUrl: "https://github.com/acme/api/actions/runs/100",
		...overrides,
	};
}

function data(runs: unknown[], source?: Record<string, unknown>) {
	return githubActionsData({
		cicd: { platform: "github_actions", source, githubActions: { runs } },
	});
}

describe("GitHub workflow card observations", () => {
	it("does not create cards for missing or malformed observations", () => {
		expect(githubActionsData(undefined)).toBeUndefined();
		expect(
			githubActionsData({ github: { repoUrl: "https://github.com/acme/api" } }),
		).toBeUndefined();
		expect(
			data([
				null,
				{ runId: 1 },
				run({ createdAt: "invalid" }),
				run({ workflowId: -1 }),
			]),
		).toBeUndefined();
	});
	it("distinguishes configured but unobserved workflows from absent configuration", () => {
		expect(data([], { workflows: ["ci.yml"] })).toEqual({
			workflows: [],
			sync: undefined,
		});
		expect(data([])).toBeUndefined();
	});
	it("keeps latest results separately for each workflow and branch", () => {
		const result = data([
			run({
				workflowId: 20,
				workflowName: "Integration",
				conclusion: "failure",
			}),
			run({ headBranch: "feature", conclusion: "failure" }),
			run(),
		]);
		expect(result?.workflows.map((w) => [w.name, w.branch, w.outcome])).toEqual(
			[
				["CI", "feature", "failing"],
				["CI", "main", "passing"],
				["Integration", "main", "failing"],
			],
		);
	});
	it("separates repositories and enterprise hosts sharing workflow identifiers", () => {
		expect(
			data([run(), run({ repositoryId: 2 }), run({ host: "github.acme.com" })])
				?.workflows,
		).toHaveLength(3);
	});
	it("chooses the newest run rather than a late update to older history", () => {
		const oldRun = run({
			runId: 90,
			createdAt: "2026-09-16T12:00:00Z",
			updatedAt: "2026-09-18T12:00:00Z",
			conclusion: "failure",
		});
		for (const runs of [
			[run(), oldRun],
			[oldRun, run()],
		]) {
			expect(data(runs)?.workflows[0].outcome).toBe("passing");
		}
	});
	it("chooses the latest rerun attempt and most recent observation regardless of input order", () => {
		const queued = run({ runAttempt: 2, status: "queued", conclusion: null });
		const running = run({
			runAttempt: 2,
			status: "in_progress",
			conclusion: null,
			updatedAt: "2026-09-17T12:02:00Z",
		});
		for (const runs of [
			[running, run(), queued],
			[queued, run(), running],
		]) {
			expect(data(runs)?.workflows[0]).toMatchObject({
				status: "in progress",
				outcome: "pending",
			});
		}
	});
	it("filters persisted history by selected branch and numeric workflow ID", () => {
		const runs = [
			run(),
			run({ headBranch: "feature" }),
			run({ workflowId: 20 }),
		];
		expect(
			data(runs, { branch: "main", workflow_id: 10 })?.workflows,
		).toHaveLength(1);
		expect(
			data(runs, { branch: "main", workflowId: 10 })?.workflows,
		).toHaveLength(1);
		expect(data(runs, { branch: "unobserved" })?.workflows).toEqual([]);
	});
	it.each([
		["success", "passing"],
		["failure", "failing"],
		["timed_out", "failing"],
		["action_required", "failing"],
		["startup_failure", "failing"],
		["cancelled", "neutral"],
		["skipped", "neutral"],
		["neutral", "neutral"],
		[null, "neutral"],
	])(
		"represents completed %s without reporting false success",
		(conclusion, expected) => {
			expect(data([run({ conclusion })])?.workflows[0].outcome).toBe(expected);
		},
	);
	it("rejects unsafe workflow links", () => {
		expect(
			data([run({ htmlUrl: "javascript:alert(1)" })])?.workflows[0].url,
		).toBeUndefined();
	});
	it("preserves the last successful sync and workflow outcomes during collection failures", () => {
		const result = githubActionsData({
			cicd: { githubActions: { runs: [run()] } },
			providerSync: {
				github: {
					lastAttemptAt: "2026-09-17T12:10:00Z",
					lastSuccessAt: "2026-09-17T12:05:00Z",
					error: "gh authentication expired",
				},
			},
		});
		expect(result?.workflows[0].outcome).toBe("passing");
		expect(result?.sync).toEqual({
			lastAttemptAt: "2026-09-17T12:10:00Z",
			lastSuccessAt: "2026-09-17T12:05:00Z",
			error: "gh authentication expired",
		});
	});
	it("shows first-sync errors without creating workflow results or invalid dates", () => {
		expect(
			githubActionsData({
				providerSync: {
					github: {
						lastAttemptAt: "invalid",
						lastSuccessAt: null,
						error: "gh not found",
					},
				},
			}),
		).toEqual({
			workflows: [],
			sync: {
				lastAttemptAt: undefined,
				lastSuccessAt: undefined,
				error: "gh not found",
			},
		});
	});
	it("filters filename selections by the exact workflow path", () => {
		const runs = [
			run({ workflowPath: ".github/workflows/ci.yml" }),
			run({
				workflowId: 20,
				workflowPath: ".github/workflows/integration.yaml",
				conclusion: "failure",
			}),
			run({ workflowId: 30, workflowPath: ".github/workflows/old.yml" }),
			run({ workflowId: 40, workflowPath: "ci.yml" }),
			run({ workflowId: 50, workflowPath: "other/ci.yml" }),
			run({ workflowId: 60 }),
			run({ workflowPath: ".github/workflows/ci.yml", headBranch: "feature" }),
		];
		const selected = data(runs, {
			workflows: ["ci.yml", "integration.yaml"],
			branch: "main",
		});
		expect(selected?.workflows).toHaveLength(2);
		expect(selected?.workflows.map((workflow) => workflow.outcome)).toEqual([
			"passing",
			"failing",
		]);
		expect(data(runs, { workflows: ["unobserved.yml"] })?.workflows).toEqual(
			[],
		);
	});
	it("retains pathless legacy history only when no filename selector is active", () => {
		expect(data([run()], { workflows: ["ci.yml"] })?.workflows).toEqual([]);
		expect(data([run()], { workflows: [] })?.workflows).toHaveLength(1);
		expect(data([run()])?.workflows).toHaveLength(1);
	});
	it("labels valid runs with missing display names using their path or workflow ID", () => {
		expect(
			data([
				run({ workflowName: "", workflowPath: ".github/workflows/ci.yml" }),
			])?.workflows[0].name,
		).toBe("ci.yml");
		expect(data([run({ workflowName: "" })])?.workflows[0].name).toBe(
			"Workflow 10",
		);
	});
});
