import { useContext, useEffect, useRef, useState } from "react";
import { useGetEvidenceHistoryQuery } from "@/graphql/generated";
import { useSelectedMap } from "@/graphql/sampleStore";
import { useBlocksData } from "@/graphql/useBlocksData";
import { EditorScope } from "@/graphql/useManifestEditor";
import { getBlockCardData } from "./blockCardData";
import { DependencyDetails } from "./DependencyDetails";
import { useEvidenceDetails } from "./EvidenceDetailsContext";
import {
	type DependencyPanel,
	EvidenceResultDetails,
} from "./EvidenceResultDetails";
import {
	type CollectorEvidence,
	collectCommand,
	collectorKey,
	collectorStatus,
	type EvidenceSection,
	integrationTitle,
	type Observation,
	resultSummary,
	SECTION_TITLES,
	statusTone,
} from "./evidence";

function ObservationProvenance({ observation }: { observation: Observation }) {
	const fields = {
		"Environment / workspace": `${observation.environment} / ${observation.workspaceId}`,
		Scope: observation.scope,
		"Observed / started": `${observation.observedAt} / ${observation.startedAt}`,
		"Branch / commit": `${observation.branch ?? "Unknown"} / ${observation.commitSha ?? "Unknown"}${observation.dirty ? " (dirty)" : ""}`,
		"Tool version": observation.toolVersion ?? "Unknown",
		...(observation.sourceUpdatedAt
			? { "Artifact source timestamp": observation.sourceUpdatedAt }
			: {}),
		"Input fingerprint": observation.inputFingerprint,
		"Collector revision": observation.collectorRevision,
		"Run / attempt": `${observation.runId} / ${observation.attempt}`,
		Observation: observation.observationId,
		...(observation.upstreamFingerprint
			? { "Upstream fingerprint": observation.upstreamFingerprint }
			: {}),
		...(observation.policyRevision
			? { "Policy revision": observation.policyRevision }
			: {}),
	};
	return (
		<details className="rounded border border-white/10 p-3 text-xs">
			<summary className="cursor-pointer text-slate-300">
				Source and provenance
			</summary>
			<dl className="mt-3 grid grid-cols-[minmax(100px,auto)_1fr] gap-x-4 gap-y-2">
				{Object.entries(fields).map(([name, value]) => (
					<div key={name} className="contents">
						<dt className="text-slate-400">{name}</dt>
						<dd className="break-all">{value}</dd>
					</div>
				))}
			</dl>
		</details>
	);
}
function CollectorHistory({
	collector,
	onSelect,
}: {
	collector: CollectorEvidence;
	onSelect: (observation: Observation) => void;
}) {
	const scope = useContext(EditorScope);
	const [expanded, setExpanded] = useState(false);
	const [offset, setOffset] = useState(0);
	const variables = {
		manifestId: collector.manifestId,
		section: collector.section,
		collectorId: collector.collectorId,
		workspaceId: collector.workspaceId,
		limit: 20,
		offset,
	};
	const history = useGetEvidenceHistoryQuery(variables, {
		queryKey: ["GetEvidenceHistory", variables, scope],
		enabled: expanded && !!collector.workspaceId,
		staleTime: 5000,
		refetchInterval: expanded ? 10_000 : false,
		refetchIntervalInBackground: false,
	});

	return (
		<details
			className="rounded border border-white/10 p-3 text-sm"
			onToggle={(event) => setExpanded(event.currentTarget.open)}
		>
			<summary className="cursor-pointer">Run history</summary>
			{expanded && (
				<div className="mt-3 space-y-3">
					{history.isLoading && <p>Loading history…</p>}
					{!!history.error && (
						<p role="alert" className="text-red-300">
							History could not be loaded.{" "}
							<button
								type="button"
								className="underline"
								onClick={() => void history.refetch()}
							>
								Retry
							</button>
						</p>
					)}
					{!collector.workspaceId && (
						<p className="text-slate-400">
							History becomes available after this workspace registers its
							collectors.
						</p>
					)}
					{history.data?.evidenceHistory.length === 0 && (
						<p>No observations on this page.</p>
					)}
					{history.data?.evidenceHistory.map((observation) => (
						<button
							key={observation.observationId}
							type="button"
							className="block w-full rounded bg-white/5 p-3 text-left hover:bg-white/10"
							onClick={() => onSelect(observation)}
						>
							<span className="block text-xs text-slate-400">
								{observation.observedAt} · {observation.environment} ·{" "}
								{observation.commitSha?.slice(0, 8) ?? observation.runId}
							</span>
							{resultSummary(observation.result)[0]}
						</button>
					))}
					<div className="flex gap-4 text-xs">
						<button
							disabled={offset === 0}
							className="disabled:opacity-30"
							type="button"
							onClick={() => setOffset(Math.max(0, offset - 20))}
						>
							Newer runs
						</button>
						<button
							disabled={(history.data?.evidenceHistory.length ?? 0) < 20}
							className="disabled:opacity-30"
							type="button"
							onClick={() => setOffset(offset + 20)}
						>
							Older runs
						</button>
					</div>
				</div>
			)}
		</details>
	);
}
function dependencyMatches(
	collector: CollectorEvidence,
	panel: DependencyPanel,
): boolean {
	return panel === "inventory"
		? collector.integration === "syft_inventory"
		: panel === "licenses"
			? ["syft_inventory", "grant_license"].includes(collector.integration)
			: collector.integration === "grype_scan";
}
export function EvidenceDetailsView({
	section,
	collectors,
	selectedId,
	onClose,
	refreshFailed = false,
	onRetry,
}: {
	section: EvidenceSection;
	collectors: CollectorEvidence[];
	selectedId: string;
	onClose: () => void;
	refreshFailed?: boolean;
	onRetry?: () => void;
}) {
	const dialog = useRef<HTMLDialogElement>(null);
	const initial = collectors.find((item) => collectorKey(item) === selectedId);
	const [panel, setPanel] = useState<DependencyPanel>(
		initial?.integration === "grype_scan"
			? "vulnerabilities"
			: initial?.integration === "grant_license"
				? "licenses"
				: "inventory",
	);
	const [activeId, setActiveId] = useState(selectedId);
	const [historical, setHistorical] = useState<Observation | null>(null);
	const visible =
		section === "dependencies"
			? collectors.filter((item) => dependencyMatches(item, panel))
			: collectors;
	const collector =
		visible.find((item) => collectorKey(item) === activeId) ?? visible[0];
	const observation =
		historical &&
		historical.collectorId === collector?.collectorId &&
		historical.workspaceId === collector?.workspaceId
			? historical
			: collector?.latest;
	useEffect(() => {
		const node = dialog.current;
		node?.showModal();
		return () => node?.close();
	}, []);
	return (
		<dialog
			ref={dialog}
			aria-labelledby="evidence-dialog-title"
			onCancel={(event) => {
				event.preventDefault();
				onClose();
			}}
			className="fixed inset-0 z-[100] m-auto max-h-[85vh] w-[min(1000px,94vw)] overflow-y-auto rounded-xl border border-white/20 bg-slate-950 p-0 text-slate-100 shadow-2xl backdrop:bg-black/65"
		>
			<header className="sticky top-0 z-10 flex items-start justify-between border-b border-white/10 bg-slate-950 p-5">
				<div>
					<h2 id="evidence-dialog-title" className="text-lg font-semibold">
						{SECTION_TITLES[section]}
					</h2>
					<p className="mt-1 text-xs text-slate-400">
						{collectors[0]?.manifestId ?? "No active collectors"}
					</p>
				</div>
				<button
					type="button"
					className="rounded border border-white/20 px-3 py-1.5 text-sm"
					onClick={onClose}
				>
					Close
				</button>
			</header>
			<div className="space-y-5 p-5">
				{refreshFailed && (
					<p
						role="alert"
						className="rounded bg-amber-500/10 p-3 text-sm text-amber-200"
					>
						Unable to refresh collector status. Showing the last loaded
						evidence; current freshness is unknown.{" "}
						{onRetry && (
							<button type="button" className="underline" onClick={onRetry}>
								Retry
							</button>
						)}
					</p>
				)}
				{section === "dependencies" && (
					<fieldset
						className="grid grid-cols-3 gap-2"
						aria-label="Dependency panels"
					>
						{(["inventory", "licenses", "vulnerabilities"] as const).map(
							(value) => (
								<button
									key={value}
									type="button"
									aria-pressed={panel === value}
									className={`rounded border p-3 text-left capitalize ${panel === value ? "border-sky-400 bg-sky-500/10" : "border-white/15"}`}
									onClick={() => {
										setPanel(value);
										setHistorical(null);
									}}
								>
									{value}
									<span className="mt-1 block text-xs text-slate-400">
										{
											collectors.filter((item) =>
												dependencyMatches(item, value),
											).length
										}{" "}
										configured sources
									</span>
								</button>
							),
						)}
					</fieldset>
				)}
				{visible.length > 1 && (
					<label className="flex flex-col gap-2 text-sm">
						Collector
						<select
							className="rounded border border-white/20 bg-slate-900 p-2"
							value={collectorKey(collector)}
							onChange={(event) => {
								setActiveId(event.target.value);
								setHistorical(null);
							}}
						>
							{visible.map((item) => (
								<option key={collectorKey(item)} value={collectorKey(item)}>
									{item.collectorId} · {integrationTitle(item.integration)} ·{" "}
									{item.workspaceId || "waiting"}
								</option>
							))}
						</select>
					</label>
				)}
				{!collector ? (
					<p className="text-sm text-slate-300">
						{collectors.length
							? `No ${panel} collector configured in index.scry. This is not a clean result.`
							: "This collector is no longer declared in the selected diagram."}
					</p>
				) : (
					<>
						<div className="flex flex-wrap items-center justify-between gap-3">
							<h3 className="font-medium">
								{integrationTitle(collector.integration)} ·{" "}
								{collector.collectorId}
							</h3>
							<span
								className={
									statusTone(collector) === "error"
										? "text-red-300"
										: statusTone(collector) === "warning"
											? "text-amber-200"
											: "text-slate-300"
								}
							>
								{refreshFailed ? "Last known: " : ""}
								{collectorStatus(collector)}
							</span>
						</div>
						{collector.message && (
							<p className="rounded bg-white/5 p-3 text-sm">
								{collector.message}
							</p>
						)}
						{(collector.stale ||
							collector.outdated ||
							collector.state === "ERROR") &&
							collector.latest && (
								<p className="text-sm text-amber-200">
									Showing the last recorded result from{" "}
									{collector.latest.observedAt}.{" "}
									{collector.outdated
										? "Inputs or the collector definition changed; this result does not describe the current project."
										: "The last observation is retained while collection is unavailable or overdue."}
								</p>
							)}
						<p className="text-xs text-slate-400">
							Freshness policy: {collector.freshnessSeconds}s · Status updated{" "}
							{collector.updatedAt ?? "not yet"}
						</p>
						{historical && observation === historical && (
							<div className="flex justify-between rounded bg-amber-500/10 p-3 text-sm text-amber-100">
								<span>Historical observation · {historical.observedAt}</span>
								<button
									type="button"
									className="underline"
									onClick={() => setHistorical(null)}
								>
									Return to latest
								</button>
							</div>
						)}
						{observation ? (
							<>
								{section === "dependencies" ? (
									<DependencyDetails
										key={`${observation.observationId}:${panel}`}
										observation={observation}
										collectors={collectors}
										panel={panel}
									/>
								) : (
									<EvidenceResultDetails
										key={observation.observationId}
										observation={observation}
										collectors={collectors}
									/>
								)}
								<ObservationProvenance observation={observation} />
							</>
						) : (
							<p className="rounded border border-white/10 p-4 text-sm text-slate-300">
								No observation yet. Run this collector from the project
								directory or start its configured schedule with scryr serve.
							</p>
						)}
						<div className="space-y-2 rounded border border-white/10 p-3">
							<p className="text-xs text-slate-400">
								Run from the project directory on your laptop
							</p>
							<code className="block select-all break-all text-xs">
								{collectCommand(collector)}
							</code>
							{["MISSING_TOOL", "INCOMPATIBLE_TOOL", "NEEDS_LOGIN"].includes(
								collector.state,
							) && (
								<p className="text-xs text-amber-200">
									Run <code>scryr collect doctor</code> for installation and
									authentication guidance.
								</p>
							)}
						</div>
						<CollectorHistory
							key={collectorKey(collector)}
							collector={collector}
							onSelect={setHistorical}
						/>
					</>
				)}
			</div>
		</dialog>
	);
}

export function EvidenceDetails() {
	const { selection, close } = useEvidenceDetails();
	return selection ? (
		<LiveEvidenceDetails
			key={`${selection.section}:${selection.selectedId}`}
			onClose={close}
		/>
	) : null;
}
function LiveEvidenceDetails({ onClose }: { onClose: () => void }) {
	const { selection } = useEvidenceDetails();
	const selectedMap = useSelectedMap();
	const { blocks, data, isLoading, error, refetch } = useBlocksData(
		selectedMap.id
			? { scryIdentifier: selectedMap.id }
			: { sample: selectedMap.key || undefined },
		false,
	);
	if (!selection) return null;
	const manifestId = selection.collectors[0]?.manifestId;
	const current = blocks
		.flatMap((block) => getBlockCardData(block)[selection.section])
		.filter((collector) => collector.manifestId === manifestId);
	return (
		<EvidenceDetailsView
			section={selection.section}
			selectedId={selection.selectedId}
			collectors={
				isLoading || (error && !data) ? selection.collectors : current
			}
			onClose={onClose}
			refreshFailed={!!error}
			onRetry={() => void refetch()}
		/>
	);
}
