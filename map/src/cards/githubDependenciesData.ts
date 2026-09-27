export const DEPENDENCY_STALE_AFTER_MS = 2 * 60 * 60 * 1000;
export const DEPENDENCY_SEVERITIES = [
	"critical",
	"high",
	"medium",
	"low",
] as const;
export type DependencySeverity = (typeof DEPENDENCY_SEVERITIES)[number];

type Raw = Record<string, unknown>;
export interface DependencySync {
	lastAttemptAt?: string;
	lastSuccessAt?: string;
	error?: string;
}
export interface DependencyInventory {
	observedAt: string;
	packages: {
		id: string;
		name: string;
		version?: string;
		license?: string;
		purl?: string;
	}[];
	directDeps?: number;
	transitiveDeps?: number;
}
export interface DependencyAlert {
	number: number;
	package: string;
	ecosystem: string;
	manifestPath: string;
	severity: DependencySeverity;
	url?: string;
	ghsaId?: string;
	cveId?: string;
	summary?: string;
	vulnerableVersionRange?: string;
	firstPatchedVersion?: string;
}
export interface DependencySecurity {
	observedAt: string;
	alerts: DependencyAlert[];
	vulnerablePackages: number;
	severityCounts: Record<DependencySeverity, number>;
	highestSeverity: DependencySeverity | "none";
}
export interface DependencyPart<T> {
	state: "current" | "unknown" | "disabled";
	stale: boolean;
	snapshot?: T;
	sync?: DependencySync;
}
export interface GithubDependenciesCardData {
	repository?: string;
	inventory: DependencyPart<DependencyInventory>;
	security: DependencyPart<DependencySecurity>;
}
function record(value: unknown): Raw | undefined {
	return value && typeof value === "object" && !Array.isArray(value)
		? (value as Raw)
		: undefined;
}
function text(value: unknown): string | undefined {
	return typeof value === "string" && value.length > 0 ? value : undefined;
}
function date(value: unknown): string | undefined {
	const result = text(value);
	return result && Number.isFinite(Date.parse(result)) ? result : undefined;
}
function count(value: unknown): number | undefined {
	return typeof value === "number" && Number.isSafeInteger(value) && value >= 0
		? value
		: undefined;
}
function repository(value: unknown): string | undefined {
	const result = text(value);
	return result && /^[^/\s]+\/[^/\s]+$/.test(result) ? result : undefined;
}
function repositoryFromUrl(value: unknown): string | undefined {
	if (typeof value !== "string") return undefined;
	try {
		const url = new URL(value);
		if (!["https:", "http:"].includes(url.protocol)) return undefined;
		return repository(
			url.pathname.replace(/^\/+|\/+$/g, "").replace(/\.git$/, ""),
		);
	} catch {
		return undefined;
	}
}
function sameRepository(value: unknown, expected: string | undefined): boolean {
	return (
		typeof value === "string" &&
		expected !== undefined &&
		value.toLowerCase() === expected.toLowerCase()
	);
}
function safeUrl(value: unknown): string | undefined {
	if (typeof value !== "string") return undefined;
	try {
		const url = new URL(value);
		return ["https:", "http:"].includes(url.protocol) &&
			!url.username &&
			!url.password
			? url.href
			: undefined;
	} catch {
		return undefined;
	}
}
function syncData(
	value: unknown,
	expectedRepository: string | undefined,
): DependencySync | undefined {
	const sync = record(value);
	if (
		!sync ||
		!sameRepository(record(sync.context)?.repository, expectedRepository)
	)
		return undefined;
	return {
		lastAttemptAt: date(sync.lastAttemptAt),
		lastSuccessAt: date(sync.lastSuccessAt),
		error: text(sync.error),
	};
}
function inventoryData(value: unknown): DependencyInventory | undefined {
	const snapshot = record(value);
	const observedAt = date(snapshot?.observedAt);
	if (!snapshot || !observedAt || !Array.isArray(snapshot.packages))
		return undefined;
	const packages: DependencyInventory["packages"] = [];
	for (const value of snapshot.packages) {
		const item = record(value);
		if (!item || !text(item.id) || !text(item.name)) return undefined;
		packages.push({
			id: String(item.id),
			name: String(item.name),
			version: text(item.version),
			license: text(item.license),
			purl: text(item.purl),
		});
	}
	return {
		observedAt,
		packages,
		directDeps: count(snapshot.directDeps),
		transitiveDeps: count(snapshot.transitiveDeps),
	};
}
function securityData(value: unknown): DependencySecurity | undefined {
	const snapshot = record(value);
	const observedAt = date(snapshot?.observedAt);
	if (!snapshot || !observedAt || !Array.isArray(snapshot.alerts))
		return undefined;
	const alerts = new Map<number, DependencyAlert>();
	for (const value of snapshot.alerts) {
		const item = record(value);
		if (
			!item ||
			!count(item.number) ||
			!text(item.package) ||
			!text(item.ecosystem) ||
			typeof item.manifestPath !== "string" ||
			!DEPENDENCY_SEVERITIES.includes(item.severity as DependencySeverity)
		)
			return undefined;
		alerts.set(Number(item.number), {
			number: Number(item.number),
			package: String(item.package),
			ecosystem: String(item.ecosystem),
			manifestPath: item.manifestPath,
			severity: item.severity as DependencySeverity,
			url: safeUrl(item.url),
			ghsaId: text(item.ghsaId),
			cveId: text(item.cveId),
			summary: text(item.summary),
			vulnerableVersionRange: text(item.vulnerableVersionRange),
			firstPatchedVersion: text(item.firstPatchedVersion),
		});
	}
	const sorted = [...alerts.values()].sort(
		(a, b) =>
			DEPENDENCY_SEVERITIES.indexOf(a.severity) -
				DEPENDENCY_SEVERITIES.indexOf(b.severity) || a.number - b.number,
	);
	const severityCounts = { critical: 0, high: 0, medium: 0, low: 0 };
	for (const alert of sorted) severityCounts[alert.severity] += 1;
	return {
		observedAt,
		alerts: sorted,
		vulnerablePackages: new Set(
			sorted.map((alert) => JSON.stringify([alert.ecosystem, alert.package])),
		).size,
		severityCounts,
		highestSeverity: sorted[0]?.severity ?? "none",
	};
}
function part<T extends { observedAt: string }>(
	enabled: boolean,
	snapshot: T | undefined,
	sync: DependencySync | undefined,
	now: number,
): DependencyPart<T> {
	if (!enabled) return { state: "disabled", stale: false };
	const stale = Boolean(
		snapshot &&
			(sync?.error ||
				now - Date.parse(snapshot.observedAt) > DEPENDENCY_STALE_AFTER_MS),
	);
	return {
		state: snapshot && !stale && !sync?.error ? "current" : "unknown",
		stale,
		snapshot,
		sync,
	};
}
/** Preserve independent collection health and never turn a failed scan into zero alerts. */
export function githubDependenciesData(
	raw: Raw | undefined,
	now = Date.now(),
): GithubDependenciesCardData | undefined {
	const dependencies = record(raw?.dependencies);
	const source = record(dependencies?.source);
	const github = record(dependencies?.github);
	const configured = source?.provider === "github";
	if (!configured && !github) return undefined;
	const repoUrl = record(raw?.github)?.repoUrl;
	const selectedRepository =
		repoUrl !== undefined
			? repositoryFromUrl(repoUrl)
			: repository(github?.repository);
	const matchingSnapshot = sameRepository(
		github?.repository,
		selectedRepository,
	)
		? github
		: undefined;
	const sync = record(raw?.providerSync);
	return {
		repository: selectedRepository,
		inventory: part(
			configured ? source?.inventory !== false : Boolean(github?.inventory),
			inventoryData(matchingSnapshot?.inventory),
			syncData(sync?.github_dependencies_inventory, selectedRepository),
			now,
		),
		security: part(
			configured ? source?.security !== false : Boolean(github?.security),
			securityData(matchingSnapshot?.security),
			syncData(sync?.github_dependencies_security, selectedRepository),
			now,
		),
	};
}
