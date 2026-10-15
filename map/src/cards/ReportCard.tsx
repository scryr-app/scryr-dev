import { Container, Text } from "@react-three/uikit";
import { currentTheme } from "@/theme/theme";

export interface OperationalReport {
	source: string;
	coverageTrend?: "up" | "down" | "stable";
	scope: string;
	observedAt: string;
	reportUrl?: string;
	runId: string;
	attempt: number;
	data: Record<string, unknown> & { kind: string };
}

export function isOperationalReport(
	value: unknown,
): value is OperationalReport {
	if (!value || typeof value !== "object") return false;
	const r = value as Partial<OperationalReport>;
	if (
		typeof r.scope !== "string" ||
		typeof r.source !== "string" ||
		typeof r.observedAt !== "string" ||
		!Number.isFinite(Date.parse(r.observedAt)) ||
		!r.data ||
		typeof r.data !== "object"
	)
		return false;
	const d = r.data;
	const numbers = (keys: string[]) =>
		keys.every(
			(k) =>
				typeof d[k] === "number" && Number.isFinite(d[k]) && Number(d[k]) >= 0,
		);
	if (d.kind === "tests")
		return numbers(["passing", "failing", "errors", "skipped", "duration"]);
	if (d.kind === "coverage")
		return (
			numbers(["covered", "total"]) && Number(d.covered) <= Number(d.total)
		);
	if (d.kind === "dependencies")
		return (
			Array.isArray(d.alerts) &&
			d.alerts.every(
				(a) =>
					a &&
					typeof a === "object" &&
					typeof a.state === "string" &&
					typeof a.severity === "string",
			)
		);
	return (
		d.kind === "deployment" &&
		typeof d.environment === "string" &&
		typeof d.status === "string" &&
		typeof d.version === "string"
	);
}

export function reportLines(report: OperationalReport): string[] {
	const d = report.data;
	if (d.kind === "tests")
		return [
			`${d.passing} passed - ${d.failing} failed`,
			`${d.errors} errors - ${d.skipped} skipped`,
			`${Number(d.duration).toFixed(2)} seconds`,
		];
	if (d.kind === "coverage")
		return [
			Number(d.total) > 0
				? `${((100 * Number(d.covered)) / Number(d.total)).toFixed(1)}% line coverage`
				: "Coverage unavailable: no executable lines",
			`${d.covered} / ${d.total} lines covered`,
			...(report.coverageTrend ? [`Trend: ${report.coverageTrend}`] : []),
		];
	if (d.kind === "dependencies") {
		const alerts = (
			d.alerts as { state: string; severity: string; package: string }[]
		).filter((a) => a.state === "open");
		return [
			`${alerts.length} open security alerts`,
			`critical: ${alerts.filter((a) => a.severity === "critical").length} - high: ${alerts.filter((a) => a.severity === "high").length}`,
			`medium: ${alerts.filter((a) => a.severity === "medium").length} - low: ${alerts.filter((a) => a.severity === "low").length}`,
			"Inventory and outdated counts not reported",
		];
	}
	return [`${d.environment}: ${d.status}`, `Version: ${d.version}`];
}

export function ReportCard({ report }: { report: OperationalReport }) {
	const stale =
		Date.now() - Date.parse(report.observedAt) > 7 * 24 * 3600 * 1000;
	const url =
		report.reportUrl && /^https?:\/\//.test(report.reportUrl)
			? report.reportUrl
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
				{report.data.kind.toUpperCase()}
			</Text>
			<Text fontSize={10} color="#b8c2ce">
				{report.scope}
			</Text>
			{reportLines(report).map((line) => (
				<Text key={line} fontSize={10} color={currentTheme.cardTextColor}>
					{line}
				</Text>
			))}
			<Text
				fontSize={8}
				color={stale ? "#f59e0b" : "#b8c2ce"}
			>{`${report.source} - ${report.observedAt}${stale ? " - older than 7 days" : ""}`}</Text>
			{url && (
				<Container
					onClick={() => window.open(url, "_blank", "noopener,noreferrer")}
				>
					<Text fontSize={10} color="#60a5fa">
						Open report
					</Text>
				</Container>
			)}
		</Container>
	);
}
