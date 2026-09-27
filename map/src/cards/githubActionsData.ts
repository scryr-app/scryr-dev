export interface WorkflowStatus {
	key: string;
	name: string;
	branch?: string;
	status: string;
	outcome: "passing" | "failing" | "pending" | "neutral";
	url?: string;
}

export interface GithubActionsCardData {
	workflows: WorkflowStatus[];
	sync?: {
		lastAttemptAt?: string;
		lastSuccessAt?: string;
		error?: string;
	};
}

type RecordValue = Record<string, unknown>;

function record(value: unknown): RecordValue | undefined {
	return value && typeof value === "object" && !Array.isArray(value)
		? (value as RecordValue)
		: undefined;
}

function string(value: unknown): string | undefined {
	return typeof value === "string" && value.length > 0 ? value : undefined;
}

function timestamp(value: unknown): string | undefined {
	const text = string(value);
	return text && Number.isFinite(Date.parse(text)) ? text : undefined;
}

function identity(value: unknown): value is number {
	return typeof value === "number" && Number.isSafeInteger(value) && value > 0;
}

function outcome(
	status: string,
	conclusion: unknown,
): WorkflowStatus["outcome"] {
	if (status !== "completed") return "pending";
	if (conclusion === "success") return "passing";
	if (
		["failure", "timed_out", "action_required", "startup_failure"].includes(
			String(conclusion),
		)
	)
		return "failing";
	return "neutral";
}

/** Read provider observations without inventing build results for collection errors. */
export function githubActionsData(
	raw: RecordValue | undefined,
): GithubActionsCardData | undefined {
	const cicd = record(raw?.cicd);
	const source = record(cicd?.source);
	const sync = record(record(raw?.providerSync)?.github);
	const runs = record(cicd?.githubActions)?.runs;
	const configured = Boolean(source && cicd?.platform === "github_actions");
	const latest = new Map<string, { run: RecordValue; order: number[] }>();
	if (Array.isArray(runs)) {
		for (const value of runs) {
			const run = record(value);
			if (
				!run ||
				!string(run.host) ||
				!string(run.repository) ||
				!identity(run.repositoryId) ||
				!identity(run.workflowId) ||
				!identity(run.runId) ||
				!identity(run.runAttempt) ||
				!string(run.status) ||
				!timestamp(run.createdAt) ||
				!timestamp(run.updatedAt)
			)
				continue;
			if (string(source?.branch) && run.headBranch !== source?.branch) continue;
			const workflowId = source?.workflow_id ?? source?.workflowId;
			if (identity(workflowId) && run.workflowId !== workflowId) continue;
			const selectedWorkflows = Array.isArray(source?.workflows)
				? source.workflows.filter(
						(name): name is string => typeof name === "string",
					)
				: [];
			if (
				selectedWorkflows.length > 0 &&
				!selectedWorkflows.some(
					(name) => run.workflowPath === `.github/workflows/${name}`,
				)
			)
				continue;
			const key = JSON.stringify([
				run.host,
				run.repositoryId,
				run.workflowId,
				run.headBranch ?? null,
			]);
			// A newly observed old run must not hide a newer run. Attempts break ties within a run.
			const order = [
				Date.parse(String(run.createdAt)),
				run.runId,
				run.runAttempt,
				Date.parse(String(run.updatedAt)),
			];
			const previous = latest.get(key);
			const differing = previous
				? order.findIndex((part, i) => part !== previous.order[i])
				: -1;
			if (
				!previous ||
				(differing >= 0 && order[differing] > previous.order[differing])
			)
				latest.set(key, { run, order });
		}
	}
	if (!configured && !sync && latest.size === 0) return undefined;
	return {
		workflows: Array.from(latest, ([key, { run }]) => ({
			key,
			name:
				string(run.workflowName) ??
				string(run.workflowPath)?.split("/").pop() ??
				`Workflow ${run.workflowId}`,
			branch: string(run.headBranch),
			status: String(
				run.status === "completed" ? (run.conclusion ?? "unknown") : run.status,
			).replaceAll("_", " "),
			outcome: outcome(String(run.status), run.conclusion),
			url:
				typeof run.htmlUrl === "string" && /^https?:\/\//.test(run.htmlUrl)
					? run.htmlUrl
					: undefined,
		})).sort(
			(a, b) => a.name.localeCompare(b.name) || a.key.localeCompare(b.key),
		),
		sync: sync
			? {
					lastAttemptAt: timestamp(sync.lastAttemptAt),
					lastSuccessAt: timestamp(sync.lastSuccessAt),
					error: string(sync.error),
				}
			: undefined,
	};
}
