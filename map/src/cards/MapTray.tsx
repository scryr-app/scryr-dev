import {
	ArrowDown,
	ArrowLeft,
	ArrowRight,
	ArrowUp,
	Box,
	ChevronDown,
	ChevronLeft,
	ChevronRight,
	ChevronUp,
	Info,
	Layers,
	Map as MapIcon,
	Maximize2,
	Minus,
	Palette,
	PencilLine,
	Plus,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { cameraStore } from "@/camera";
import { Button } from "@/components/button";
import { ThemeOptions } from "@/theme/ThemeSwitcher";
import { cn } from "@/utils";
import { getFocusCardId, useCardFocus } from "./CardFocusContext";
import styles from "./MapTray.module.css";
import { useMapTray } from "./MapTrayContext";

const CARD_TYPES = [
	{ index: 0, icon: Info, label: "Info" },
	{ index: 1, icon: Layers, label: "Diagrams" },
] as const;

/**
 * Floating pill toolbar at the bottom of the screen.
 * Left section: card-type selectors.
 * Right section (after divider): theme palette picker.
 */
/** Returns mousedown/mouseup/mouseleave props that fire `action` immediately
 * then repeatedly at ~60 fps while the button is held.
 * `onLeave` is called on mouseup and mouseleave (e.g. to clear hover state). */
function holdAction(action: () => void, onLeave?: () => void) {
	let interval: ReturnType<typeof setInterval> | null = null;
	const stop = () => {
		if (interval !== null) {
			clearInterval(interval);
			interval = null;
		}
		window.removeEventListener("pointerup", stop);
		window.removeEventListener("pointercancel", stop);
		onLeave?.();
	};
	return {
		onPointerDown: (e: React.PointerEvent) => {
			e.preventDefault();
			e.currentTarget.setPointerCapture(e.pointerId);
			action();
			interval = setInterval(action, 16);
			window.addEventListener("pointerup", stop, { once: true });
			window.addEventListener("pointercancel", stop, { once: true });
		},
		onPointerUp: stop,
		onPointerCancel: stop,
	};
}

interface MapTrayProps {
	isPyodideOpen: boolean;
	onTogglePyodide: () => void;
}

export function MapTray({ isPyodideOpen, onTogglePyodide }: MapTrayProps) {
	const { activeCardIndex, toggleCard, selectedBlock, getActiveCardIndex } =
		useMapTray();
	const { cards, focusCard } = useCardFocus();
	const selectedCardId = selectedBlock
		? getFocusCardId(
				selectedBlock.name,
				getActiveCardIndex(selectedBlock.name) ?? 0,
			)
		: null;
	const focusTarget =
		cards.find((card) => card.id === selectedCardId) ??
		cards.find((card) => card.blockId === selectedBlock?.name);
	const [themeOpen, setThemeOpen] = useState(false);
	const [viewMenuOpen, setViewMenuOpen] = useState(false);
	const [viewMode, setViewMode] = useState<"top-down" | "isometric" | null>(
		"isometric",
	);
	const cameraAction = (action: () => void) => {
		setViewMode(null);
		action();
	};
	const [hoveredIndex, setHoveredIndex] = useState<
		| number
		| "editor"
		| "theme"
		| "zoom-in"
		| "zoom-out"
		| "pan-up"
		| "pan-down"
		| "pan-left"
		| "pan-right"
		| "rotate-up"
		| "rotate-down"
		| "rotate-left"
		| "rotate-right"
		| null
	>(null);
	const menuRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		if (!themeOpen) return;
		const onOutside = (e: MouseEvent) => {
			if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
				setThemeOpen(false);
			}
		};
		document.addEventListener("mousedown", onOutside);
		return () => document.removeEventListener("mousedown", onOutside);
	}, [themeOpen]);

	return (
		<div ref={menuRef} data-camera-occluder className={styles.tray}>
			{/* Theme dropdown — opens above the pill */}
			{themeOpen && (
				<div className={styles.themeMenu}>
					<ThemeOptions onSelect={() => setThemeOpen(false)} />
				</div>
			)}

			{/* Pill */}
			<div className={styles.pill}>
				<div className={styles.control}>
					{viewMenuOpen && (
						<div className={styles.viewMenu}>
							<button
								type="button"
								onClick={() => {
									setViewMode("top-down");
									cameraStore.setTopDownView();
									setViewMenuOpen(false);
								}}
								className={cn(
									styles.viewOption,
									viewMode === "top-down" && styles.selected,
								)}
							>
								<MapIcon size={14} /> Top Down
							</button>
							<button
								type="button"
								onClick={() => {
									setViewMode("isometric");
									cameraStore.setIsometricView();
									setViewMenuOpen(false);
								}}
								className={cn(
									styles.viewOption,
									viewMode === "isometric" && styles.selected,
								)}
							>
								<Box size={14} /> Isometric
							</button>
						</div>
					)}
					<Button
						type="button"
						variant="ghost"
						size="icon"
						className={cn(styles.viewButton, viewMode && styles.selected)}
						onClick={() => setViewMenuOpen((open) => !open)}
						aria-label="Select map view"
					>
						{viewMode === "top-down" ? (
							<MapIcon size={18} />
						) : (
							<Box size={18} />
						)}
					</Button>
				</div>

				{/* Rotate section — 4-quadrant circle */}
				<div className={styles.control}>
					{(hoveredIndex === "rotate-up" ||
						hoveredIndex === "rotate-down" ||
						hoveredIndex === "rotate-left" ||
						hoveredIndex === "rotate-right") && (
						<div className={styles.tooltip}>
							{hoveredIndex === "rotate-up"
								? "Rotate Up"
								: hoveredIndex === "rotate-down"
									? "Rotate Down"
									: hoveredIndex === "rotate-left"
										? "Rotate Left"
										: "Rotate Right"}
							<span className={styles.shortcut}>
								{hoveredIndex === "rotate-up"
									? "W"
									: hoveredIndex === "rotate-down"
										? "S"
										: hoveredIndex === "rotate-left"
											? "A"
											: "D"}
							</span>
						</div>
					)}
					<div className={styles.directionControl}>
						<button
							type="button"
							className={styles.upButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.rotateUp()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("rotate-up")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Rotate Up"
						>
							<ChevronUp size={10} />
						</button>
						<button
							type="button"
							className={styles.downButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.rotateDown()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("rotate-down")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Rotate Down"
						>
							<ChevronDown size={10} />
						</button>
						<button
							type="button"
							className={styles.leftButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.rotateLeft()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("rotate-left")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Rotate Left"
						>
							<ChevronLeft size={10} />
						</button>
						<button
							type="button"
							className={styles.rightButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.rotateRight()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("rotate-right")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Rotate Right"
						>
							<ChevronRight size={10} />
						</button>
					</div>
				</div>

				{/* Zoom section — top/bottom half split */}
				<div className={styles.control}>
					{(hoveredIndex === "zoom-in" || hoveredIndex === "zoom-out") && (
						<div className={styles.tooltip}>
							{hoveredIndex === "zoom-in" ? "Zoom In" : "Zoom Out"}
							<span className={styles.shortcut}>
								{hoveredIndex === "zoom-in" ? "E" : "Q"}
							</span>
						</div>
					)}
					<div className={styles.zoomControl}>
						<button
							type="button"
							className={styles.zoomInButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.zoomIn()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("zoom-in")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Zoom In"
						>
							<Plus size={11} />
						</button>
						<button
							type="button"
							className={styles.zoomOutButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.zoomOut()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("zoom-out")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Zoom Out"
						>
							<Minus size={11} />
						</button>
					</div>
				</div>

				{/* Pan section — 4-quadrant circle */}
				<div className={styles.control}>
					{(hoveredIndex === "pan-up" ||
						hoveredIndex === "pan-down" ||
						hoveredIndex === "pan-left" ||
						hoveredIndex === "pan-right") && (
						<div className={styles.tooltip}>
							{hoveredIndex === "pan-up"
								? "Pan Up"
								: hoveredIndex === "pan-down"
									? "Pan Down"
									: hoveredIndex === "pan-left"
										? "Pan Left"
										: "Pan Right"}
							<span className={styles.panShortcut}>
								<span className={styles.shiftKey}>⇧</span>
								{hoveredIndex === "pan-up"
									? "W"
									: hoveredIndex === "pan-down"
										? "S"
										: hoveredIndex === "pan-left"
											? "A"
											: "D"}
							</span>
						</div>
					)}
					<div className={styles.directionControl}>
						<button
							type="button"
							className={styles.upButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.panUp()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("pan-up")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Pan Up"
						>
							<ArrowUp size={10} />
						</button>
						<button
							type="button"
							className={styles.downButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.panDown()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("pan-down")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Pan Down"
						>
							<ArrowDown size={10} />
						</button>
						<button
							type="button"
							className={styles.leftButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.panLeft()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("pan-left")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Pan Left"
						>
							<ArrowLeft size={10} />
						</button>
						<button
							type="button"
							className={styles.rightButton}
							{...holdAction(
								() => cameraAction(() => cameraStore.panRight()),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("pan-right")}
							onMouseLeave={() => setHoveredIndex(null)}
							aria-label="Pan Right"
						>
							<ArrowRight size={10} />
						</button>
					</div>
				</div>

				{/* Vertical divider */}
				<div className={styles.divider} />

				{/* Info button — middle section */}
				{(() => {
					const { icon: Icon, label, index } = CARD_TYPES[0];
					return (
						<div className={styles.control}>
							{hoveredIndex === index && (
								<div className={styles.tooltip}>{label}</div>
							)}
							<Button
								type="button"
								variant="ghost"
								size="icon"
								className={cn(
									styles.trayButton,
									activeCardIndex === index && styles.selectedCard,
								)}
								aria-label={label}
								aria-pressed={activeCardIndex === index}
								onClick={() => toggleCard(index)}
								onMouseEnter={() => setHoveredIndex(index)}
								onMouseLeave={() => setHoveredIndex(null)}
							>
								<Icon size={18} />
							</Button>
						</div>
					);
				})()}

				{/* Remaining card type buttons */}
				{CARD_TYPES.slice(1).map(({ index, icon: Icon, label }) => (
					<div key={index} className={styles.control}>
						{hoveredIndex === index && (
							<div className={styles.tooltip}>{label}</div>
						)}
						<Button
							type="button"
							variant="ghost"
							size="icon"
							className={cn(
								styles.trayButton,
								activeCardIndex === index && styles.selectedCard,
							)}
							aria-label={label}
							aria-pressed={activeCardIndex === index}
							onClick={() => toggleCard(index)}
							onMouseEnter={() => setHoveredIndex(index)}
							onMouseLeave={() => setHoveredIndex(null)}
						>
							<Icon size={18} />
						</Button>
					</div>
				))}

				<Button
					type="button"
					variant="ghost"
					size="icon"
					className={styles.trayButton}
					aria-label="Focus selected card"
					title={
						focusTarget
							? "Focus selected card"
							: "Select a block to focus its card"
					}
					disabled={!focusTarget}
					onClick={() => {
						if (focusTarget) focusCard(focusTarget.id);
					}}
				>
					<Maximize2 size={18} />
				</Button>
				{/* Divider between card diagrams and the editor */}
				<div className={styles.divider} />

				{/* Theme section */}
				<div className={styles.control}>
					{hoveredIndex === "editor" && (
						<div className={styles.tooltip}>
							{isPyodideOpen ? "Hide Editor" : "Show Editor"}
						</div>
					)}
					<Button
						type="button"
						variant="ghost"
						size="icon"
						className={cn(styles.trayButton, isPyodideOpen && styles.selected)}
						onClick={onTogglePyodide}
						onMouseEnter={() => setHoveredIndex("editor")}
						onMouseLeave={() => setHoveredIndex(null)}
						aria-label={isPyodideOpen ? "Hide editor" : "Show editor"}
					>
						<PencilLine size={18} />
					</Button>
				</div>

				<div className={styles.control}>
					{hoveredIndex === "theme" && (
						<div className={styles.tooltip}>Style</div>
					)}
					<Button
						type="button"
						variant="ghost"
						size="icon"
						className={cn(styles.trayButton, themeOpen && styles.selected)}
						aria-label="Choose diagram theme"
						aria-expanded={themeOpen}
						onClick={() => setThemeOpen((o) => !o)}
						onMouseEnter={() => setHoveredIndex("theme")}
						onMouseLeave={() => setHoveredIndex(null)}
					>
						<Palette size={18} />
					</Button>
				</div>
			</div>
		</div>
	);
}
