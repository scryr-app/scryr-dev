/** Category slots retain their identity as cards are added and reordered. */
export const integrationCategories = [
	{ id: "repository", label: "Repository", providers: ["GitHub"] },
	{ id: "metrics", label: "Metrics", providers: ["Grafana"] },
	{ id: "cicd", label: "CI/CD", providers: ["GitHub Actions"] },
	{
		id: "tests",
		label: "Tests",
		providers: ["JUnit reports", "Coverage reports"],
	},
	{
		id: "dependencies",
		label: "Dependencies",
		providers: ["Dependency reports"],
	},
	{
		id: "performance",
		label: "Performance",
		providers: ["Grafana", "PostHog"],
	},
	{ id: "uptime", label: "Uptime", providers: ["Grafana"] },
	{ id: "analytics", label: "Analytics", providers: ["PostHog"] },
] as const;
export type CardCategory = (typeof integrationCategories)[number]["id"];
export type ViewKind =
	| "grafana_performance"
	| "grafana_metrics"
	| "grafana_uptime"
	| "posthog_performance"
	| "posthog_analytics"
	| "github_actions_pipeline";
export const viewCategories: Record<ViewKind, CardCategory> = {
	grafana_performance: "performance",
	grafana_metrics: "metrics",
	grafana_uptime: "uptime",
	posthog_performance: "performance",
	posthog_analytics: "analytics",
	github_actions_pipeline: "cicd",
};
export interface IntegrationView {
	id: string;
	kind: ViewKind;
	category: CardCategory;
	title: string;
	source?: { queries: Record<string, string>; environment?: string } | null;
	integration: { id: string; kind: string; repository?: string };
	workflowId?: number | null;
	branch?: string | null;
	observations?: { runs: ActionRun[] } | null;
}
export interface ActionRun {
	repository: string;
	workflowId: number;
	headBranch?: string;
	status: string;
	conclusion?: string;
	updatedAt: string;
	workflowName?: string;
}
export function isIntegrationView(value: unknown): value is IntegrationView {
	if (!value || typeof value !== "object") return false;
	const c = value as Partial<IntegrationView>;
	return (
		typeof c.id === "string" &&
		c.id.length > 0 &&
		typeof c.title === "string" &&
		c.kind !== undefined &&
		Object.hasOwn(viewCategories, c.kind) &&
		c.category === viewCategories[c.kind] &&
		!!c.integration
	);
}
export function openIntegrationSetup(category: CardCategory) {
	window.dispatchEvent(
		new CustomEvent("scryr:integration-setup", { detail: category }),
	);
}
