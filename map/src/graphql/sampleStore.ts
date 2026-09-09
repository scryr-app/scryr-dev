/**
 * Lightweight cross-boundary store for the selected Scryr map.
 * Works outside the React Three Fiber scene via useSyncExternalStore.
 */
import { useSyncExternalStore } from "react";
import { clearRuntimePreviewBlocks } from "@/graphql/runtimePreviewStore";

export const FALLBACK_MAP_KEYS = [
	"mern",
	"lamp",
	"calcom",
	"dokku",
	"mattermost",
	"open_saas",
	"otel_demo",
	"plane",
] as const;

export type FallbackMapKey = (typeof FALLBACK_MAP_KEYS)[number];

export const FALLBACK_MAP_LABELS: Record<FallbackMapKey, string> = {
	mern: "MERN",
	lamp: "LAMP",
	calcom: "Cal.com",
	dokku: "Dokku",
	mattermost: "Mattermost",
	open_saas: "OpenSaaS",
	otel_demo: "OTel Demo",
	plane: "Plane",
};

export interface SelectedMap {
	id: string | null;
	key: string;
}

export function getMapLabel(key: string): string {
	if (key in FALLBACK_MAP_LABELS) {
		return FALLBACK_MAP_LABELS[key as FallbackMapKey];
	}

	const fileName = key.split("/").filter(Boolean).at(-1);
	return fileName || "All Maps";
}

function createMapSelectionStore() {
	let current: SelectedMap = { id: null, key: "" };
	const listeners = new Set<() => void>();

	return {
		get: (): SelectedMap => current,
		set: (next: SelectedMap): void => {
			current = next;
			clearRuntimePreviewBlocks();
			for (const l of listeners) l();
		},
		subscribe: (l: () => void): (() => void) => {
			listeners.add(l);
			return () => {
				listeners.delete(l);
			};
		},
	};
}

export const mapSelectionStore = createMapSelectionStore();
export const sampleStore = {
	get: (): string => mapSelectionStore.get().key,
	set: (key: string): void => {
		mapSelectionStore.set({ id: null, key });
	},
};

/** React hook — triggers re-render whenever the selected map changes. */
export function useSelectedMap(): SelectedMap {
	return useSyncExternalStore(
		mapSelectionStore.subscribe,
		mapSelectionStore.get,
	);
}

/** React hook — retained for code paths that still need the artifact key. */
export function useSample(): string {
	return useSyncExternalStore(
		mapSelectionStore.subscribe,
		() => mapSelectionStore.get().key,
		() => mapSelectionStore.get().key,
	);
}
