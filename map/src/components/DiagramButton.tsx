import { FileCode2, FolderGit2, Layers } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import {
	type GetScryrMapsQuery,
	useGetScryrMapsQuery,
} from "@/graphql/generated";
import {
	getMapLabel,
	mapSelectionStore,
	type SelectedMap,
	useSelectedMap,
} from "@/graphql/sampleStore";

interface Props {
	/** Extra CSS classes applied to the trigger pill/button */
	className?: string;
}

type ScryrMapOption = GetScryrMapsQuery["scryrMaps"][number];

const DIAGRAM_SEARCH_PARAM = "diagram";

function diagramSourceLabel(map: ScryrMapOption): string {
	const folderPath = map.folderPath.trim().replace(/\/+$/, "");
	const fileName = map.fileName.trim();
	if (fileName === "index.scry") {
		return folderPath || map.key;
	}
	if (folderPath && fileName) {
		return `${folderPath}/${fileName}`;
	}
	return fileName || folderPath || map.key;
}

function diagramDisplayName(map: ScryrMapOption): string {
	return map.name.trim() || map.scryIdentifier || map.key;
}

function diagramUrlValue(map: ScryrMapOption): string {
	return map.scryIdentifier || map.id || map.key;
}

function selectedMapFromScryrMap(map: ScryrMapOption): SelectedMap {
	return {
		id: map.scryIdentifier || map.id,
		key: map.key,
	};
}

function selectedMapEquals(left: SelectedMap, right: SelectedMap): boolean {
	return left.id === right.id && left.key === right.key;
}

function currentDiagramParam(): string | null {
	return new URLSearchParams(window.location.search).get(DIAGRAM_SEARCH_PARAM);
}

function updateDiagramParam(value: string): void {
	const url = new URL(window.location.href);
	url.searchParams.set(DIAGRAM_SEARCH_PARAM, value);
	window.history.pushState({}, "", `${url.pathname}${url.search}${url.hash}`);
}

function matchDiagramParam(
	maps: ScryrMapOption[],
	value: string | null,
): ScryrMapOption | undefined {
	const normalizedValue = value?.trim();
	if (!normalizedValue) {
		return undefined;
	}

	return (
		maps.find((map) => map.scryIdentifier === normalizedValue) ??
		maps.find(
			(map) =>
				map.id === normalizedValue ||
				map.key === normalizedValue ||
				diagramDisplayName(map) === normalizedValue,
		)
	);
}

function fallbackSourceLabel(key: string): string {
	if (key.endsWith("/index.scry")) {
		return key.slice(0, -"/index.scry".length);
	}
	if (key === "index.scry") {
		return "";
	}
	return key;
}

function fallbackDisplayLabel(key: string): string {
	return getMapLabel(key);
}

function fallbackTitle(key: string): string {
	const sourceLabel = fallbackSourceLabel(key);
	return sourceLabel
		? `${fallbackDisplayLabel(key)} - ${sourceLabel}`
		: fallbackDisplayLabel(key);
}

function scryrMapTitle(map: ScryrMapOption): string {
	const sourceLabel = diagramSourceLabel(map);
	return sourceLabel
		? `${diagramDisplayName(map)} - ${sourceLabel}`
		: diagramDisplayName(map);
}

function visibleMapLabels(map: ScryrMapOption | { id: null; key: string }): {
	primary: string;
	secondary: string;
	title: string;
} {
	if ("folderPath" in map) {
		return {
			primary: diagramDisplayName(map),
			secondary: diagramSourceLabel(map),
			title: scryrMapTitle(map),
		};
	}

	return {
		primary: fallbackDisplayLabel(map.key),
		secondary: fallbackSourceLabel(map.key),
		title: fallbackTitle(map.key),
	};
}

function selectedMapTitle(
	map: ScryrMapOption | undefined,
	fallbackKey: string,
): string {
	if (map) {
		return scryrMapTitle(map);
	}
	return fallbackTitle(fallbackKey);
}

export function DiagramButton({ className = "" }: Props) {
	const current = useSelectedMap();
	const mapsQuery = useGetScryrMapsQuery();
	const maps = mapsQuery.data?.scryrMaps ?? [];
	const [open, setOpen] = useState(false);
	const panelRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		if (maps.length === 0) {
			return;
		}

		const urlMap = matchDiagramParam(maps, currentDiagramParam());
		if (urlMap) {
			const next = selectedMapFromScryrMap(urlMap);
			if (!selectedMapEquals(current, next)) {
				mapSelectionStore.set(next);
			}
			return;
		}

		if (
			current.id &&
			maps.some((map) => selectedMapFromScryrMap(map).id === current.id)
		) {
			return;
		}

		const matchingMap =
			maps.find((map) => map.key === current.key) ?? maps.at(0);
		if (!matchingMap) {
			return;
		}

		mapSelectionStore.set(selectedMapFromScryrMap(matchingMap));
	}, [current, maps]);

	useEffect(() => {
		function handlePopState() {
			const urlMap = matchDiagramParam(maps, currentDiagramParam());
			if (!urlMap) {
				return;
			}

			const next = selectedMapFromScryrMap(urlMap);
			if (!selectedMapEquals(mapSelectionStore.get(), next)) {
				mapSelectionStore.set(next);
			}
		}

		window.addEventListener("popstate", handlePopState);
		return () => window.removeEventListener("popstate", handlePopState);
	}, [maps]);

	// close on outside click
	useEffect(() => {
		if (!open) return;
		function handler(e: MouseEvent) {
			if (!panelRef.current?.contains(e.target as Node)) setOpen(false);
		}
		document.addEventListener("mousedown", handler);
		return () => document.removeEventListener("mousedown", handler);
	}, [open]);

	function select(map: SelectedMap) {
		mapSelectionStore.set(map);
		const selectedScryrMap = maps.find((scryrMap) =>
			selectedMapEquals(selectedMapFromScryrMap(scryrMap), map),
		);
		updateDiagramParam(
			selectedScryrMap ? diagramUrlValue(selectedScryrMap) : map.key,
		);
		setOpen(false);
	}

	const visibleMaps = maps;
	const mapsByRepository = visibleMaps.reduce((groups, map) => {
		const repository = map.folderPath.trim() || "Other repositories";
		const group = groups.get(repository) ?? [];
		group.push(map);
		groups.set(repository, group);
		return groups;
	}, new Map<string, ScryrMapOption[]>());
	const currentMap = maps.find((map) =>
		selectedMapEquals(selectedMapFromScryrMap(map), current),
	);
	const currentLabels = currentMap
		? visibleMapLabels(currentMap)
		: visibleMapLabels({ id: null, key: current.key });
	const currentTitle = selectedMapTitle(currentMap, current.key);

	return (
		<div className={`relative ${className}`} ref={panelRef}>
			{/* Trigger */}
			<button
				type="button"
				onClick={() => setOpen((v) => !v)}
				title={currentTitle}
				className="flex max-w-[min(42rem,calc(100vw-16rem))] items-center gap-2 text-white/60 hover:text-white/90 hover:bg-white/10 transition-colors rounded-full px-3 py-1.5 text-[13px]"
			>
				<Layers size={13} strokeWidth={2.2} className="shrink-0" />
				<span className="flex min-w-0 flex-col text-left leading-snug">
					<span className="truncate font-medium">{currentLabels.primary}</span>
					{currentLabels.secondary && (
						<span className="truncate text-[11px] text-white/40">
							{currentLabels.secondary}
						</span>
					)}
				</span>
			</button>

			{/* Dropdown panel */}
			{open && (
				<div
					className="absolute top-full left-0 mt-2 w-[min(28rem,calc(100vw-2rem))] rounded-xl border border-white/15 shadow-2xl z-[1100] overflow-hidden py-1"
					style={{ background: "rgba(0,0,0,0.88)" }}
				>
					<p className="text-[11px] text-white/40 font-medium uppercase tracking-widest px-4 pt-2 pb-1.5">
						Repositories
					</p>
					{mapsQuery.isLoading && (
						<div className="px-4 py-2 text-[13px] text-white/45">Loading</div>
					)}
					{Boolean(mapsQuery.error) && maps.length === 0 && (
						<div className="px-4 py-2 text-[12px] text-white/45">
							Unable to load diagrams
						</div>
					)}
					<div className="max-h-[min(28rem,calc(100vh-7rem))] overflow-y-auto">
						{Array.from(mapsByRepository.entries()).map(
							([repository, repositoryMaps]) => (
								<div key={repository}>
									<div className="flex items-center gap-2 border-t border-white/10 px-4 pb-1 pt-2 text-[10px] font-semibold uppercase tracking-widest text-white/45 first:border-t-0">
										<FolderGit2 size={13} className="text-white/50" />
										<span className="truncate">{repository}</span>
									</div>
									{repositoryMaps.map((map) => {
										const isSelected =
											(map.id && map.id === current.id) ||
											(!current.id && map.key === current.key);
										const labels = visibleMapLabels(map);

										return (
											<button
												key={map.id ?? map.key}
												type="button"
												onClick={() => select({ id: map.id, key: map.key })}
												title={labels.title}
												className={`w-full min-w-0 text-left px-4 py-2.5 text-[13px] transition-colors ${
													isSelected
														? "text-white bg-white/10"
														: "text-white/60 hover:text-white hover:bg-white/5"
												}`}
											>
												<div className="flex min-w-0 items-start gap-2 pl-5">
													<FileCode2
														size={14}
														className="mt-0.5 shrink-0 text-white/35"
													/>
													<span className="min-w-0">
														<span className="block whitespace-normal break-words font-medium leading-snug">
															{labels.primary}
														</span>
														{labels.secondary && (
															<span className="mt-0.5 block truncate text-[12px] text-white/40">
																{labels.secondary}
															</span>
														)}
													</span>
												</div>
											</button>
										);
									})}
								</div>
							),
						)}
					</div>
				</div>
			)}
		</div>
	);
}
