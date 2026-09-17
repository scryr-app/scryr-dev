import { Container, Text } from "@react-three/uikit";
import { useState } from "react";
import { currentTheme } from "@/theme/theme";
import { useEvidenceDetails } from "./EvidenceDetailsContext";
import {
	type CollectorEvidence,
	collectorKey,
	collectorStatus,
	type EvidenceSection,
	integrationTitle,
	resultSummary,
	SECTION_TITLES,
	statusTone,
} from "./evidence";

export interface EvidenceCardProps {
	collectors: CollectorEvidence[];
}
export function EvidenceCard({
	section,
	collectors,
}: EvidenceCardProps & { section: EvidenceSection }) {
	const [selectedId, setSelectedId] = useState<string | null>(null);
	const details = useEvidenceDetails();
	const index = Math.max(
		0,
		collectors.findIndex((collector) => collectorKey(collector) === selectedId),
	);
	const collector = collectors[index];
	if (!collector) return null;
	const tone = statusTone(collector);
	const statusColor =
		tone === "error"
			? "#ef4444"
			: tone === "warning"
				? "#eab308"
				: currentTheme.cardMutedTextColor;
	const lines = collector.latest
		? resultSummary(collector.latest.result)
		: [collector.message ?? "Waiting for a local collection."];
	return (
		<Container
			sizeX={2.8}
			sizeY={1.8}
			pixelSize={0.01}
			flexDirection="column"
			padding={12}
			gap={5}
		>
			<Container flexDirection="row" justifyContent="space-between">
				<Text fontSize={16} color={currentTheme.cardTextColor}>
					{SECTION_TITLES[section].toUpperCase()}
				</Text>
				<Text
					fontSize={9}
					color={currentTheme.cardMutedTextColor}
				>{`${index + 1}/${collectors.length}`}</Text>
			</Container>
			<Container flexDirection="row" justifyContent="space-between">
				<Text
					fontSize={10}
					color={currentTheme.cardTextColor}
				>{`${integrationTitle(collector.integration)} · ${collector.collectorId}`}</Text>
			</Container>
			<Text fontSize={9} color={statusColor}>
				{collectorStatus(collector)}
			</Text>
			<Container flexDirection="column" flexGrow={1} gap={3} overflow="hidden">
				{lines.slice(0, 3).map((line) => (
					<Text key={line} fontSize={10} color={currentTheme.cardTextColor}>
						{line.slice(0, 105)}
					</Text>
				))}
			</Container>
			<Text fontSize={8} color={currentTheme.cardMutedTextColor}>
				{collector.latest
					? `${collector.latest.environment} · ${collector.latest.observedAt} · ${collector.latest.commitSha?.slice(0, 8) ?? collector.latest.workspaceId.slice(0, 12)}`
					: "Configured locally · No observation yet"}
			</Text>
			<Container flexDirection="row" justifyContent="space-between">
				{collectors.length > 1 && (
					<Container
						cursor="pointer"
						onClick={(event) => {
							event.stopPropagation();
							setSelectedId(
								collectorKey(
									collectors[
										(index + collectors.length - 1) % collectors.length
									],
								),
							);
						}}
					>
						<Text fontSize={10} color={currentTheme.cardTextColor}>
							Previous
						</Text>
					</Container>
				)}
				<Container
					cursor="pointer"
					onClick={(event) => {
						event.stopPropagation();
						details.open({
							section,
							collectors,
							selectedId: collectorKey(collector),
						});
					}}
				>
					<Text fontSize={10} color={currentTheme.cardTextColor}>
						Details & history
					</Text>
				</Container>
				{collectors.length > 1 && (
					<Container
						cursor="pointer"
						onClick={(event) => {
							event.stopPropagation();
							setSelectedId(
								collectorKey(collectors[(index + 1) % collectors.length]),
							);
						}}
					>
						<Text fontSize={10} color={currentTheme.cardTextColor}>
							Next
						</Text>
					</Container>
				)}
			</Container>
		</Container>
	);
}
