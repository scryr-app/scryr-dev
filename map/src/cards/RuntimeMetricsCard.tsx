import { Container, Text } from "@react-three/uikit";
import { currentTheme } from "@/theme/theme";
import type { RuntimeMetricSnapshot } from "../graphql/useDiagramMetrics";

export function runtimeMetricLines(
	snapshot: RuntimeMetricSnapshot,
	performance: boolean,
): string[] {
	if (snapshot.source === "posthog") {
		return Object.entries(snapshot.values)
			.map(
				([name, point]) =>
					`${point.label ?? name}: ${Number.isFinite(point.value) ? `${point.value} ${point.unit ?? ""}` : "unavailable"}`,
			)
			.concat((snapshot.missing ?? []).map((name) => `${name}: unavailable`));
	}
	const fields = performance
		? ["cpuCurrent", "cpuAvg", "cpuPeak", "memoryUsage"]
		: [
				"requestRate",
				"responseTimeP50",
				"responseTimeP95",
				"responseTimeP99",
				"errorRate",
			];
	const labels: Record<string, string> = {
		requestRate: "Request rate",
		responseTimeP50: "P50 latency",
		responseTimeP95: "P95 latency",
		responseTimeP99: "P99 latency",
		errorRate: "Server errors",
		cpuCurrent: "CPU current",
		cpuAvg: "CPU average",
		cpuPeak: "CPU peak",
		memoryUsage: "Memory usage",
	};
	return fields.map((name) => {
		const point = snapshot.values[name];
		return point && Number.isFinite(point.value)
			? `${labels[name]}: ${point.value.toFixed(2)} ${point.unit ?? ""}`
			: `${labels[name]}: unavailable`;
	});
}
export function RuntimeMetricsCard({
	snapshot,
	performance = false,
}: {
	snapshot: RuntimeMetricSnapshot;
	performance?: boolean;
}) {
	const analytics = snapshot.source === "posthog";
	const url =
		snapshot.dashboardUrl && /^https:\/\//.test(snapshot.dashboardUrl)
			? snapshot.dashboardUrl
			: undefined;
	return (
		<Container
			sizeX={2.8}
			sizeY={1.8}
			pixelSize={0.01}
			flexDirection="column"
			padding={12}
			gap={3}
		>
			<Text fontSize={16} color={currentTheme.cardTextColor}>
				{analytics ? "ANALYTICS" : performance ? "PERFORMANCE" : "METRICS"}
			</Text>
			<Text
				fontSize={10}
				color={snapshot.status === "ready" ? "#b8c2ce" : "#f59e0b"}
			>{`${snapshot.environment ?? ""} · ${{ loading: "Loading", ready: "Snapshot", partial: "Partial data", no_data: "No data", stale: "Stale snapshot", error: "Unavailable" }[snapshot.status]}`}</Text>
			{snapshot.status === "loading" ? (
				<Text fontSize={11} color="#b8c2ce">
					{analytics ? "Loading from PostHog…" : "Loading from Grafana…"}
				</Text>
			) : (
				runtimeMetricLines(snapshot, performance).map((line) => (
					<Text key={line} fontSize={10} color={currentTheme.cardTextColor}>
						{line}
					</Text>
				))
			)}
			{analytics &&
				snapshot.windowStart !== undefined &&
				snapshot.windowEnd !== undefined && (
					<Text
						fontSize={8}
						color="#b8c2ce"
					>{`${new Date(snapshot.windowStart * 1000).toISOString()} – ${new Date(snapshot.windowEnd * 1000).toISOString()}`}</Text>
				)}
			{snapshot.error && (
				<Text fontSize={8} color="#f59e0b">
					{snapshot.error}
				</Text>
			)}
			{snapshot.fetchedAt && (
				<Text
					fontSize={8}
					color="#b8c2ce"
				>{`Fetched ${snapshot.fetchedAt}`}</Text>
			)}
			{url && (
				<Container
					onClick={() => window.open(url, "_blank", "noopener,noreferrer")}
				>
					<Text fontSize={10} color="#60a5fa">
						{analytics ? "Open PostHog" : "Open Grafana for this window"}
					</Text>
				</Container>
			)}
		</Container>
	);
}
