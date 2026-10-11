import {
	type BlockCardData,
	hasGraphData,
	normalizeBlockCardData,
} from "@/cards/blockCardData";
import {
	asRecord,
	getValue,
	parseRawJsonString,
	type RawRecord,
	firstNumber as rawNumber,
	firstString as rawString,
} from "./blockJson";
import type { GetBlocksQuery } from "./generated";
import type { RuntimeMetricSnapshot } from "./useDiagramMetrics";

const firstString = (raw: RawRecord | undefined, paths: string[][]) =>
	rawString(raw, paths) ?? null;
const firstNumber = (raw: RawRecord | undefined, paths: string[][]) =>
	rawNumber(raw, paths) ?? null;

type Block = GetBlocksQuery["blocks"][number];
export interface BlockViewModel extends Block {
	cards: Omit<BlockCardData, "runtimeMetrics" | "runtimeAnalytics">;
	hasGraphData: boolean;
	manifestId: string;
	runtimeConfiguration: { metrics: boolean; analytics: boolean };
	observations: {
		metrics?: RuntimeMetricSnapshot;
		analytics?: RuntimeMetricSnapshot;
	};
}

// The 3D font lacks these punctuation glyphs. Preserve non-string data.
function sanitizeText<T>(value: T): T {
	if (typeof value === "string")
		return value.replaceAll("…", "...").replaceAll("—", "-") as T;
	if (Array.isArray(value)) return value.map(sanitizeText) as T;
	if (value && typeof value === "object")
		return Object.fromEntries(
			Object.entries(value).map(([key, item]) => [key, sanitizeText(item)]),
		) as T;
	return value;
}

function nonEmptyString(value: unknown): string | null {
	return typeof value === "string" && value.trim().length > 0 ? value : null;
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

export function normalizeBlock(block: Block): BlockViewModel {
	const sanitizedBlock = sanitizeText(block);
	const raw = parseRawJsonString(sanitizedBlock.rawJsonString);
	const rawLinks = normalizeLinks(
		getValue(raw, ["info", "links"]) ?? getValue(raw, ["links"]),
	);

	const metadata: Block = {
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
	return {
		...metadata,
		cards: normalizeBlockCardData(metadata, raw),
		hasGraphData: hasGraphData(metadata, raw),
		manifestId: typeof raw?.manifestId === "string" ? raw.manifestId : "",
		runtimeConfiguration: {
			metrics: Boolean(asRecord(raw?.metrics)?.provider),
			analytics: Boolean(raw?.analytics),
		},
		observations: {},
	};
}

export function normalizeBlocksData(data?: GetBlocksQuery): BlockViewModel[] {
	if (!data || !Array.isArray(data.blocks)) {
		return [];
	}

	return data.blocks.filter(Boolean).map(normalizeBlock);
}
