import { Container, Text } from "@react-three/uikit";
import {
	Key,
	Package,
	RefreshCw,
	ShieldAlert,
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

export interface DependenciesCardProps {
	/** Total number of dependencies */
	totalDeps?: number;
	/** Number of outdated dependencies */
	outdatedDeps?: number;
	/** Number of dependencies with vulnerabilities */
	vulnerableDeps?: number;
	/** Highest severity vulnerability level */
	maxSeverity?: "critical" | "high" | "medium" | "low" | "none";
	/** Number of direct dependencies */
	directDeps?: number;
	/** Number of transitive dependencies */
	transitiveDeps?: number;
	/** Dependency update lag (days behind latest) */
	updateLag?: number;
	/** License compliance status */
	licenseCompliance?: "compliant" | "warning" | "violation";
}

/**
 * DependenciesCard displays dependency health and security information.
 * Shows package counts, vulnerabilities, update status, and compliance.
 */
export function DependenciesCard({
	totalDeps,
	outdatedDeps,
	vulnerableDeps,
	maxSeverity,
	directDeps,
	transitiveDeps,
	updateLag,
	licenseCompliance,
}: DependenciesCardProps) {
	const severityColor = {
		critical: "#dc2626",
		high: "#ea580c",
		medium: "#f59e0b",
		low: "#84cc16",
		none: "#34d399",
	};

	const complianceColor = {
		compliant: "#34d399",
		warning: "#f59e0b",
		violation: "#dc2626",
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
			{totalDeps === undefined && vulnerableDeps === undefined && (
				<Text fontSize={10} color="#b8c2ce">
					No dependency results reported
				</Text>
			)}
			{/* Header */}
			<Container flexDirection="row" alignItems="center" gap={4}>
				<Package width={12} height={12} color={c} />
				<Text fontSize={16} color={c}>
					DEPS
				</Text>
			</Container>

			{/* Packages */}
			<Section
				label="PACKAGES"
				icon={
					<Package
						width={8}
						height={8}
						color={currentTheme.cardMutedTextColor}
					/>
				}
			>
				{totalDeps !== undefined && (
					<Text fontSize={13} color={c}>
						{totalDeps} total
					</Text>
				)}
				{directDeps !== undefined && (
					<Text fontSize={13} color={currentTheme.cardMutedTextColor}>
						{directDeps} direct
					</Text>
				)}
				{transitiveDeps !== undefined && (
					<Text fontSize={13} color={currentTheme.cardMutedTextColor}>
						{transitiveDeps} trans
					</Text>
				)}
			</Section>

			{/* Freshness */}
			<Section
				label="FRESHNESS"
				icon={
					<RefreshCw
						width={8}
						height={8}
						color={currentTheme.cardMutedTextColor}
					/>
				}
			>
				{outdatedDeps !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<RefreshCw
								width={10}
								height={10}
								color={outdatedDeps > 10 ? "#f59e0b" : "#34d399"}
							/>
						</Container>
						<Text
							fontSize={13}
							color={outdatedDeps > 10 ? "#f59e0b" : "#34d399"}
						>
							{outdatedDeps} outdated
						</Text>
					</Container>
				)}
				{updateLag !== undefined && (
					<Text fontSize={13} color={updateLag > 30 ? "#f59e0b" : c}>
						{updateLag}d lag
					</Text>
				)}
			</Section>

			{/* Security */}
			<Section
				label="SECURITY"
				icon={
					<ShieldAlert
						width={8}
						height={8}
						color={currentTheme.cardMutedTextColor}
					/>
				}
			>
				{vulnerableDeps !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<ShieldAlert
								width={10}
								height={10}
								color={vulnerableDeps > 0 ? "#dc2626" : "#34d399"}
							/>
						</Container>
						<Text
							fontSize={13}
							color={vulnerableDeps > 0 ? "#dc2626" : "#34d399"}
						>
							{vulnerableDeps > 0 ? `${vulnerableDeps} vuln` : "no vuln"}
						</Text>
					</Container>
				)}
				{maxSeverity && (
					<Text fontSize={13} color={severityColor[maxSeverity]}>
						{maxSeverity}
					</Text>
				)}
			</Section>

			{/* License */}
			{licenseCompliance && (
				<Section
					label="LICENSE"
					icon={
						<Key width={8} height={8} color={currentTheme.cardMutedTextColor} />
					}
				>
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Key
								width={10}
								height={10}
								color={complianceColor[licenseCompliance]}
							/>
						</Container>
						<Text fontSize={13} color={complianceColor[licenseCompliance]}>
							{licenseCompliance}
						</Text>
					</Container>
				</Section>
			)}
		</Container>
	);
}
