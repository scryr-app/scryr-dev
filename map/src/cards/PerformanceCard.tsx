import { Container, Text } from "@react-three/uikit";
import { Clock, Cpu, Gauge } from "@react-three/uikit-lucide";
import { currentTheme } from "@/theme/theme";

const CARD_SIZE_X = 2.8;
const CARD_SIZE_Y = 1.8;
const PIXEL_SIZE = 0.01;
const LABEL_COLOR = "rgba(255,255,255,0.40)";

// Chart visual constants (virtual px at PIXEL_SIZE=0.01)
const CHART_H = 40; // inner bar area height
const CHART_BAR_COLOR = "#60a5fa"; // blue-400 default bar color
const CHART_WARN_COLOR = "#f59e0b"; // amber for >60%
const CHART_CRIT_COLOR = "#ef4444"; // red for >80%

function barColor(pct: number): string {
	if (pct > 80) return CHART_CRIT_COLOR;
	if (pct > 60) return CHART_WARN_COLOR;
	return CHART_BAR_COLOR;
}

function cpuTextColor(pct: number | undefined): string {
	if (pct === undefined) return "rgba(255,255,255,0.6)";
	if (pct > 80) return CHART_CRIT_COLOR;
	if (pct > 60) return CHART_WARN_COLOR;
	return CHART_BAR_COLOR;
}

interface CpuChartProps {
	/** CPU usage percentages (0–100), oldest first */
	data: number[];
}

interface BarDatum {
	id: string;
	pct: number;
	h: number;
}

/** Pre-process CPU samples into sized, keyed bar data. */
function buildBars(data: number[], barW: number): BarDatum[] {
	return data.map((v, i) => {
		const pct = Math.min(100, Math.max(0, v));
		return {
			id: `bar-${i}-${barW}`,
			pct,
			h: Math.max(2, Math.round((pct / 100) * CHART_H)),
		};
	});
}

/**
 * Renders a bar-chart sparkline of CPU usage over time.
 * Each bar represents one sample; height encodes the percentage.
 * Bars colour from blue → amber → red as load increases.
 */
function CpuChart({ data }: CpuChartProps) {
	const count = data.length;

	// Distribute available width (220px content - 8px L+R padding) across bars
	const available = 204;
	const barW =
		count > 0 ? Math.max(3, Math.floor((available - count) / count)) : 8;
	const bars = buildBars(data, barW);

	return (
		<Container
			flexDirection="row"
			alignItems="flex-end"
			gap={1}
			height={CHART_H + 8}
			backgroundColor="rgba(0,0,0,0.25)"
			borderRadius={4}
			paddingLeft={4}
			paddingRight={4}
			paddingBottom={4}
			paddingTop={4}
			overflow="hidden"
		>
			{count === 0 ? (
				<Text fontSize={9} color={LABEL_COLOR}>
					no data
				</Text>
			) : (
				bars.map(({ id, pct, h }) => (
					<Container
						key={id}
						width={barW}
						height={h}
						backgroundColor={barColor(pct)}
						borderRadius={1}
					/>
				))
			)}
		</Container>
	);
}

export interface PerformanceCardProps {
	/** CPU usage history as percentages 0–100, oldest first */
	cpuHistory?: number[];
	/** Current CPU usage percentage */
	cpuCurrent?: number;
	/** Average CPU usage percentage over the window */
	cpuAvg?: number;
	/** Peak (max) CPU usage percentage over the window */
	cpuPeak?: number;
	/** Current memory usage percentage */
	memoryUsage?: number;
	/** Label describing the time window, e.g. "Last 10 min" */
	timeWindow?: string;
}

/**
 * PerformanceCard renders a CPU usage over time bar-chart sparkline
 * alongside current, average, and peak CPU readings and memory usage.
 *
 * Layout:
 *   ┌─ "CPU USAGE" label ─────────────── "Last N min" ─┐
 *   │  26%  cpu                                         │
 *   │  CPU HISTORY ─────────────────────────────────── │
 *   │  ▁▃▅▇▅▃▄▆▄▃▅▇▆▄▃▅▆▇ (bar chart)                 │
 *   ├─ AVG ──────┬─ PEAK ──────┬─ MEM ─────────────────┤
 *   │  24.3%     │  81.2%      │  62%                   │
 *   └────────────┴─────────────┴───────────────────────┘
 */
export function PerformanceCard({
	cpuHistory = [],
	cpuCurrent,
	timeWindow = "Window unavailable",
}: PerformanceCardProps) {
	const c = currentTheme.cardTextColor;

	return (
		<Container
			sizeX={CARD_SIZE_X}
			sizeY={CARD_SIZE_Y}
			pixelSize={PIXEL_SIZE}
			flexDirection="column"
			padding={10}
			gap={5}
		>
			{/* ── Header row ─────────────────────────────────────── */}
			<Container
				flexDirection="row"
				justifyContent="space-between"
				alignItems="center"
			>
				<Container flexDirection="row" alignItems="center" gap={4}>
					<Gauge width={12} height={12} color={c} />
					<Text fontSize={16} color={c}>
						PERFORMANCE
					</Text>
				</Container>
				<Text fontSize={11} color={LABEL_COLOR}>
					{timeWindow}
				</Text>
			</Container>

			{/* ── Current CPU ─────────────────────────────────────── */}
			<Container flexDirection="row" alignItems="flex-end" gap={5}>
				<Container
					backgroundColor="rgba(255,255,255,0.08)"
					borderRadius={3}
					padding={2}
				>
					<Cpu width={12} height={12} color={cpuTextColor(cpuCurrent)} />
				</Container>
				<Text fontSize={24} color={cpuTextColor(cpuCurrent)}>
					{cpuCurrent !== undefined ? `${cpuCurrent.toFixed(1)}%` : "--"}
				</Text>
				<Text fontSize={9} color={LABEL_COLOR}>
					now
				</Text>
			</Container>

			{/* ── Chart ───────────────────────────────────────────── */}
			<Container flexDirection="column" gap={3}>
				<Container flexDirection="row" alignItems="center" gap={3}>
					<Clock width={8} height={8} color={LABEL_COLOR} />
					<Text fontSize={8} color={LABEL_COLOR}>
						HISTORY
					</Text>
				</Container>
				<CpuChart data={cpuHistory} />
			</Container>
		</Container>
	);
}
