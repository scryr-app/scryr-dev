import { Container, Text } from "@react-three/uikit";
import {
	Activity,
	GitFork,
	Github,
	GitPullRequest,
	Shield,
	Star,
	Tag,
	Users,
} from "@react-three/uikit-lucide";
import type { ReactNode } from "react";
import { currentTheme } from "@/theme/theme";

// Dimensions match Block defaults: width(3) * 0.8, height(2) * 0.8
const CARD_SIZE_X = 2.8;
const CARD_SIZE_Y = 1.8;
// 1 pixel = 0.01 world units -> 240x160 virtual pixel space
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

export interface GithubCardProps {
	/** Repository URL */
	repoUrl?: string;
	/** Number of stars */
	stars?: number;
	/** Number of forks */
	forks?: number;
	/** Number of open issues */
	openIssues?: number;
	/** Number of open PRs */
	openPRs?: number;
	/** Last commit date */
	lastCommit?: string;
	/** Primary language */
	primaryLanguage?: string;
	/** Lines of code */
	linesOfCode?: number;
	/** Test coverage percentage */
	coverage?: number;
	/** Security vulnerabilities count */
	vulnerabilities?: number;
	/** Outdated dependencies count */
	outdatedDeps?: number;
	/** Active contributors (last 30 days) */
	activeContributors?: number;
	/** Latest release version */
	latestRelease?: string;
	/** License type */
	license?: string;
	/** Build status */
	buildStatus?: "passing" | "failing" | "pending";
}

/**
 * GithubCard displays comprehensive GitHub repository information on a 3D card.
 * Shows repo stats, code health, security, and activity metrics.
 */
export function GithubCard({
	repoUrl,
	stars,
	forks,
	openIssues,
	openPRs,
	lastCommit,
	primaryLanguage,
	linesOfCode,
	coverage,
	vulnerabilities,
	outdatedDeps,
	activeContributors,
	latestRelease,
	license,
	buildStatus,
}: GithubCardProps) {
	const buildColor = {
		passing: "#34d399",
		failing: "#dc2626",
		pending: "#f59e0b",
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
			{/* Header: title + release */}
			<Container
				flexDirection="row"
				justifyContent="space-between"
				alignItems="center"
			>
				<Container flexDirection="row" alignItems="center" gap={4}>
					<Github width={12} height={12} color={c} />
					<Text fontSize={16} color={c}>
						GITHUB
					</Text>
				</Container>
				{latestRelease && (
					<Container flexDirection="row" alignItems="center" gap={3}>
						<Tag width={9} height={9} color={currentTheme.cardMutedTextColor} />
						<Text fontSize={11} color={currentTheme.cardMutedTextColor}>
							{latestRelease}
						</Text>
					</Container>
				)}
			</Container>

			{/* Repo */}
			<Section
				label="REPO"
				icon={
					<Github
						width={8}
						height={8}
						color={currentTheme.cardMutedTextColor}
					/>
				}
			>
				{repoUrl && (
					<Text fontSize={11} color={c}>
						{repoUrl.split("/").slice(-2).join("/")}
					</Text>
				)}
				{(primaryLanguage ?? license) && (
					<Text fontSize={11} color={currentTheme.cardMutedTextColor}>
						{[primaryLanguage, license].filter(Boolean).join(" / ")}
					</Text>
				)}
			</Section>

			{/* Activity */}
			<Section
				label="ACTIVITY"
				icon={
					<Activity
						width={8}
						height={8}
						color={currentTheme.cardMutedTextColor}
					/>
				}
			>
				{stars !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Star width={10} height={10} color={c} />
						</Container>
						<Text fontSize={13} color={c}>
							{stars}
						</Text>
					</Container>
				)}
				{forks !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<GitFork width={10} height={10} color={c} />
						</Container>
						<Text fontSize={13} color={c}>
							{forks}
						</Text>
					</Container>
				)}
				{activeContributors !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Users width={10} height={10} color={c} />
						</Container>
						<Text fontSize={13} color={c}>
							{activeContributors}
						</Text>
					</Container>
				)}
			</Section>

			{/* Issues / PRs */}
			<Section
				label="ISSUES"
				icon={
					<GitPullRequest
						width={8}
						height={8}
						color={currentTheme.cardMutedTextColor}
					/>
				}
			>
				{openIssues !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Activity
								width={10}
								height={10}
								color={openIssues > 20 ? "#f59e0b" : c}
							/>
						</Container>
						<Text fontSize={13} color={openIssues > 20 ? "#f59e0b" : c}>
							{openIssues} open
						</Text>
					</Container>
				)}
				{openPRs !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<GitPullRequest width={10} height={10} color={c} />
						</Container>
						<Text fontSize={13} color={c}>
							{openPRs}
						</Text>
					</Container>
				)}
			</Section>

			{/* Health */}
			<Section
				label="HEALTH"
				icon={
					<Shield
						width={8}
						height={8}
						color={currentTheme.cardMutedTextColor}
					/>
				}
			>
				{buildStatus && (
					<Text fontSize={13} color={buildColor[buildStatus]}>
						{buildStatus}
					</Text>
				)}
				{coverage !== undefined && (
					<Text fontSize={13} color={coverage < 70 ? "#f59e0b" : c}>
						{coverage}% cov
					</Text>
				)}
				{vulnerabilities !== undefined && (
					<Container flexDirection="row" alignItems="center" gap={4}>
						<Container
							backgroundColor="rgba(255,255,255,0.08)"
							borderRadius={3}
							padding={2}
						>
							<Shield
								width={10}
								height={10}
								color={vulnerabilities > 0 ? "#dc2626" : "#34d399"}
							/>
						</Container>
						<Text
							fontSize={13}
							color={vulnerabilities > 0 ? "#dc2626" : "#34d399"}
						>
							{vulnerabilities > 0 ? `${vulnerabilities} cve` : "no cve"}
						</Text>
					</Container>
				)}
				{outdatedDeps !== undefined && (
					<Text fontSize={13} color={outdatedDeps > 5 ? "#f59e0b" : c}>
						{outdatedDeps} outdated
					</Text>
				)}
			</Section>

			{/* Footer */}
			<Container
				flexDirection="row"
				justifyContent="space-between"
				alignItems="center"
			>
				{lastCommit && (
					<Text fontSize={10} color={currentTheme.cardMutedTextColor}>
						{lastCommit}
					</Text>
				)}
				{linesOfCode !== undefined && (
					<Text fontSize={10} color={currentTheme.cardMutedTextColor}>
						{linesOfCode >= 1000
							? `${Math.round(linesOfCode / 1000)}k`
							: String(linesOfCode)}{" "}
						loc
					</Text>
				)}
			</Container>
		</Container>
	);
}
