import { useSyncExternalStore } from "react";
import type { GetBlocksQuery } from "@/graphql/generated";

export type RuntimePreviewBlock = GetBlocksQuery["blocks"][number];

function createRuntimePreviewStore() {
	let current: RuntimePreviewBlock[] | null = null;
	const listeners = new Set<() => void>();

	return {
		get: () => current,
		set: (blocks: RuntimePreviewBlock[] | null) => {
			current = blocks;
			for (const listener of listeners) {
				listener();
			}
		},
		subscribe: (listener: () => void) => {
			listeners.add(listener);
			return () => {
				listeners.delete(listener);
			};
		},
	};
}

export const runtimePreviewStore = createRuntimePreviewStore();

export function setRuntimePreviewBlocks(blocks: RuntimePreviewBlock[]) {
	runtimePreviewStore.set(blocks);
}

export function clearRuntimePreviewBlocks() {
	runtimePreviewStore.set(null);
}

export function useRuntimePreviewBlocks() {
	return useSyncExternalStore(
		runtimePreviewStore.subscribe,
		runtimePreviewStore.get,
	);
}
