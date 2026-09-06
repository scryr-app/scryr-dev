import { Container, Text } from "@react-three/uikit";
import { GitMerge, Hammer, Play, Rocket } from "@react-three/uikit-lucide";
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

export interface CICDCardProps {
	/** CI/CD platform name */
	platform?: string;
	/** Build status */
	buildStatus?: "passing" | "failing" | "pending";
	/** Last build timestamp */
	lastBuild?: string;
	/** Deployment status for production */
	deployStatusProd?: "deployed" | "deploying" | "failed";
	/** Deployment status for staging */
	deployStatusStaging?: "deployed" | "deploying" | "failed";
	/** Deployment frequency (deploys per week) */
	deployFrequency?: number;
	/** Pipeline duration in minutes */
	pipelineDuration?: number;
	/** Failed builds count (last 7 days) */
	failedBuilds?: number;
}

/**
 * CICDCard displays CI/CD pipeline information on a 3D card.
 * Shows build status, deployments, frequency, and pipeline metrics.
 */
export function CICDCard({
	platform,
	buildStatus,
	lastBuild,
	deployStatusProd,
	deployStatusStaging,
	deployFrequency,
	pipelineDuration,
	failedBuilds,
}: CICDCardProps) {
	const buildColor = {
		passing: "#34d399",
		failing: "#dc2626",
		pending: "#f59e0b",
	};
	const deployColor = {
		deployed: "#34d399",
		deploying: "#f59e0b",
		failed: "#dc2626",
	};
	const deployLabel = {
		deployed: "live",
		deploying: "building",
		failed: "failed",
	};

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
			{/* Header: title + platform */}
			<Container
				flexDirection="row"
				justifyContent="space-between"
				alignItems="center"
			>
				<Container flexDirection="row" alignItems="center" gap={4}>
					<Rocket width={12} height={12} color={c} />
					<Text fontSize={16} color={c}>
						CI/CD
					</Text>
				</Container>
				{platform && (
					<Text fontSize={11} color={LABEL_COLOR}>
						{platform}
					</Text>
				)}
			</Container>

			{/* Build */}
			<Section
				label="BUILD"
				icon={<Hammer width={8} height={8} color={LABEL_COLOR} />}
			>
				{buildStatus && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Hammer width={10} height={10} color={buildColor[buildStatus]} />
						</Container>
						<Text fontSize={13} color={buildColor[buildStatus]}>
							{buildStatus}
						</Text>
					</Container>
				)}
				{lastBuild && (
					<Text fontSize={11} color={LABEL_COLOR}>
						{lastBuild}
					</Text>
				)}
			</Section>

			{/* Deploy */}
			<Section
				label="DEPLOY"
				icon={<Rocket width={8} height={8} color={LABEL_COLOR} />}
			>
				{deployStatusProd && (
					<Container flexDirection="column" gap={2}>
						<Text fontSize={8} color={LABEL_COLOR}>
							prod
						</Text>
						<Text fontSize={12} color={deployColor[deployStatusProd]}>
							{deployLabel[deployStatusProd]}
						</Text>
					</Container>
				)}
				{deployStatusStaging && (
					<Container flexDirection="column" gap={2} alignItems="flex-end">
						<Text fontSize={8} color={LABEL_COLOR}>
							staging
						</Text>
						<Text fontSize={12} color={deployColor[deployStatusStaging]}>
							{deployLabel[deployStatusStaging]}
						</Text>
					</Container>
				)}
			</Section>

			{/* Pipeline */}
			<Section
				label="PIPELINE"
				icon={<GitMerge width={8} height={8} color={LABEL_COLOR} />}
			>
				{pipelineDuration !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<GitMerge
								width={10}
								height={10}
								color={pipelineDuration > 30 ? "#f59e0b" : c}
							/>
						</Container>
						<Text fontSize={13} color={pipelineDuration > 30 ? "#f59e0b" : c}>
							{pipelineDuration}m
						</Text>
					</Container>
				)}
				{deployFrequency !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Play width={10} height={10} color={c} />
						</Container>
						<Text fontSize={13} color={c}>
							{deployFrequency}x/wk
						</Text>
					</Container>
				)}
				{failedBuilds !== undefined && (
					<Text fontSize={13} color={failedBuilds > 0 ? "#dc2626" : "#34d399"}>
						{failedBuilds > 0 ? `${failedBuilds} fail` : "clean"}
					</Text>
				)}
			</Section>
		</Container>
	);
}
