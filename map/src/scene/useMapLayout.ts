import { useEffect, useState } from "react";
import type { Block } from "@/graphql/generated";
import { type LayoutResult, layoutBlocks } from "./layout";

export interface MapLayoutState {
	layout: LayoutResult | null;
	layoutError: Error | null;
}

export function useMapLayout(blocks: Block[]): MapLayoutState {
	const [layout, setLayout] = useState<LayoutResult | null>(null);
	const [layoutError, setLayoutError] = useState<Error | null>(null);

	useEffect(() => {
		let isCurrent = true;
		setLayout(null);
		setLayoutError(null);

		if (blocks.length === 0) {
			return () => {
				isCurrent = false;
			};
		}

		layoutBlocks(blocks)
			.then((nextLayout) => {
				if (isCurrent) {
					setLayout(nextLayout);
				}
			})
			.catch((err) => {
				console.error("Error during layout:", err);
				if (isCurrent) {
					setLayoutError(err instanceof Error ? err : new Error(String(err)));
				}
			});

		return () => {
			isCurrent = false;
		};
	}, [blocks]);

	return { layout, layoutError };
}
