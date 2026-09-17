// @vitest-environment jsdom
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { LayoutBlock, LayoutResult } from "./layout";
import { type LayoutView, layoutBlocksForView } from "./layoutOverview";
import { useMapLayout } from "./useMapLayout";

vi.mock("./layoutOverview", () => ({ layoutBlocksForView: vi.fn() }));
const layout: LayoutResult = {
	nodes: [],
	edges: [],
	groups: [],
	width: 100,
	height: 100,
};
const blocks: LayoutBlock[] = [
	{
		name: "a",
		connections: [],
		tags: [],
	},
];
const view: LayoutView = { position: [8, 6, 8], fov: 40, aspect: 16 / 9 };

beforeEach(() => {
	vi.mocked(layoutBlocksForView).mockReset().mockResolvedValue(layout);
});

describe("layout request lifecycle", () => {
	it("preserves the layout on metadata updates and resize, then uses current aspect for a new graph", async () => {
		const { result, rerender } = renderHook(
			({ blocks, view }) => useMapLayout(blocks, view),
			{ initialProps: { blocks, view } },
		);
		await waitFor(() => expect(result.current.layout).toBe(layout));
		const initialState = result.current;
		const portraitView = { ...view, aspect: 9 / 16 };
		rerender({
			blocks: blocks.map((block) => ({
				...block,
				rawJsonString: '{"metrics":[{"kind":"openmetrics","id":"http"}]}',
			})),
			view: portraitView,
		});
		expect(layoutBlocksForView).toHaveBeenCalledTimes(1);
		expect(result.current).toBe(initialState);
		rerender({ blocks: [{ ...blocks[0], name: "b" }], view: portraitView });
		await waitFor(() => expect(result.current.layoutView?.aspect).toBe(9 / 16));
		expect(layoutBlocksForView).toHaveBeenCalledTimes(2);
	});

	it("recalculates for camera preset changes", async () => {
		const { result, rerender } = renderHook(
			({ view }) => useMapLayout(blocks, view),
			{ initialProps: { view } },
		);
		await waitFor(() => expect(result.current.layout).toBe(layout));
		rerender({ view: { ...view, fov: 32, position: [10, 11, 16] } });
		await waitFor(() => expect(result.current.layoutView?.fov).toBe(32));
		expect(layoutBlocksForView).toHaveBeenCalledTimes(2);
	});

	it("ignores stale results and stops their candidate search", async () => {
		let resolveOld: (value: LayoutResult) => void = () => {};
		vi.mocked(layoutBlocksForView).mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveOld = resolve;
				}),
		);
		const { result, rerender } = renderHook(
			({ blocks }) => useMapLayout(blocks, view),
			{ initialProps: { blocks } },
		);
		const isOldCurrent = vi.mocked(layoutBlocksForView).mock.calls[0][2];
		rerender({ blocks: [{ ...blocks[0], name: "b" }] });
		await waitFor(() => expect(result.current.layout).toBe(layout));
		expect(isOldCurrent?.()).toBe(false);
		await act(async () => resolveOld({ ...layout, width: 999 }));
		expect(result.current.layout).toBe(layout);
	});
});
