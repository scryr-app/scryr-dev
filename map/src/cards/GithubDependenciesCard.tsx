import { Container, Text } from "@react-three/uikit";
import { useEffect, useState } from "react";
import { currentTheme } from "@/theme/theme";
import {
	DEPENDENCY_SEVERITIES,
	type DependencyPart,
	type GithubDependenciesCardData,
} from "./githubDependenciesData";

const warning = "#f59e0b";
const danger = "#dc2626";
const PAGE_SIZE = 20;

function usePage(repository: string | undefined, total: number) {
	const [selection, setSelection] = useState({ repository, page: 0 });
	const lastPage = Math.max(0, Math.ceil(total / PAGE_SIZE) - 1);
	const page =
		selection.repository === repository
			? Math.min(selection.page, lastPage)
			: 0;
	useEffect(() => {
		setSelection((previous) => {
			const nextPage =
				previous.repository === repository
					? Math.min(previous.page, lastPage)
					: 0;
			return previous.repository === repository && previous.page === nextPage
				? previous
				: { repository, page: nextPage };
		});
	}, [repository, lastPage]);
	return {
		page,
		lastPage,
		total,
		setPage: (next: number) =>
			setSelection({ repository, page: Math.max(0, Math.min(next, lastPage)) }),
	};
}

function PageControls({
	label,
	pagination,
}: {
	label: string;
	pagination: ReturnType<typeof usePage>;
}) {
	const { page, lastPage, total, setPage } = pagination;
	return (
		<Container flexDirection="column" flexShrink={0} gap={2}>
			<Text fontSize={9} color={currentTheme.cardMutedTextColor}>
				{`${label}: ${total ? `${page * PAGE_SIZE + 1}–${Math.min((page + 1) * PAGE_SIZE, total)}` : "0"} of ${total}`}
			</Text>
			{lastPage > 0 && (
				<Container flexDirection="row" justifyContent="space-between">
					<Container onClick={page > 0 ? () => setPage(page - 1) : undefined}>
						<Text
							fontSize={9}
							color={
								page > 0
									? currentTheme.cardTextColor
									: currentTheme.cardMutedTextColor
							}
						>{`Previous ${label.toLowerCase()}`}</Text>
					</Container>
					<Container
						onClick={page < lastPage ? () => setPage(page + 1) : undefined}
					>
						<Text
							fontSize={9}
							color={
								page < lastPage
									? currentTheme.cardTextColor
									: currentTheme.cardMutedTextColor
							}
						>{`Next ${label.toLowerCase()}`}</Text>
					</Container>
				</Container>
			)}
		</Container>
	);
}

function utc(value: string): string {
	return new Date(value)
		.toISOString()
		.replace("T", " ")
		.replace(/\.\d+Z$/, " UTC");
}
function SyncDetails({
	label,
	part,
}: {
	label: string;
	part: DependencyPart<{ observedAt: string }>;
}) {
	if (part.state === "disabled") return null;
	return (
		<Container flexDirection="column" flexShrink={0} gap={2}>
			{part.snapshot && (
				<Text
					fontSize={8}
					color={currentTheme.cardMutedTextColor}
				>{`${label} observed: ${utc(part.snapshot.observedAt)}${part.stale ? " · stale" : ""}`}</Text>
			)}
			{part.sync?.lastSuccessAt && (
				<Text
					fontSize={8}
					color={currentTheme.cardMutedTextColor}
				>{`${label} last sync: ${utc(part.sync.lastSuccessAt)}`}</Text>
			)}
			{part.sync?.error && (
				<Text
					fontSize={9}
					color={warning}
				>{`${label} collection error: ${part.sync.error}`}</Text>
			)}
			{part.sync?.error && part.sync.lastAttemptAt && (
				<Text
					fontSize={8}
					color={currentTheme.cardMutedTextColor}
				>{`Attempted: ${utc(part.sync.lastAttemptAt)}`}</Text>
			)}
		</Container>
	);
}

/** Repository-wide snapshots with collection failures kept separate from zero alerts. */
export function GithubDependenciesCard({
	repository,
	inventory,
	security,
}: GithubDependenciesCardData) {
	const snapshot = security.snapshot;
	const alertPage = usePage(repository, snapshot?.alerts.length ?? 0);
	const packagePage = usePage(
		repository,
		inventory.snapshot?.packages.length ?? 0,
	);
	const securityKnown = security.state === "current" && snapshot;
	const securitySummary = securityKnown
		? `${snapshot.vulnerablePackages} vulnerable packages · ${snapshot.alerts.length} open alerts`
		: security.state === "disabled"
			? "Security disabled"
			: "Current security unknown";
	const securityColor = securityKnown
		? snapshot.alerts.length > 0
			? danger
			: currentTheme.cardTextColor
		: security.state === "disabled"
			? currentTheme.cardMutedTextColor
			: warning;
	return (
		<Container
			sizeX={2.8}
			sizeY={1.8}
			pixelSize={0.01}
			flexDirection="column"
			padding={12}
			gap={3}
		>
			<Text fontSize={15} color={currentTheme.cardTextColor}>
				Dependencies
			</Text>
			<Text
				fontSize={8}
				color={currentTheme.cardMutedTextColor}
			>{`Repository-wide · ${repository ?? "repository not resolved"}`}</Text>
			<Text fontSize={10} color={securityColor}>
				{securitySummary}
			</Text>
			<Text
				fontSize={9}
				color={
					inventory.state === "unknown"
						? warning
						: currentTheme.cardMutedTextColor
				}
			>
				{inventory.state === "current" && inventory.snapshot
					? `Packages: ${inventory.snapshot.packages.length}`
					: inventory.state === "disabled"
						? "Inventory disabled"
						: "Inventory unavailable"}
			</Text>
			<Container
				flexDirection="column"
				flexGrow={1}
				flexBasis={0}
				minHeight={0}
				overflow="scroll"
				gap={5}
			>
				{snapshot && (
					<Container flexDirection="column" flexShrink={0} gap={2}>
						{security.state !== "current" && (
							<Text
								fontSize={9}
								color={warning}
							>{`Last known: ${snapshot.vulnerablePackages} vulnerable packages · ${snapshot.alerts.length} open alerts`}</Text>
						)}
						<Text
							fontSize={9}
							color={securityColor}
						>{`${security.state === "current" ? "Highest severity" : "Last known highest severity"}: ${snapshot.highestSeverity}`}</Text>
						<Text fontSize={8} color={currentTheme.cardMutedTextColor}>
							{DEPENDENCY_SEVERITIES.map(
								(severity) =>
									`${severity}: ${snapshot.severityCounts[severity]}`,
							).join(" · ")}
						</Text>
					</Container>
				)}
				<SyncDetails label="Security" part={security} />
				<SyncDetails label="Inventory" part={inventory} />
				{snapshot && <PageControls label="Alerts" pagination={alertPage} />}
				{snapshot?.alerts
					.slice(alertPage.page * PAGE_SIZE, (alertPage.page + 1) * PAGE_SIZE)
					.map((alert) => (
						<Container
							key={alert.number}
							flexDirection="column"
							flexShrink={0}
							gap={2}
							onClick={
								alert.url
									? () =>
											window.open(alert.url, "_blank", "noopener,noreferrer")
									: undefined
							}
						>
							<Text
								fontSize={10}
								color={securityColor}
							>{`${alert.package} (${alert.ecosystem}) · ${alert.severity}${alert.url ? " ↗" : ""}`}</Text>
							{alert.summary && (
								<Text fontSize={9} color={currentTheme.cardTextColor}>
									{alert.summary}
								</Text>
							)}
							<Text
								fontSize={8}
								color={currentTheme.cardMutedTextColor}
							>{`#${alert.number}${alert.ghsaId ? ` · ${alert.ghsaId}` : ""}${alert.cveId ? ` · ${alert.cveId}` : ""}${alert.manifestPath ? ` · ${alert.manifestPath}` : ""}`}</Text>
							{alert.vulnerableVersionRange && (
								<Text
									fontSize={8}
									color={currentTheme.cardMutedTextColor}
								>{`Affected: ${alert.vulnerableVersionRange}`}</Text>
							)}
							<Text fontSize={9} color={currentTheme.cardTextColor}>
								{alert.firstPatchedVersion
									? `First patched version: ${alert.firstPatchedVersion}`
									: "Patched version not reported"}
							</Text>
						</Container>
					))}
				{inventory.snapshot && (
					<Container flexDirection="column" flexShrink={0} gap={2}>
						<Text
							fontSize={10}
							color={
								inventory.state === "current"
									? currentTheme.cardTextColor
									: warning
							}
						>{`${inventory.state === "current" ? "Inventory" : "Last known inventory"}: ${inventory.snapshot.packages.length} packages`}</Text>
						{inventory.snapshot.directDeps !== undefined && (
							<Text
								fontSize={9}
								color={currentTheme.cardMutedTextColor}
							>{`Direct dependencies: ${inventory.snapshot.directDeps}`}</Text>
						)}
						{inventory.snapshot.transitiveDeps !== undefined && (
							<Text
								fontSize={9}
								color={currentTheme.cardMutedTextColor}
							>{`Transitive dependencies: ${inventory.snapshot.transitiveDeps}`}</Text>
						)}
						<PageControls label="Packages" pagination={packagePage} />
						{inventory.snapshot.packages
							.slice(
								packagePage.page * PAGE_SIZE,
								(packagePage.page + 1) * PAGE_SIZE,
							)
							.map((item, index) => (
								<Text
									key={`${item.id}:${index}`}
									fontSize={8}
									color={currentTheme.cardMutedTextColor}
								>{`${item.name}${item.version ? ` ${item.version}` : ""}${item.license ? ` · ${item.license}` : ""}`}</Text>
							))}
					</Container>
				)}
			</Container>
		</Container>
	);
}
