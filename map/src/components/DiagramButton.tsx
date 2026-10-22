import { FileCode2, Folder, Layers } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { isLocalAuthMode } from "@/auth/env";
import { type GetScryrMapsQuery, useGetScryrMapsQuery } from "@/graphql/generated";
import {
	getMapLabel,
	mapSelectionStore,
	type SelectedMap,
	useSelectedMap,
} from "@/graphql/sampleStore";
import styles from "./DiagramButton.module.css";
import { Loading } from "./Loading";

interface Props {
	/** Extra CSS classes applied to the trigger pill/button */
	className?: string;
}

type ScryrMapOption = GetScryrMapsQuery["scryrMaps"][number];

const DIAGRAM_SEARCH_PARAM = "diagram";

function diagramGroup(map: ScryrMapOption): string {
	const folder = map.folderPath.trim().replaceAll("\\", "/").replace(/\/+$/, "");
	return !isLocalAuthMode && /^(?:manifest\/)?tests\/samples\/[^/]+$/.test(folder)
		? "Samples"
		: map.folderPath.trim();
}

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
	return sourceLabel ? `${fallbackDisplayLabel(key)} - ${sourceLabel}` : fallbackDisplayLabel(key);
}

function scryrMapTitle(map: ScryrMapOption): string {
	const sourceLabel = diagramSourceLabel(map);
	return sourceLabel ? `${diagramDisplayName(map)} - ${sourceLabel}` : diagramDisplayName(map);
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

function selectedMapTitle(map: ScryrMapOption | undefined, fallbackKey: string): string {
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

		if (current.id && maps.some((map) => selectedMapFromScryrMap(map).id === current.id)) {
			return;
		}

		const matchingMap =
			maps.find((map) => map.key === current.key) ??
			(!isLocalAuthMode
				? maps.find((map) => diagramGroup(map) === "Samples" && map.name === "MERN Customer Path")
				: undefined) ??
			maps.at(0);
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
		updateDiagramParam(selectedScryrMap ? diagramUrlValue(selectedScryrMap) : map.key);
		setOpen(false);
	}

	const visibleMaps = maps;
	const mapsByGroup = visibleMaps.reduce((groups, map) => {
		const groupName = diagramGroup(map);
		const group = groups.get(groupName) ?? [];
		group.push(map);
		groups.set(groupName, group);
		return groups;
	}, new Map<string, ScryrMapOption[]>());
	const orderedGroups = Array.from(mapsByGroup.entries()).sort(
		([left], [right]) => Number(Boolean(left)) - Number(Boolean(right)),
	);
	const currentMap = maps.find((map) => selectedMapEquals(selectedMapFromScryrMap(map), current));
	const currentLabels = currentMap
		? visibleMapLabels(currentMap)
		: visibleMapLabels({ id: null, key: current.key });
	const currentTitle = selectedMapTitle(currentMap, current.key);

	return (
		<div className={`${styles.container} ${className}`} ref={panelRef}>
			{/* Trigger */}
			<button
				type="button"
				onClick={() => setOpen((v) => !v)}
				title={currentTitle}
				aria-expanded={open}
				className={styles.trigger}
			>
				<Layers size={13} strokeWidth={2.2} className={styles.triggerIcon} />
				<span className={styles.triggerLabels}>
					<span className={styles.triggerPrimary}>{currentLabels.primary}</span>
					{currentLabels.secondary && (
						<span className={styles.triggerSecondary}>{currentLabels.secondary}</span>
					)}
				</span>
			</button>

			{/* Dropdown panel */}
			{open && (
				<div className={styles.panel} style={{ background: "rgba(0,0,0,0.88)" }}>
					{mapsQuery.isLoading && (
						<div className={styles.loading}>
							<Loading />
						</div>
					)}
					{Boolean(mapsQuery.error) && maps.length === 0 && (
						<div className={styles.error}>Unable to load diagrams</div>
					)}
					<div className={styles.list}>
						{orderedGroups.map(([groupName, groupMaps]) => (
							<div key={groupName}>
								{groupName && (
									<div className={styles.groupHeading}>
										<Folder size={13} className={styles.folderIcon} />
										<span className={styles.groupLabel}>{groupName}</span>
									</div>
								)}
								{groupMaps.map((map) => {
									const isSelected =
										(map.id && map.id === current.id) || (!current.id && map.key === current.key);
									const labels = visibleMapLabels(map);

									return (
										<button
											key={map.id ?? map.key}
											type="button"
											onClick={() => select({ id: map.id, key: map.key })}
											title={labels.title}
											className={`${styles.option} ${isSelected ? styles.selectedOption : ""}`}
										>
											<div
												className={`${styles.optionContent} ${groupName ? styles.groupedOption : ""}`}
											>
												<span className={styles.sourceIconContainer}>
													<FileCode2 size={16} className={styles.sourceIcon} aria-hidden="true" />
												</span>
												<span className={styles.optionLabels}>
													<span className={styles.optionPrimary}>{labels.primary}</span>
													{labels.secondary && (
														<span className={styles.optionSecondary}>{labels.secondary}</span>
													)}
												</span>
											</div>
										</button>
									);
								})}
							</div>
						))}
					</div>
				</div>
			)}
		</div>
	);
}
