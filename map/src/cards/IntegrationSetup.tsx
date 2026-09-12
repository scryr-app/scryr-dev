import { Container, Text } from "@react-three/uikit";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { currentTheme } from "@/theme/theme";
import {
	type CardCategory,
	integrationCategories,
	openIntegrationSetup,
} from "./integrationCatalog";

export function SetupLink({ category }: { category: CardCategory }) {
	const definition = integrationCategories.find((c) => c.id === category);
	return (
		<Container flexDirection="column" gap={3}>
			<Container
				cursor="pointer"
				onClick={(event) => {
					event.stopPropagation();
					openIntegrationSetup(category);
				}}
			>
				<Text fontSize={11} color="#60a5fa">
					Please Set Up
				</Text>
			</Container>
			<Text fontSize={9} color={currentTheme.cardTextColor}>
				{definition?.providers.join(" · ") ?? ""}
			</Text>
		</Container>
	);
}
export function SetupCard({ category }: { category: CardCategory }) {
	const definition = integrationCategories.find((c) => c.id === category);
	return (
		<Container
			sizeX={2.8}
			sizeY={1.8}
			pixelSize={0.01}
			flexDirection="column"
			padding={12}
			gap={10}
		>
			<Text fontSize={16} color={currentTheme.cardTextColor}>
				{definition?.label ?? category}
			</Text>
			<Text fontSize={11} color={currentTheme.cardTextColor}>
				No integration or data configured.
			</Text>
			<SetupLink category={category} />
		</Container>
	);
}
const instructions: Record<string, string> = {
	Grafana: `grafana_authentication = GrafanaAuthentication()
grafana = Grafana(authentication=grafana_authentication, endpoint="https://your-prometheus-endpoint")
grafana_performance = GrafanaPerformance(
    integration=grafana,
    queries=GrafanaPerformanceQueries(
        cpu_current=PrometheusQuery(expression="YOUR_PROMQL", unit=MetricUnit.percent),
    ),
)
# Include grafana in Manifest.integrations and grafana_performance in Manifest.cards.
# Credentials: [authentication.grafana_authentication] with username and token.`,
	PostHog: `posthog_authentication = PostHogAuthentication()
posthog = PostHog(authentication=posthog_authentication, project_id=123)
posthog_performance = PostHogPerformance(
    integration=posthog,
    queries=PostHogPerformanceQueries(
        response_time_p95=PostHogQuery(
            expression="YOUR_HOGQL_WITH_{start}_{end}_{environment}",
            unit=MetricUnit.milliseconds,
        ),
    ),
)
# Include posthog in Manifest.integrations and posthog_performance in Manifest.cards.
# Credentials: [authentication.posthog_authentication] with api_key.`,
	"GitHub Actions": `github_actions = GitHubActions(repository="owner/repository")
github_pipeline = GitHubActionsPipeline(integration=github_actions, branch="main")
# Include github_actions in Manifest.integrations and github_pipeline in Manifest.cards.
# Send workflow observations with:
scryr report actions --manifest api --card github_pipeline --event-file workflow-event.json`,
	GitHub: `# Declare repository metadata in index.scry:
github=Github(repo_url="https://github.com/owner/repository")`,
	"JUnit reports":
		'Declare Tests(source=TestReportSource(files=["junit.xml"])) in index.scry, then run scryr report tests --manifest api.',
	"Coverage reports":
		"Run scryr report coverage --help to select your coverage artifact and manifest.",
	"Dependency reports":
		"Run scryr report dependencies --help to select your dependency report and manifest.",
};
function setupInstructions(provider: string, category: CardCategory): string {
	const source = instructions[provider] ?? "";
	if (provider === "Grafana" && category === "uptime")
		return source
			.replaceAll("GrafanaPerformance", "GrafanaUptime")
			.replaceAll("grafana_performance", "grafana_uptime")
			.replace("cpu_current", "uptime")
			.replace("MetricUnit.percent", "MetricUnit.ratio");
	if (provider === "Grafana" && category === "metrics")
		return source
			.replaceAll("GrafanaPerformance", "GrafanaMetrics")
			.replaceAll("grafana_performance", "grafana_metrics")
			.replace("cpu_current", "request_rate")
			.replace("MetricUnit.percent", "MetricUnit.requests_per_second");
	if (provider === "PostHog" && category === "analytics")
		return source
			.replaceAll("PostHogPerformance", "PostHogAnalytics")
			.replaceAll("posthog_performance", "posthog_analytics")
			.replace("response_time_p95", "page_views")
			.replace("MetricUnit.milliseconds", "MetricUnit.count");
	return source;
}
export function IntegrationSetupDialog() {
	const [category, setCategory] = useState<CardCategory | null>(null);
	const closeButton = useRef<HTMLButtonElement>(null);
	useEffect(() => {
		if (!category) return;
		const previous = document.activeElement;
		closeButton.current?.focus();
		return () => {
			if (previous instanceof HTMLElement) previous.focus();
		};
	}, [category]);
	useEffect(() => {
		const open = (event: Event) => {
			const id: unknown = (event as CustomEvent).detail;
			const definition = integrationCategories.find((c) => c.id === id);
			if (definition) setCategory(definition.id);
		};
		window.addEventListener("scryr:integration-setup", open);
		return () => window.removeEventListener("scryr:integration-setup", open);
	}, []);
	useEffect(() => {
		if (!category) return;
		const close = (event: KeyboardEvent) => {
			if (event.key === "Escape") setCategory(null);
		};
		window.addEventListener("keydown", close);
		return () => window.removeEventListener("keydown", close);
	}, [category]);
	const definition = integrationCategories.find((c) => c.id === category);
	if (!definition) return null;
	return createPortal(
		<div className="fixed inset-0 z-[2000] flex items-center justify-center bg-black/60 p-6">
			<div
				onKeyDown={(event) => {
					event.stopPropagation();
					if (event.key === "Escape") setCategory(null);
					if (event.key === "Tab") {
						event.preventDefault();
						closeButton.current?.focus();
					}
				}}
				role="dialog"
				aria-modal="true"
				aria-labelledby="integration-setup-title"
				className="max-h-[85vh] w-full max-w-3xl overflow-auto rounded-xl border border-white/20 bg-slate-900 p-6 text-white shadow-xl"
			>
				<div className="flex justify-between gap-4">
					<h2 id="integration-setup-title" className="text-xl font-semibold">
						Set up {definition.label}
					</h2>
					<button
						ref={closeButton}
						type="button"
						onClick={() => setCategory(null)}
						className="rounded px-3 py-1 hover:bg-white/10"
					>
						Close
					</button>
				</div>
				<p className="my-4 text-sm text-slate-300">
					Import the concrete types from scryr. Add the integration and selected
					views to index.scry. Store credentials in scryr.secrets.toml beside
					it, then run scryr check and scryr serve --watch.
				</p>
				{definition.providers.map((provider) => (
					<section key={provider} className="mt-5">
						<h3 className="mb-2 font-semibold">{provider}</h3>
						<pre className="overflow-x-auto rounded bg-black/30 p-3 text-xs whitespace-pre-wrap">
							{setupInstructions(provider, definition.id)}
						</pre>
					</section>
				))}
			</div>
		</div>,
		document.body,
	);
}
