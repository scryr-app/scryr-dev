import { Container, Text } from "@react-three/uikit";
import type { CICDCardProps } from "./CICDCard";
import { currentTheme } from "@/theme/theme";
import type {
	GithubActionsCardData,
	WorkflowStatus,
} from "./githubActionsData";

const colors: Record<WorkflowStatus["outcome"], string> = {
	passing: "#34d399",
	failing: "#dc2626",
	pending: "#f59e0b",
	neutral: "#b8c2ce",
};

function utc(value: string): string {
	return new Date(value)
		.toISOString()
		.replace("T", " ")
		.replace(/\.\d+Z$/, " UTC");
}

export function GithubActionsCard({
	workflows,
	sync,
	pipeline,
}: GithubActionsCardData & { pipeline?: CICDCardProps }) {
	const failing = workflows.filter((w) => w.outcome === "failing").length;
	const pending = workflows.filter((w) => w.outcome === "pending").length;
	return (
		<Container
			sizeX={2.8}
			sizeY={1.8}
			pixelSize={0.01}
			flexDirection="column"
			padding={12}
			gap={4}
		>
			<Text fontSize={16} color={currentTheme.cardTextColor}>
				GitHub Actions
			</Text>
			<Text
				fontSize={10}
				color={failing ? colors.failing : currentTheme.cardMutedTextColor}
			>
				{`${workflows.length} workflow branches · ${failing} failing · ${pending} pending`}
			</Text>
			<Container
				flexDirection="column"
				flexGrow={1}
				flexBasis={0}
				minHeight={0}
				overflow="scroll"
				gap={5}
			>
				{workflows.length === 0 && (
					<Text fontSize={11} color={currentTheme.cardMutedTextColor}>
						No workflow runs observed
					</Text>
				)}
				{workflows.map((workflow) => (
					<Container
						key={workflow.key}
						flexDirection="column"
						flexShrink={0}
						gap={1}
						onClick={() => {
							if (workflow.url)
								window.open(workflow.url, "_blank", "noopener,noreferrer");
						}}
					>
						<Text fontSize={11} color={currentTheme.cardTextColor}>
							{workflow.name}
						</Text>
						<Text
							fontSize={10}
							color={colors[workflow.outcome]}
						>{`${workflow.branch ?? "No branch"} · ${workflow.status}${workflow.url ? " ↗" : ""}`}</Text>
					</Container>
				))}
				{pipeline && (
					<Container flexDirection="column" flexShrink={0} gap={2}>
						{pipeline.deployStatusProd && (
							<Text
								fontSize={9}
								color={currentTheme.cardTextColor}
							>{`Production: ${pipeline.deployStatusProd}`}</Text>
						)}
						{pipeline.deployStatusStaging && (
							<Text
								fontSize={9}
								color={currentTheme.cardTextColor}
							>{`Staging: ${pipeline.deployStatusStaging}`}</Text>
						)}
						{pipeline.pipelineDuration !== undefined && (
							<Text
								fontSize={9}
								color={currentTheme.cardTextColor}
							>{`Pipeline: ${pipeline.pipelineDuration} minutes`}</Text>
						)}
						{pipeline.deployFrequency !== undefined && (
							<Text
								fontSize={9}
								color={currentTheme.cardTextColor}
							>{`Deployments: ${pipeline.deployFrequency}/week`}</Text>
						)}
						{pipeline.failedBuilds !== undefined && (
							<Text
								fontSize={9}
								color={currentTheme.cardTextColor}
							>{`Failed builds (7 days): ${pipeline.failedBuilds}`}</Text>
						)}
					</Container>
				)}

				{sync?.error && (
					<Text
						fontSize={9}
						color={colors.pending}
					>{`Collection error: ${sync.error}`}</Text>
				)}
			</Container>
			<Text fontSize={8} color={currentTheme.cardMutedTextColor}>
				{sync?.lastSuccessAt
					? `Last sync: ${utc(sync.lastSuccessAt)}`
					: sync
						? "No successful sync yet"
						: "No polling sync reported"}
			</Text>
			{sync?.error && (
				<Text
					fontSize={8}
					color={colors.pending}
				>{`Sync failed${sync.lastAttemptAt ? `: ${utc(sync.lastAttemptAt)}` : ""} · showing last observations`}</Text>
			)}
		</Container>
	);
}
