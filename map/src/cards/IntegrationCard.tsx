import { Container, Text } from "@react-three/uikit";
import { currentTheme } from "@/theme/theme";
import type { RuntimeMetricSnapshot } from "../graphql/useDiagramMetrics";
import { SetupLink } from "./IntegrationSetup";
import type { ActionRun, IntegrationView } from "./integrationCatalog";

export function selectActionRun(
	card: IntegrationView,
	runs: ActionRun[],
): ActionRun | undefined {
	return runs
		.filter(
			(run) =>
				run.repository === card.integration.repository &&
				(card.workflowId == null || run.workflowId === card.workflowId) &&
				(card.branch == null || run.headBranch === card.branch),
		)
		.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))[0];
}
export function IntegrationCard({
	card,
	snapshot,
	runs = [],
}: {
	card: IntegrationView;
	snapshot?: RuntimeMetricSnapshot;
	runs?: ActionRun[];
}) {
	const run =
		card.kind === "github_actions_pipeline"
			? selectActionRun(card, [...runs, ...(card.observations?.runs ?? [])])
			: undefined;
	const values = Object.entries(snapshot?.values ?? {});
	const empty = !run && values.length === 0 && snapshot?.status !== "loading";
	const status = run
		? `${run.status}${run.conclusion ? ` · ${run.conclusion}` : ""}`
		: snapshot?.status === "loading"
			? "Loading…"
			: !card.source && card.kind !== "github_actions_pipeline"
				? "Queries are not configured."
				: snapshot?.status === "stale"
					? "Stale data"
					: snapshot?.status === "partial"
						? "Partial data"
						: empty
							? "No data available."
							: "Snapshot";
	const url = snapshot?.dashboardUrl;
	return (
		<Container
			sizeX={2.8}
			sizeY={1.8}
			pixelSize={0.01}
			flexDirection="column"
			padding={12}
			gap={5}
		>
			<Text fontSize={15} paddingRight={18} color={currentTheme.cardTextColor}>
				{card.title}
			</Text>
			<Text fontSize={9} color="#b8c2ce">
				{card.category.toUpperCase()} · {status}
			</Text>
			{run && (
				<Text
					fontSize={10}
					color={currentTheme.cardTextColor}
				>{`${run.workflowName ?? "Workflow"} · ${run.updatedAt}`}</Text>
			)}
			{values.map(([name, point]) => (
				<Text key={name} fontSize={10} color={currentTheme.cardTextColor}>
					{`${point.label ?? name.replace(/([A-Z])/g, " $1")}: ${Number.isFinite(point.value) ? `${point.value.toFixed(2)} ${point.unit ?? ""}` : "unavailable"}`}
				</Text>
			))}
			{(snapshot?.missing ?? []).map((name) => (
				<Text
					key={name}
					fontSize={9}
					color="#f59e0b"
				>{`${name}: unavailable`}</Text>
			))}
			{snapshot?.error && (
				<Text fontSize={9} color="#f59e0b">
					{snapshot.error}
				</Text>
			)}
			{empty && <SetupLink category={card.category} />}
			{url && /^https:\/\//.test(url) && (
				<Container
					cursor="pointer"
					onClick={(e) => {
						e.stopPropagation();
						window.open(url, "_blank", "noopener,noreferrer");
					}}
				>
					<Text fontSize={10} color="#60a5fa">
						Open dashboard
					</Text>
				</Container>
			)}
		</Container>
	);
}
