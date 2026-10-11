// @vitest-environment jsdom
import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getBlockCardData } from "@/cards/blockCardData";
import { normalizeBlock } from "./blockViewModel";
import type { GetBlocksQuery } from "./generated";
import { useBlocksData } from "./useBlocksData";
import type { RuntimeMetricSnapshot } from "./useDiagramMetrics";

const state = vi.hoisted(() => ({
	query: {
		data: undefined as GetBlocksQuery | undefined,
		error: null,
		isLoading: false,
	},
	preview: null as GetBlocksQuery["blocks"] | null,
	runtime: {
		data: undefined as
			| { diagramMetrics: Record<string, RuntimeMetricSnapshot> }
			| undefined,
		error: null as Error | null,
		isFetching: false,
	},
}));
vi.mock("./generated", () => ({ useGetBlocksQuery: () => state.query }));
vi.mock("./runtimePreviewStore", () => ({
	useRuntimePreviewBlocks: () => state.preview,
}));
vi.mock("./useDiagramMetrics", () => ({
	useDiagramMetrics: vi.fn(() => state.runtime),
}));

function block(raw: unknown = {}): GetBlocksQuery["blocks"][number] {
	return {
		name: "api…",
		description: " ",
		links: [],
		frameworks: [],
		connections: [],
		tags: [],
		docs: [],
		rawJsonString: JSON.stringify(raw),
	} as unknown as GetBlocksQuery["blocks"][number];
}
afterEach(() => {
	vi.restoreAllMocks();
	state.preview = null;
	state.runtime = { data: undefined, error: null, isFetching: false };
});

describe("block data boundary", () => {
	it.each(["{", "null", "[]", '"text"'])(
		"handles invalid manifest records: %s",
		(rawJsonString) => {
			const result = normalizeBlock({ ...block(), rawJsonString });
			expect(result.description).toBeNull();
			expect(result.observations).toEqual({});
			expect(result.cards.reports).toEqual([]);
		},
	);
	it("preserves metadata precedence, aliases, zeroes, and safe text", () => {
		const result = normalizeBlock({
			...block({
				info: {
					description: "worker—service",
					owner_team: "platform",
					max_replicas: 0,
					frameworks: ["react", 2],
					links: [{ site_name: "docs", http_url: "https://example.com" }],
				},
				repository: { stars: 0 },
			}),
			language: "Python",
		});
		expect(result).toMatchObject({
			name: "api...",
			description: "worker-service",
			ownerTeam: "platform",
			maxReplicas: 0,
			language: "Python",
			frameworks: ["react"],
			links: [{ siteName: "docs", httpUrl: "https://example.com" }],
		});
		expect(result.cards.github.stars).toBe(0);
	});
	it("parses once and keeps runtime updates separate from manifest and card data", () => {
		const input = block({
			manifestId: "api",
			metrics: { provider: "prometheus" },
			analytics: { provider: "posthog" },
		});
		state.query.data = { blocks: [input] };
		const parse = vi.spyOn(JSON, "parse");
		const { result, rerender } = renderHook(() => useBlocksData());
		const cards = result.current.blocks[0].cards;
		expect(result.current.blocks[0].observations.metrics?.status).toBe(
			"no_data",
		);
		state.runtime.isFetching = true;
		rerender();
		expect(result.current.blocks[0].observations.metrics?.status).toBe(
			"loading",
		);
		state.runtime.error = new Error("offline");
		rerender();
		expect(result.current.blocks[0].observations.metrics?.status).toBe("error");
		const snapshot: RuntimeMetricSnapshot = {
			status: "ready",
			values: {},
			analytics: { status: "partial", values: {} },
		};
		state.runtime.data = { diagramMetrics: { api: snapshot } };
		rerender();
		const view = result.current.blocks[0];
		expect(view.observations.metrics).toBe(snapshot);
		expect(getBlockCardData(view).runtimeAnalytics).toMatchObject({
			status: "partial",
			source: "posthog",
		});
		expect(view.cards).toBe(cards);
		expect(view.rawJsonString).toBe(input.rawJsonString);
		expect(
			parse.mock.calls.filter(([value]) => value === input.rawJsonString),
		).toHaveLength(1);
	});
	it("normalizes preview blocks and does not attach fetched observations", () => {
		state.preview = [
			block({
				info: { language: "Rust" },
				metrics: { provider: "prometheus" },
				github: { stars: 0 },
			}),
		];
		state.query.error = null;
		const { result } = renderHook(() => useBlocksData());
		expect(result.current.blocks[0].language).toBe("Rust");
		expect(result.current.blocks[0].cards.github.stars).toBe(0);
		expect(result.current.blocks[0].observations).toEqual({});
	});
});
