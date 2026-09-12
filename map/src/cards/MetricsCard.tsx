import { Container, Text } from "@react-three/uikit";
import {
	Activity,
	Cpu,
	MemoryStick,
	ShieldCheck,
	Timer,
	Zap,
} from "@react-three/uikit-lucide";
import type { ReactNode } from "react";
import { currentTheme } from "@/theme/theme";

const CARD_SIZE_X = 2.8;
const CARD_SIZE_Y = 1.8;
const PIXEL_SIZE = 0.01;

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
			backgroundColor={currentTheme.cardInsetColor}
			borderRadius={5}
			padding={7}
			gap={4}
		>
			{label && (
				<Container flexDirection="row" alignItems="center" gap={3}>
					{icon}
					<Text fontSize={8} color={currentTheme.cardMutedTextColor}>
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

export interface MetricsCardProps {
	/** Response time p50 in ms */
	responseTimeP50?: number;
	/** Response time p95 in ms */
	responseTimeP95?: number;
	/** Response time p99 in ms */
	responseTimeP99?: number;
	/** Request rate (requests per minute) */
	requestRate?: number;
	/** Error rate percentage */
	errorRate?: number;
	/** Success rate percentage */
	successRate?: number;
	/** Uptime percentage */
	uptime?: number;
	/** Active connections count */
	activeConnections?: number;
	/** CPU usage percentage */
	cpuUsage?: number;
	/** Memory usage percentage */
	memoryUsage?: number;
}

/**
 * MetricsCard displays service performance metrics on a 3D card.
 * Shows response times, request rates, error rates, uptime, and resource usage.
 */
export function MetricsCard({
	responseTimeP50,
	responseTimeP95,
	responseTimeP99,
	requestRate,
	errorRate,
	successRate,
	uptime,
	activeConnections,
	cpuUsage,
	memoryUsage,
}: MetricsCardProps) {
	const c = currentTheme.cardTextColor;

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
			<Container flexDirection="row" alignItems="center" gap={4}>
				<Activity width={12} height={12} color={c} />
				<Text fontSize={16} color={c}>
					METRICS
				</Text>
			</Container>

			{/* Latency */}
			<Section
				label="LATENCY"
				icon={
					<Timer width={8} height={8} color={currentTheme.cardMutedTextColor} />
				}
			>
				{responseTimeP50 !== undefined && (
					<Text fontSize={13} color={c}>
						p50 {responseTimeP50}ms
					</Text>
				)}
				{responseTimeP95 !== undefined && (
					<Text fontSize={13} color={responseTimeP95 > 500 ? "#f59e0b" : c}>
						p95 {responseTimeP95}ms
					</Text>
				)}
				{responseTimeP99 !== undefined && (
					<Text
						fontSize={13}
						color={
							responseTimeP99 > 1000
								? "#dc2626"
								: responseTimeP99 > 500
									? "#f59e0b"
									: c
						}
					>
						p99 {responseTimeP99}ms
					</Text>
				)}
			</Section>

			{/* Throughput */}
			<Section
				label="THROUGHPUT"
				icon={
					<Zap width={8} height={8} color={currentTheme.cardMutedTextColor} />
				}
			>
				{requestRate !== undefined && (
					<Text fontSize={13} color={c}>
						{requestRate} req/m
					</Text>
				)}
				{activeConnections !== undefined && (
					<Text fontSize={13} color={c}>
						{activeConnections} conn
					</Text>
				)}
			</Section>

			{/* Reliability */}
			<Section
				label="RELIABILITY"
				icon={
					<ShieldCheck
						width={8}
						height={8}
						color={currentTheme.cardMutedTextColor}
					/>
				}
			>
				{uptime !== undefined && (
					<Text fontSize={13} color={uptime < 99.9 ? "#f59e0b" : "#34d399"}>
						{uptime}% up
					</Text>
				)}
				{successRate !== undefined && (
					<Text fontSize={13} color={successRate < 99 ? "#f59e0b" : c}>
						{successRate}% ok
					</Text>
				)}
				{errorRate !== undefined && (
					<Text fontSize={13} color={errorRate > 1 ? "#dc2626" : "#34d399"}>
						{errorRate}% err
					</Text>
				)}
			</Section>

			{/* Resources */}
			<Section
				label="RESOURCES"
				icon={
					<Cpu width={8} height={8} color={currentTheme.cardMutedTextColor} />
				}
			>
				{cpuUsage !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Cpu
								width={10}
								height={10}
								color={
									cpuUsage > 80 ? "#dc2626" : cpuUsage > 60 ? "#f59e0b" : c
								}
							/>
						</Container>
						<Text
							fontSize={13}
							color={cpuUsage > 80 ? "#dc2626" : cpuUsage > 60 ? "#f59e0b" : c}
						>
							{cpuUsage}%
						</Text>
					</Container>
				)}
				{memoryUsage !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<MemoryStick
								width={10}
								height={10}
								color={
									memoryUsage > 80
										? "#dc2626"
										: memoryUsage > 60
											? "#f59e0b"
											: c
								}
							/>
						</Container>
						<Text
							fontSize={13}
							color={
								memoryUsage > 80 ? "#dc2626" : memoryUsage > 60 ? "#f59e0b" : c
							}
						>
							{memoryUsage}%
						</Text>
					</Container>
				)}
			</Section>
		</Container>
	);
}
