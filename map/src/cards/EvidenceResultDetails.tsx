import { type ReactNode, useId, useState } from "react";
import {
	type CollectorEvidence,
	detectedLicenses,
	type EvidenceResult,
	formatNumber,
	formatSeconds,
	metricLabel,
	type Observation,
	safeHttpUrl,
} from "./evidence";

export type DependencyPanel = "inventory" | "licenses" | "vulnerabilities";
export function EvidenceLink({
	href,
	children,
}: {
	href?: string | null;
	children: ReactNode;
}) {
	const safe = safeHttpUrl(href);
	return safe ? (
		<a
			className="text-sky-300 underline underline-offset-2"
			href={safe}
			target="_blank"
			rel="noreferrer"
		>
			{children}
		</a>
	) : (
		<span>{children}</span>
	);
}
interface DetailRow {
	id: string;
	values: ReactNode[];
	search: string;
	category?: string;
}
export interface DetailPage {
	search: string;
	filter: string;
	offset: number;
	limit: number;
}
export interface RemoteDetailPage extends DetailPage {
	total: number;
	loading: boolean;
	onChange: (page: DetailPage) => void;
}
function DetailTable({
	columns,
	rows,
	categoryLabel = "Category",
	remote,
	categoryOptions,
}: {
	columns: string[];
	rows: DetailRow[];
	categoryLabel?: string;
	remote?: RemoteDetailPage;
	categoryOptions?: string[];
}) {
	const categoryId = useId();
	const [localSearch, setSearch] = useState("");
	const [localCategory, setCategory] = useState("");
	const [page, setPage] = useState(0);
	const search = remote?.search ?? localSearch;
	const category = remote?.filter ?? localCategory;
	const categories =
		categoryOptions ??
		Array.from(
			new Set(rows.flatMap((row) => (row.category ? [row.category] : []))),
		).sort();
	const filtered = remote
		? rows
		: rows.filter(
				(row) =>
					(!category || row.category === category) &&
					row.search.toLowerCase().includes(search.toLowerCase()),
			);
	const total = remote?.total ?? filtered.length;
	const limit = remote?.limit ?? 25;
	const pages = Math.max(1, Math.ceil(total / limit));
	const currentPage = remote
		? Math.floor(remote.offset / limit)
		: Math.min(page, pages - 1);
	const displayed = remote
		? filtered
		: filtered.slice(currentPage * limit, (currentPage + 1) * limit);
	const changeSearch = (value: string) => {
		if (remote)
			remote.onChange({
				search: value,
				filter: remote.filter,
				offset: 0,
				limit: remote.limit,
			});
		else {
			setSearch(value);
			setPage(0);
		}
	};
	const changeCategory = (value: string) => {
		if (remote)
			remote.onChange({
				search: remote.search,
				filter: value,
				offset: 0,
				limit: remote.limit,
			});
		else {
			setCategory(value);
			setPage(0);
		}
	};
	const changePage = (value: number) => {
		if (remote)
			remote.onChange({
				search: remote.search,
				filter: remote.filter,
				offset: value * limit,
				limit: remote.limit,
			});
		else setPage(value);
	};

	return (
		<div className="space-y-3" aria-busy={remote?.loading}>
			<div className="flex flex-wrap items-end gap-3">
				<label className="flex grow flex-col gap-1 text-xs text-slate-300">
					Filter results
					<input
						className="rounded border border-white/20 bg-slate-900 p-2 text-sm text-white"
						value={search}
						onChange={(event) => changeSearch(event.target.value)}
						placeholder="Package, license, advisory, path, or name"
					/>
				</label>
				{(categories.length > 0 || remote) && (
					<label
						htmlFor={categoryId}
						className="flex flex-col gap-1 text-xs text-slate-300"
					>
						{categoryLabel}
						{remote && !categoryOptions ? (
							<input
								id={categoryId}
								className="rounded border border-white/20 bg-slate-900 p-2 text-sm text-white"
								placeholder="All (or exact value)"
								value={category}
								onChange={(event) => changeCategory(event.target.value)}
							/>
						) : (
							<select
								id={categoryId}
								className="rounded border border-white/20 bg-slate-900 p-2 text-sm text-white"
								value={category}
								onChange={(event) => changeCategory(event.target.value)}
							>
								<option value="">All</option>
								{categories.map((value) => (
									<option key={value}>{value}</option>
								))}
							</select>
						)}
					</label>
				)}
			</div>
			{remote?.loading && (
				<p role="status" className="text-xs text-slate-400">
					Loading filtered page…
				</p>
			)}
			<div className="overflow-x-auto rounded border border-white/10">
				<table className="w-full text-left text-sm">
					<thead className="bg-white/5">
						<tr>
							{columns.map((column) => (
								<th key={column} className="p-3 font-medium text-slate-300">
									{column}
								</th>
							))}
						</tr>
					</thead>
					<tbody>
						{displayed.map((row) => (
							<tr key={row.id} className="border-t border-white/10">
								{row.values.map((value, index) => (
									<td
										key={columns[index]}
										className="max-w-lg break-words p-3 align-top"
									>
										{value}
									</td>
								))}
							</tr>
						))}
					</tbody>
				</table>
				{!displayed.length && !remote?.loading && (
					<p className="p-4 text-sm text-slate-400">No matching results.</p>
				)}
			</div>
			<div className="flex items-center justify-between text-xs text-slate-300">
				<span>
					{total} results · Page {currentPage + 1} of {pages}
				</span>
				<div className="flex gap-3">
					<button
						className="disabled:opacity-30"
						type="button"
						disabled={currentPage === 0 || remote?.loading}
						onClick={() => changePage(currentPage - 1)}
					>
						Previous results
					</button>
					<button
						className="disabled:opacity-30"
						type="button"
						disabled={currentPage + 1 >= pages || remote?.loading}
						onClick={() => changePage(currentPage + 1)}
					>
						Next results
					</button>
				</div>
			</div>
		</div>
	);
}

/** Never join package IDs from a different inventory, workspace, or environment. */
export function matchingInventoryObservation(
	observation: Observation,
	collectors: CollectorEvidence[],
): Observation | undefined {
	const result = observation.result;
	if (
		result.__typename !== "LicenseResult" &&
		result.__typename !== "VulnerabilityResult"
	)
		return undefined;
	return (
		collectors
			.map((collector) => collector.latest)
			.find(
				(candidate) =>
					candidate?.workspaceId === observation.workspaceId &&
					candidate.environment === observation.environment &&
					candidate.result.__typename === "InventoryResult" &&
					candidate.result.artifactHash === result.inventoryHash,
			) ?? undefined
	);
}
export function matchingInventory(
	observation: Observation,
	collectors: CollectorEvidence[],
): Extract<EvidenceResult, { __typename: "InventoryResult" }> | undefined {
	const result = matchingInventoryObservation(observation, collectors)?.result;
	return result?.__typename === "InventoryResult" ? result : undefined;
}

function CompleteNotice({
	complete,
	subject,
}: {
	complete: boolean;
	subject: string;
}) {
	return !complete ? (
		<p
			role="status"
			className="rounded border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-200"
		>
			Incomplete {subject}. Missing data is unknown; these results do not
			establish a clean state.
		</p>
	) : null;
}
export function EvidenceResultDetails({
	observation,
	collectors,
	dependencyPanel = "inventory",
	remote,
	packageLabels,
}: {
	observation: Observation;
	collectors: CollectorEvidence[];
	dependencyPanel?: DependencyPanel;
	remote?: RemoteDetailPage;
	packageLabels?: ReadonlyMap<string, string>;
}) {
	const result = observation.result;
	const inventory = matchingInventory(observation, collectors);
	const packageLabel = (id: string) => {
		if (packageLabels?.has(id)) return packageLabels.get(id);
		const pkg = inventory?.packages.find((item) => item.id === id);
		return pkg ? `${pkg.name}@${pkg.version} (${pkg.ecosystem})` : id;
	};
	switch (result.__typename) {
		case "GitResult":
			return (
				<dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-3 text-sm">
					<dt>Branch</dt>
					<dd>{result.branch ?? "Detached HEAD"}</dd>
					<dt>Commit</dt>
					<dd className="break-all font-mono">
						{result.commit ?? "No commit"}
					</dd>
					<dt>Working tree</dt>
					<dd>
						{result.dirty ? `${result.changedFiles} changed files` : "Clean"}
					</dd>
					<dt>Tracking</dt>
					<dd>
						{result.ahead} ahead · {result.behind} behind
					</dd>
					<dt>Remote</dt>
					<dd>
						<EvidenceLink href={result.remoteUrl}>
							{result.remoteUrl ?? "No remote"}
						</EvidenceLink>
					</dd>
				</dl>
			);
		case "PullRequestsResult":
			return (
				<>
					<CompleteNotice
						complete={result.complete}
						subject="pull request listing"
					/>
					<DetailTable
						columns={["Pull request", "Status", "Review", "Branch / commit"]}
						categoryLabel="Review"
						rows={result.items.map((item) => ({
							id: String(item.number),
							category: item.reviewDecision ?? "Unknown",
							search: `${item.number} ${item.title} ${item.state} ${item.reviewDecision} ${item.headRefName}`,
							values: [
								<EvidenceLink key="link" href={item.url}>
									#{item.number} {item.title}
								</EvidenceLink>,
								item.state,
								item.reviewDecision ?? "Unknown",
								`${item.headRefName ?? "Unknown"} / ${item.headRefOid?.slice(0, 8) ?? "Unknown"}`,
							],
						}))}
					/>
				</>
			);
		case "WorkflowsResult":
			return (
				<>
					<p className="text-sm text-slate-300">
						Remote runs from {result.repository}. Branches and commits belong to
						each workflow run.
					</p>
					<CompleteNotice
						complete={result.complete}
						subject="workflow listing"
					/>
					<DetailTable
						columns={[
							"Workflow",
							"Status",
							"Branch / commit",
							"Run / attempt",
							"Updated",
						]}
						categoryLabel="Conclusion"
						rows={result.items.map((item) => ({
							id: `${item.runId}:${item.attempt}`,
							category: item.conclusion ?? item.status,
							search: `${item.name} ${item.status} ${item.conclusion} ${item.branch} ${item.commit}`,
							values: [
								<EvidenceLink key="link" href={item.url}>
									{item.name}
								</EvidenceLink>,
								item.conclusion ?? item.status,
								`${item.branch} / ${item.commit.slice(0, 8)}`,
								`${item.runId} / ${item.attempt}`,
								item.updatedAt,
							],
						}))}
					/>
				</>
			);
		case "CheckResult":
			return (
				<>
					<p className={result.passed ? "text-emerald-300" : "text-red-300"}>
						{result.name}: {result.passed ? "Passed" : "Failed"} ·{" "}
						{formatSeconds(result.durationSeconds)}
					</p>
					<DetailTable
						columns={["Severity", "Diagnostic", "Location"]}
						categoryLabel="Severity"
						rows={result.diagnostics.map((item, index) => ({
							id: String(index),
							category: item.severity,
							search: `${item.severity} ${item.message} ${item.path}`,
							values: [
								item.severity,
								item.message,
								item.path
									? `${item.path}${item.line ? `:${item.line}` : ""}`
									: "—",
							],
						}))}
					/>
				</>
			);
		case "TestResult":
			return (
				<>
					<p className="text-sm">
						Suite {result.suite}: {result.passing} passed, {result.failing}{" "}
						failed, {result.errors} errors, {result.skipped} skipped ·{" "}
						{formatSeconds(result.durationSeconds)}
					</p>
					<DetailTable
						columns={["Test", "Suite", "Status", "Duration", "Message"]}
						categoryLabel="Status"
						rows={result.cases.map((item, index) => ({
							id: `${item.suite}:${item.name}:${index}`,
							category: item.status,
							search: `${item.name} ${item.suite} ${item.status} ${item.message}`,
							values: [
								item.name,
								item.suite,
								item.status,
								formatSeconds(item.durationSeconds),
								item.message ?? "—",
							],
						}))}
					/>
				</>
			);
		case "CoverageResult":
			return (
				<div className="space-y-3">
					<p>Suite: {result.suite}</p>
					<p className="text-3xl">
						{result.total > 0
							? `${formatNumber((100 * result.covered) / result.total)}%`
							: "Coverage unavailable"}
					</p>
					<p>
						{result.covered} covered / {result.total} measured lines
					</p>
					<p className="text-sm text-slate-400">
						Coverage is scoped to this collector and input revision; it is not
						combined with unrelated test runs.
					</p>
				</div>
			);
		case "InventoryResult":
			return (
				<>
					<CompleteNotice complete={result.complete} subject="inventory" />
					{dependencyPanel === "licenses" ? (
						<>
							<p className="text-sm text-amber-200">
								Detected license evidence from Syft. Policy has not been
								evaluated by this collector.
							</p>
							<DetailTable
								remote={remote}
								columns={["Package", "License expressions", "Evidence"]}
								categoryLabel={remote ? "Ecosystem" : "License completeness"}
								rows={result.packages.map((item) => ({
									id: item.id,
									category: detectedLicenses(item.licenses).length
										? "Detected"
										: "Unknown",
									search: `${item.name} ${item.version} ${item.licenses.join(" ")}`,
									values: [
										`${item.name}@${item.version}`,
										detectedLicenses(item.licenses).join("; ") || "Unknown",
										item.paths.join(", ") || "No source path",
									],
								}))}
							/>
						</>
					) : (
						<>
							<DetailTable
								remote={remote}
								columns={[
									"Package",
									"Version",
									"Ecosystem",
									"Licenses",
									"Source",
								]}
								categoryLabel="Ecosystem"
								rows={result.packages.map((item) => ({
									id: item.id,
									category: item.ecosystem,
									search: `${item.name} ${item.version} ${item.ecosystem} ${item.purl} ${item.paths.join(" ")} ${item.licenses.join(" ")}`,
									values: [
										item.name,
										item.version,
										item.ecosystem,
										detectedLicenses(item.licenses).join("; ") || "Unknown",
										item.paths.join(", ") || item.purl || "Unknown",
									],
								}))}
							/>
							<p className="text-xs text-slate-400">
								{result.totalRelationships} dependency relationships · Inventory{" "}
								{result.artifactHash}
							</p>
						</>
					)}
				</>
			);
		case "LicenseResult":
			return (
				<>
					<CompleteNotice
						complete={result.complete}
						subject="license evaluation"
					/>
					{!inventory && (
						<p className="text-amber-200">
							Matching inventory unavailable. Package identifiers are shown
							without joining unrelated inventories.
						</p>
					)}
					<p className="break-all text-xs text-slate-400">
						Policy {result.policyRevision} · Inventory {result.inventoryHash}
					</p>
					<DetailTable
						remote={remote}
						columns={["Package", "License expression", "Decision", "Reason"]}
						categoryLabel="Decision"
						categoryOptions={["allow", "deny", "review"]}
						rows={result.items.map((item, index) => ({
							id: `${item.packageId}:${index}`,
							category: item.decision,
							search: `${packageLabel(item.packageId)} ${item.expression} ${item.decision} ${item.reason}`,
							values: [
								packageLabel(item.packageId),
								item.expression ?? "Unknown",
								item.decision,
								item.reason,
							],
						}))}
					/>
				</>
			);
		case "VulnerabilityResult":
			return (
				<>
					<CompleteNotice
						complete={result.complete}
						subject="vulnerability scan"
					/>
					{!inventory && (
						<p className="text-amber-200">
							Matching inventory unavailable. Package identifiers are shown
							without joining unrelated inventories.
						</p>
					)}
					<p className="text-sm text-slate-300">
						Advisory database: {result.databaseVersion ?? "Unknown version"} ·
						Age{" "}
						{result.databaseAgeSeconds == null
							? "unknown"
							: formatSeconds(result.databaseAgeSeconds)}
					</p>
					<DetailTable
						remote={remote}
						columns={[
							"Package",
							"Advisory",
							"Severity",
							"Fixed versions",
							"Aliases",
						]}
						categoryLabel="Severity"
						categoryOptions={[
							"critical",
							"high",
							"medium",
							"low",
							"negligible",
							"unknown",
						]}
						rows={result.items.map((item, index) => ({
							id: `${item.packageId}:${item.advisoryId}:${index}`,
							category: item.severity,
							search: `${packageLabel(item.packageId)} ${item.advisoryId} ${item.severity} ${item.aliases.join(" ")} ${item.fixVersions.join(" ")}`,
							values: [
								packageLabel(item.packageId),
								<EvidenceLink key="link" href={item.advisoryUrl}>
									{item.advisoryId}
								</EvidenceLink>,
								item.severity,
								item.fixVersions.join(", ") || "No fix reported",
								item.aliases.join(", ") || "—",
							],
						}))}
					/>
				</>
			);
		case "MetricsResult":
			return (
				<>
					<CompleteNotice complete={result.complete} subject="scrape" />
					<p className="text-sm text-slate-300">
						Scraped {result.scrapedAt}. A rate requires two valid samples;
						unavailable series are never treated as zero.
					</p>
					<DetailTable
						columns={["Metric / labels", "Value", "Unit", "Type"]}
						categoryLabel="Metric type"
						rows={result.samples.map((item, index) => ({
							id: `${item.name}:${index}`,
							category: item.metricType,
							search: `${metricLabel(item)} ${item.metricType} ${item.unit}`,
							values: [
								metricLabel(item),
								formatNumber(item.value),
								item.unit ?? "—",
								item.metricType,
							],
						}))}
					/>
				</>
			);
		case "BenchmarkResult":
			return (
				<div className="space-y-4">
					<p className="break-all rounded bg-black/30 p-3 font-mono text-sm">
						{result.command}
					</p>
					<dl className="grid grid-cols-2 gap-3 text-sm">
						<dt>Mean</dt>
						<dd>{formatSeconds(result.meanSeconds)}</dd>
						<dt>Median</dt>
						<dd>{formatSeconds(result.medianSeconds)}</dd>
						<dt>Standard deviation</dt>
						<dd>{formatSeconds(result.stddevSeconds)}</dd>
						<dt>Runs</dt>
						<dd>{result.runs}</dd>
						<dt>Machine</dt>
						<dd>{result.machine}</dd>
						<dt>Baseline</dt>
						<dd>
							{result.baselineMeanSeconds == null
								? "No comparable baseline"
								: formatSeconds(result.baselineMeanSeconds)}
						</dd>
					</dl>
					<p className="text-sm text-slate-400">
						Compare measurements only with equivalent commands, inputs, and
						machines.
					</p>
				</div>
			);
	}
}
