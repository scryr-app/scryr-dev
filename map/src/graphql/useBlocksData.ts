import { useMemo } from "react";
import {
	type GetBlocksQuery,
	type GetBlocksQueryVariables,
	useGetBlocksQuery,
} from "@/graphql/generated";
import { useRuntimePreviewBlocks } from "@/graphql/runtimePreviewStore";
import { useDiagramMetrics } from "./useDiagramMetrics";

type Block = GetBlocksQuery["blocks"][number];
type RawRecord = Record<string, unknown>;

function asRecord(value: unknown): RawRecord | undefined {
	return value && typeof value === "object" && !Array.isArray(value)
		? (value as RawRecord)
		: undefined;
}

function parseRawJsonString(
	rawJsonString?: string | null,
): RawRecord | undefined {
	if (!rawJsonString) {
		return undefined;
	}

	try {
		return asRecord(JSON.parse(rawJsonString));
	} catch {
		return undefined;
	}
}

function getValue(source: RawRecord | undefined, path: string[]): unknown {
	let current: unknown = source;

	for (const part of path) {
		const record = asRecord(current);
		if (!record) {
			return undefined;
		}
		current = record[part];
	}

	return current;
}

function firstString(
	source: RawRecord | undefined,
	paths: string[][],
): string | null {
	for (const path of paths) {
		const value = getValue(source, path);
		if (typeof value === "string" && value.length > 0) {
			return value;
		}
	}

	return null;
}

function nonEmptyString(value: unknown): string | null {
	return typeof value === "string" && value.trim().length > 0 ? value : null;
}

function firstNumber(
	source: RawRecord | undefined,
	paths: string[][],
): number | null {
	for (const path of paths) {
		const value = getValue(source, path);
		if (typeof value === "number" && Number.isFinite(value)) {
			return value;
		}
	}

	return null;
}

function firstStringArray(
	source: RawRecord | undefined,
	paths: string[][],
): string[] {
	for (const path of paths) {
		const value = getValue(source, path);
		const strings = asStringArray(value);
		if (strings.length > 0) {
			return strings;
		}
	}

	return [];
}

function asStringArray(value: unknown): string[] {
	if (!Array.isArray(value)) {
		return [];
	}

	return value.filter((item): item is string => typeof item === "string");
}

function normalizeLinks(value: unknown): Block["links"] {
	if (!Array.isArray(value)) {
		return [];
	}

	return value
		.filter((link): link is Block["links"][number] => Boolean(link))
		.map((link) => ({
			siteName:
				typeof link.siteName === "string"
					? link.siteName
					: typeof (link as RawRecord).site_name === "string"
						? ((link as RawRecord).site_name as string)
						: null,
			httpUrl:
				typeof link.httpUrl === "string"
					? link.httpUrl
					: typeof (link as RawRecord).http_url === "string"
						? ((link as RawRecord).http_url as string)
						: null,
		}));
}

function normalizeBlock(block: Block): Block {
	// The 3D text font does not include these Unicode punctuation glyphs.
	// Normalize them at the data boundary so labels and metadata cards are safe.
	const sanitizedBlock = JSON.parse(
		JSON.stringify(block).replaceAll("…", "...").replaceAll("—", "-"),
	) as Block;
	const raw = parseRawJsonString(sanitizedBlock.rawJsonString);
	const rawLinks = normalizeLinks(
		getValue(raw, ["info", "links"]) ?? getValue(raw, ["links"]),
	);

	return {
		...sanitizedBlock,
		connections: asStringArray(sanitizedBlock.connections),
		description:
			nonEmptyString(sanitizedBlock.description) ??
			firstString(raw, [["info", "description"], ["description"]]),
		version:
			sanitizedBlock.version ??
			firstString(raw, [["info", "version"], ["version"]]),
		language:
			sanitizedBlock.language ??
			firstString(raw, [["info", "language"], ["language"]]),
		frameworks:
			asStringArray(sanitizedBlock.frameworks).length > 0
				? asStringArray(sanitizedBlock.frameworks)
				: firstStringArray(raw, [["info", "frameworks"], ["frameworks"]]),
		deployment:
			sanitizedBlock.deployment ??
			firstString(raw, [["info", "deployment"], ["deployment"]]),
		sourceCodeUrl:
			sanitizedBlock.sourceCodeUrl ??
			firstString(raw, [
				["github", "repoUrl"],
				["sourceCodeUrl"],
				["source_code_url"],
			]),
		docs:
			asStringArray(sanitizedBlock.docs).length > 0
				? asStringArray(sanitizedBlock.docs)
				: firstStringArray(raw, [["info", "docs"], ["docs"]]),
		ownerTeam:
			sanitizedBlock.ownerTeam ??
			firstString(raw, [
				["info", "ownerTeam"],
				["info", "owner_team"],
				["ownerTeam"],
				["owner_team"],
			]),
		authType:
			sanitizedBlock.authType ??
			firstString(raw, [
				["info", "authType"],
				["info", "auth_type"],
				["authType"],
				["auth_type"],
			]),
		monitoring:
			sanitizedBlock.monitoring ??
			firstString(raw, [["info", "monitoring"], ["monitoring"]]),
		logAggregation:
			sanitizedBlock.logAggregation ??
			firstString(raw, [
				["info", "logAggregation"],
				["info", "log_aggregation"],
				["logAggregation"],
				["log_aggregation"],
			]),
		tracing:
			sanitizedBlock.tracing ??
			firstString(raw, [["info", "tracing"], ["tracing"]]),
		iacTool:
			sanitizedBlock.iacTool ??
			firstString(raw, [
				["info", "iacTool"],
				["info", "iac_tool"],
				["iacTool"],
				["iac_tool"],
			]),
		cicdTool:
			sanitizedBlock.cicdTool ??
			firstString(raw, [["cicd", "platform"], ["cicdTool"], ["cicd_tool"]]),
		maxReplicas:
			sanitizedBlock.maxReplicas ??
			firstNumber(raw, [
				["info", "maxReplicas"],
				["info", "max_replicas"],
				["maxReplicas"],
				["max_replicas"],
			]),
		minReplicas:
			sanitizedBlock.minReplicas ??
			firstNumber(raw, [
				["info", "minReplicas"],
				["info", "min_replicas"],
				["minReplicas"],
				["min_replicas"],
			]),
		links:
			sanitizedBlock.links.length > 0
				? normalizeLinks(sanitizedBlock.links)
				: rawLinks,
		tags: asStringArray(sanitizedBlock.tags),
	};
}

function normalizeBlocksData(data?: GetBlocksQuery): Block[] {
	if (!data || !Array.isArray(data.blocks)) {
		return [];
	}

	return data.blocks.filter(Boolean).map(normalizeBlock);
}

export function useBlocksData(variables?: GetBlocksQueryVariables) {
	const runtimePreviewBlocks = useRuntimePreviewBlocks();
	const query = useGetBlocksQuery(variables, {
		enabled: runtimePreviewBlocks === null,
		refetchInterval: 30_000,
	});
	const normalized = useMemo(
		() => normalizeBlocksData(query.data),
		[query.data],
	);
	const configured = normalized.some((block) => {
		const raw = parseRawJsonString(block.rawJsonString);
		return Boolean(asRecord(raw?.metrics)?.provider || raw?.analytics);
	});
	const runtime = useDiagramMetrics(
		variables,
		runtimePreviewBlocks === null && configured,
	);
	const blocks = useMemo(
		() =>
			normalized.map((block) => {
				const raw = parseRawJsonString(block.rawJsonString);
				if (!raw || (!asRecord(raw.metrics)?.provider && !raw.analytics))
					return block;
				const id = typeof raw.manifestId === "string" ? raw.manifestId : "";
				const unavailable = {
					status: runtime.error
						? "error"
						: runtime.isFetching
							? "loading"
							: "no_data",
					error: runtime.error
						? "Unable to fetch diagram observations"
						: undefined,
					values: {},
				};
				const snapshot = runtime.data?.diagramMetrics[id];
				return {
					...block,
					rawJsonString: JSON.stringify({
						...raw,
						...(asRecord(raw.metrics)?.provider
							? { runtimeMetrics: snapshot ?? unavailable }
							: {}),
						...(raw.analytics
							? {
									runtimeAnalytics: {
										...(snapshot?.analytics ?? unavailable),
										source: "posthog",
									},
								}
							: {}),
					}),
				};
			}),
		[normalized, runtime.data, runtime.error, runtime.isFetching],
	);

	return {
		...query,
		blocks: runtimePreviewBlocks ?? blocks,
		error: runtimePreviewBlocks ? null : query.error,
		isLoading: runtimePreviewBlocks ? false : query.isLoading,
	};
}
