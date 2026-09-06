import { Container, Text } from "@react-three/uikit";
import {
	CircleCheck,
	Gauge,
	Minus,
	TestTube,
	TrendingDown,
	TrendingUp,
} from "@react-three/uikit-lucide";
import type { ReactNode } from "react";
import { currentTheme } from "@/theme/theme";

const CARD_SIZE_X = 2.8;
const CARD_SIZE_Y = 1.8;
const PIXEL_SIZE = 0.01;
const INSET_BG = "rgba(0,0,0,0.22)";
const LABEL_COLOR = "rgba(255,255,255,0.40)";

function Section({
	label,
	icon,
	children,
}: {
	label?: string;
	icon?: ReactNode;
	children: ReactNode;
}) {
	return (
		<Container
			flexDirection="column"
			backgroundColor={INSET_BG}
			borderRadius={5}
			padding={7}
			gap={4}
		>
			{label && (
				<Container flexDirection="row" alignItems="center" gap={3}>
					{icon}
					<Text fontSize={8} color={LABEL_COLOR}>
						{label}
					</Text>
				</Container>
			)}
			<Container
				flexDirection="row"
				justifyContent="space-between"
				alignItems="center"
			>
				{children}
			</Container>
		</Container>
	);
}

export interface TestsCardProps {
	/** Total number of tests */
	total?: number;
	/** Number of passing tests */
	passing?: number;
	/** Number of failing tests */
	failing?: number;
	/** Test coverage percentage */
	coverage?: number /** Coverage trend indicator (up, down, or stable) */;
	coverageTrend?: "up" | "down" | "stable";
	/** Number of flaky tests */
	flakyTests?: number;
	/** Test execution time in seconds */
	executionTime?: number /** Last test run time */;
	lastRun?: string;
}

/**
 * TestsCard displays test suite statistics on a 3D card.
 * Shows total tests, pass/fail breakdown, coverage, flaky tests, and execution time.
 */
export function TestsCard({
	total,
	passing,
	failing,
	coverage,
	coverageTrend,
	flakyTests,
	executionTime,
	lastRun,
}: TestsCardProps) {
	const c = currentTheme.cardTextColor;

	// Derived pass rate
	const passRate =
		passing !== undefined && total !== undefined && total > 0
			? Math.round((passing / total) * 100)
			: undefined;

	return (
		<Container
			sizeX={CARD_SIZE_X}
			sizeY={CARD_SIZE_Y}
			pixelSize={PIXEL_SIZE}
			flexDirection="column"
			padding={12}
			gap={5}
			alignItems="stretch"
		>
			{/* Header */}
			<Container
				flexDirection="row"
				justifyContent="space-between"
				alignItems="center"
			>
				<Container flexDirection="row" alignItems="center" gap={4}>
					<TestTube width={10} height={10} color={c} />
					<Text fontSize={16} color={c}>
						TESTS
					</Text>
				</Container>
				{lastRun && (
					<Text fontSize={10} color={LABEL_COLOR}>
						{lastRun}
					</Text>
				)}
			</Container>

			{/* Suite */}
			<Section
				label="SUITE"
				icon={<TestTube width={8} height={8} color={LABEL_COLOR} />}
			>
				{total !== undefined && (
					<Text fontSize={13} color={c}>
						{total} tests
					</Text>
				)}
				{executionTime !== undefined && (
					<Text fontSize={13} color={executionTime > 120 ? "#f59e0b" : c}>
						{executionTime}s
					</Text>
				)}
			</Section>

			{/* Results */}
			<Section
				label="RESULTS"
				icon={<CircleCheck width={8} height={8} color={LABEL_COLOR} />}
			>
				{passing !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<CircleCheck width={10} height={10} color="#34d399" />
						</Container>
						<Text fontSize={13} color="#34d399">
							{passing}
						</Text>
					</Container>
				)}
				{failing !== undefined && (
					<Text fontSize={13} color={failing > 0 ? "#dc2626" : "#34d399"}>
						{failing > 0 ? `${failing} fail` : "clean"}
					</Text>
				)}
				{passRate !== undefined && (
					<Text fontSize={13} color={passRate < 90 ? "#f59e0b" : "#34d399"}>
						{passRate}%
					</Text>
				)}
			</Section>

			{/* Coverage */}
			<Section
				label="COVERAGE"
				icon={<Gauge width={8} height={8} color={LABEL_COLOR} />}
			>
				{coverage !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Gauge
								width={10}
								height={10}
								color={coverage < 80 ? "#f59e0b" : "#34d399"}
							/>
						</Container>
						<Text fontSize={13} color={coverage < 80 ? "#f59e0b" : "#34d399"}>
							{coverage}%
						</Text>
						{coverageTrend === "up" && (
							<TrendingUp width={10} height={10} color="#34d399" />
						)}
						{coverageTrend === "down" && (
							<TrendingDown width={10} height={10} color="#dc2626" />
						)}
						{coverageTrend === "stable" && (
							<Minus width={10} height={10} color={LABEL_COLOR} />
						)}
					</Container>
				)}
				{flakyTests !== undefined && (
					<Text fontSize={13} color={flakyTests > 5 ? "#f59e0b" : c}>
						{flakyTests} flaky
					</Text>
				)}
			</Section>
		</Container>
	);
}
