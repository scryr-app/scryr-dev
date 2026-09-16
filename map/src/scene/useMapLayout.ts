import { useEffect, useEffectEvent, useState } from "react";
import type { Block } from "@/graphql/generated";
import type { LayoutBlock, LayoutResult } from "./layout";
import { type LayoutView, layoutBlocksForView } from "./layoutOverview";

export interface MapLayoutState {
	layout: LayoutResult | null;
	layoutView: LayoutView | null;
	layoutError: Error | null;
}

export function useMapLayout(
	blocks: Block[],
	view: LayoutView,
): MapLayoutState {
	const [state, setState] = useState<MapLayoutState>({
		layout: null,
		layoutView: null,
		layoutError: null,
	});
	// Runtime metrics/card metadata refresh independently of the graph. They
	// must not rerun ELK or reset a camera the user has already positioned.
	const topology = JSON.stringify(
		blocks.map(({ name, connections, tags }) => ({ name, connections, tags })),
	);
	const {
		fov,
		position: [x, y, z],
	} = view;
	const getAspect = useEffectEvent(() => view.aspect);

	useEffect(() => {
		let isCurrent = true;
		const requestBlocks: LayoutBlock[] = JSON.parse(topology);
		// Capture the latest viewport when the graph/theme changes; resizing
		// alone does not rearrange blocks or reset navigation.
		const requestView: LayoutView = {
			position: [x, y, z],
			fov,
			aspect: getAspect(),
		};
		setState({ layout: null, layoutView: null, layoutError: null });
		if (requestBlocks.length > 0) {
			layoutBlocksForView(requestBlocks, requestView, () => isCurrent)
				.then((layout) => {
					if (isCurrent)
						setState({ layout, layoutView: requestView, layoutError: null });
				})
				.catch((err) => {
					if (isCurrent) {
						console.error("Error during layout:", err);
						setState({
							layout: null,
							layoutView: null,
							layoutError: err instanceof Error ? err : new Error(String(err)),
						});
					}
				});
		}
		return () => {
			isCurrent = false;
		};
	}, [topology, fov, x, y, z]);

	return state;
}
