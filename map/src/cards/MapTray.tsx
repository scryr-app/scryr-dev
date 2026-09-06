import {
	Activity,
	ArrowDown,
	ArrowLeft,
	ArrowRight,
	ArrowUp,
	ChevronDown,
	ChevronLeft,
	ChevronRight,
	ChevronUp,
	Gauge,
	GitFork,
	Info,
	Layers,
	Minus,
	Moon,
	Package,
	Palette,
	PencilLine,
	Plus,
	Rocket,
	Sun,
	TestTube,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { cameraStore } from "@/camera";
import { Button } from "@/components/button";
import { cn } from "@/utils";
import {
	getDiagramMode,
	setDiagramMode,
	setThemePreset,
	ThemePresets,
} from "../theme/theme";
import { useMapTray } from "./MapTrayContext";

const CARD_TYPES = [
	{ index: 0, icon: Info, label: "Info" },
	{ index: 1, icon: GitFork, label: "GitHub" },
	{ index: 2, icon: Activity, label: "Metrics" },
	{ index: 3, icon: Rocket, label: "CI/CD" },
	{ index: 4, icon: TestTube, label: "Tests" },
	{ index: 5, icon: Package, label: "Dependencies" },
	{ index: 6, icon: Gauge, label: "Performance" },
	{ index: 7, icon: Layers, label: "Diagrams" },
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
		onLeave?.();
	};
	return {
		onMouseDown: (e: React.MouseEvent) => {
			e.preventDefault();
			action();
			interval = setInterval(action, 16);
		},
		onMouseUp: stop,
		onMouseLeave: stop,
	};
}

interface MapTrayProps {
	isPyodideOpen: boolean;
	onTogglePyodide: () => void;
}

export function MapTray({ isPyodideOpen, onTogglePyodide }: MapTrayProps) {
	const { activeCardIndex, toggleCard } = useMapTray();
	const [themeOpen, setThemeOpen] = useState(false);
	const [hoveredIndex, setHoveredIndex] = useState<
		| number
		| "editor"
		| "theme"
		| "diagram-mode"
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
	const [diagramMode, setDiagramModeState] = useState<"light" | "dark">(() => {
		const stored = localStorage.getItem("diagramMode");
		return stored === "dark" ? "dark" : "light";
	});

	const getCurrentTheme = (): keyof typeof ThemePresets => {
		const stored = localStorage.getItem("selectedTheme");
		return stored && stored in ThemePresets
			? (stored as keyof typeof ThemePresets)
			: "AutumnOffice";
	};

	const handleThemeSelect = (name: keyof typeof ThemePresets) => {
		setThemeOpen(false);
		localStorage.setItem("selectedTheme", name);
		setThemePreset(name);
		setTimeout(() => window.location.reload(), 300);
	};

	useEffect(() => {
		const stored = localStorage.getItem("selectedTheme");
		if (stored && stored in ThemePresets) {
			setThemePreset(stored as keyof typeof ThemePresets);
		}
		setDiagramMode(getDiagramMode());
	}, []);

	const handleDiagramModeToggle = () => {
		const nextMode = diagramMode === "light" ? "dark" : "light";
		setDiagramModeState(nextMode);
		localStorage.setItem("diagramMode", nextMode);
		setDiagramMode(nextMode);
		setTimeout(() => window.location.reload(), 150);
	};

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

	const currentSelectedTheme = getCurrentTheme();

	return (
		<div
			ref={menuRef}
			style={{
				position: "fixed",
				bottom: "1.5rem",
				left: "50%",
				transform: "translateX(-50%)",
				zIndex: 1000,
			}}
		>
			{/* Theme dropdown — opens above the pill */}
			{themeOpen && (
				<div
					className="absolute bottom-full mb-2 right-0 bg-black/70 backdrop-blur-md border border-white/15 rounded-xl p-1.5 min-w-[160px] shadow-2xl"
					style={{ animation: "traySlideUp 0.15s ease-out" }}
				>
					<p className="text-[9px] font-semibold uppercase tracking-widest text-white/30 px-2 py-1">
						Theme
					</p>
					{Object.keys(ThemePresets).map((name) => {
						const key = name as keyof typeof ThemePresets;
						const isActive = key === currentSelectedTheme;
						return (
							<button
								type="button"
								key={name}
								onClick={() => handleThemeSelect(key)}
								className={cn(
									"w-full text-left px-2.5 py-1.5 rounded-lg text-xs transition-colors",
									isActive
										? "bg-white/20 text-white font-semibold"
										: "text-white/60 hover:bg-white/10 hover:text-white",
								)}
							>
								{name}
							</button>
						);
					})}
				</div>
			)}

			{/* Pill */}
			<div className="flex items-center gap-0.5 rounded-full px-1.5 py-1 bg-black/40 backdrop-blur-md border border-white/15 shadow-2xl">
				{/* Rotate section — 4-quadrant circle */}
				<div className="relative">
					{(hoveredIndex === "rotate-up" ||
						hoveredIndex === "rotate-down" ||
						hoveredIndex === "rotate-left" ||
						hoveredIndex === "rotate-right") && (
						<div
							className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2.5 py-1 rounded-md bg-black/70 backdrop-blur-sm border border-white/15 text-white text-xs font-medium whitespace-nowrap pointer-events-none flex items-center gap-1.5"
							style={{ animation: "traySlideUp 0.1s ease-out" }}
						>
							{hoveredIndex === "rotate-up"
								? "Rotate Up"
								: hoveredIndex === "rotate-down"
									? "Rotate Down"
									: hoveredIndex === "rotate-left"
										? "Rotate Left"
										: "Rotate Right"}
							<span className="opacity-50 text-[10px] font-mono border border-white/30 rounded px-1">
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
					<div className="relative w-10 h-10 overflow-hidden rounded-full">
						<button
							type="button"
							className="absolute inset-0 flex items-start justify-center pt-1 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							style={{ clipPath: "polygon(50% 50%, 0 0, 100% 0)" }}
							{...holdAction(
								() => cameraStore.rotateUp(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("rotate-up")}
						>
							<ChevronUp size={10} />
						</button>
						<button
							type="button"
							className="absolute inset-0 flex items-end justify-center pb-1 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							style={{ clipPath: "polygon(50% 50%, 100% 100%, 0 100%)" }}
							{...holdAction(
								() => cameraStore.rotateDown(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("rotate-down")}
						>
							<ChevronDown size={10} />
						</button>
						<button
							type="button"
							className="absolute inset-0 flex items-center justify-start pl-1 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							style={{ clipPath: "polygon(50% 50%, 0 0, 0 100%)" }}
							{...holdAction(
								() => cameraStore.rotateLeft(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("rotate-left")}
						>
							<ChevronLeft size={10} />
						</button>
						<button
							type="button"
							className="absolute inset-0 flex items-center justify-end pr-1 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							style={{ clipPath: "polygon(50% 50%, 100% 0, 100% 100%)" }}
							{...holdAction(
								() => cameraStore.rotateRight(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("rotate-right")}
						>
							<ChevronRight size={10} />
						</button>
					</div>
				</div>

				{/* Zoom section — top/bottom half split */}
				<div className="relative">
					{(hoveredIndex === "zoom-in" || hoveredIndex === "zoom-out") && (
						<div
							className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2.5 py-1 rounded-md bg-black/70 backdrop-blur-sm border border-white/15 text-white text-xs font-medium whitespace-nowrap pointer-events-none flex items-center gap-1.5"
							style={{ animation: "traySlideUp 0.1s ease-out" }}
						>
							{hoveredIndex === "zoom-in" ? "Zoom In" : "Zoom Out"}
							<span className="opacity-50 text-[10px] font-mono border border-white/30 rounded px-1">
								{hoveredIndex === "zoom-in" ? "E" : "Q"}
							</span>
						</div>
					)}
					<div className="flex flex-col w-10 h-10 overflow-hidden rounded-full">
						<button
							type="button"
							className="flex w-full h-1/2 items-end justify-center pb-0.5 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							{...holdAction(
								() => cameraStore.zoomIn(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("zoom-in")}
						>
							<Plus size={11} />
						</button>
						<button
							type="button"
							className="flex w-full h-1/2 items-start justify-center pt-0.5 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							{...holdAction(
								() => cameraStore.zoomOut(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("zoom-out")}
						>
							<Minus size={11} />
						</button>
					</div>
				</div>

				{/* Pan section — 4-quadrant circle */}
				<div className="relative">
					{(hoveredIndex === "pan-up" ||
						hoveredIndex === "pan-down" ||
						hoveredIndex === "pan-left" ||
						hoveredIndex === "pan-right") && (
						<div
							className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2.5 py-1 rounded-md bg-black/70 backdrop-blur-sm border border-white/15 text-white text-xs font-medium whitespace-nowrap pointer-events-none flex items-center gap-1.5"
							style={{ animation: "traySlideUp 0.1s ease-out" }}
						>
							{hoveredIndex === "pan-up"
								? "Pan Up"
								: hoveredIndex === "pan-down"
									? "Pan Down"
									: hoveredIndex === "pan-left"
										? "Pan Left"
										: "Pan Right"}
							<span className="opacity-50 text-[10px] font-mono border border-white/30 rounded px-1 flex items-center gap-0.5">
								<span className="text-[13px]">⇧</span>
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
					<div className="relative w-10 h-10 overflow-hidden rounded-full">
						<button
							type="button"
							className="absolute inset-0 flex items-start justify-center pt-1 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							style={{ clipPath: "polygon(50% 50%, 0 0, 100% 0)" }}
							{...holdAction(
								() => cameraStore.panUp(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("pan-up")}
						>
							<ArrowUp size={10} />
						</button>
						<button
							type="button"
							className="absolute inset-0 flex items-end justify-center pb-1 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							style={{ clipPath: "polygon(50% 50%, 100% 100%, 0 100%)" }}
							{...holdAction(
								() => cameraStore.panDown(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("pan-down")}
						>
							<ArrowDown size={10} />
						</button>
						<button
							type="button"
							className="absolute inset-0 flex items-center justify-start pl-1 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							style={{ clipPath: "polygon(50% 50%, 0 0, 0 100%)" }}
							{...holdAction(
								() => cameraStore.panLeft(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("pan-left")}
						>
							<ArrowLeft size={10} />
						</button>
						<button
							type="button"
							className="absolute inset-0 flex items-center justify-end pr-1 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150 cursor-pointer"
							style={{ clipPath: "polygon(50% 50%, 100% 0, 100% 100%)" }}
							{...holdAction(
								() => cameraStore.panRight(),
								() => setHoveredIndex(null),
							)}
							onMouseEnter={() => setHoveredIndex("pan-right")}
						>
							<ArrowRight size={10} />
						</button>
					</div>
				</div>

				{/* Vertical divider */}
				<div className="w-px h-6 bg-white/20 mx-1.5" />

				{/* Info button — middle section */}
				{(() => {
					const { icon: Icon, label, index } = CARD_TYPES[0];
					return (
						<div className="relative">
							{hoveredIndex === index && (
								<div
									className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2.5 py-1 rounded-md bg-black/70 backdrop-blur-sm border border-white/15 text-white text-xs font-medium whitespace-nowrap pointer-events-none flex items-center gap-1.5"
									style={{ animation: "traySlideUp 0.1s ease-out" }}
								>
									{label}
								</div>
							)}
							<Button
								type="button"
								variant="ghost"
								size="icon"
								className={cn(
									"rounded-full size-10 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150",
									activeCardIndex === index &&
										"bg-white/20 text-white shadow-inner",
								)}
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
					<div key={index} className="relative">
						{hoveredIndex === index && (
							<div
								className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2.5 py-1 rounded-md bg-black/70 backdrop-blur-sm border border-white/15 text-white text-xs font-medium whitespace-nowrap pointer-events-none flex items-center gap-1.5"
								style={{ animation: "traySlideUp 0.1s ease-out" }}
							>
								{label}
							</div>
						)}
						<Button
							type="button"
							variant="ghost"
							size="icon"
							className={cn(
								"rounded-full size-10 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150",
								activeCardIndex === index &&
									"bg-white/20 text-white shadow-inner",
							)}
							onClick={() => toggleCard(index)}
							onMouseEnter={() => setHoveredIndex(index)}
							onMouseLeave={() => setHoveredIndex(null)}
						>
							<Icon size={18} />
						</Button>
					</div>
				))}

				{/* Theme section */}
				<div className="relative">
					{hoveredIndex === "editor" && (
						<div
							className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2.5 py-1 rounded-md bg-black/70 backdrop-blur-sm border border-white/15 text-white text-xs font-medium whitespace-nowrap pointer-events-none"
							style={{ animation: "traySlideUp 0.1s ease-out" }}
						>
							{isPyodideOpen ? "Hide Editor" : "Show Editor"}
						</div>
					)}
					<Button
						type="button"
						variant="ghost"
						size="icon"
						className={cn(
							"rounded-full size-10 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150",
							isPyodideOpen && "bg-white/20 text-white",
						)}
						onClick={onTogglePyodide}
						onMouseEnter={() => setHoveredIndex("editor")}
						onMouseLeave={() => setHoveredIndex(null)}
						aria-label={isPyodideOpen ? "Hide editor" : "Show editor"}
					>
						<PencilLine size={18} />
					</Button>
				</div>

				<div className="relative">
					{hoveredIndex === "theme" && (
						<div
							className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2.5 py-1 rounded-md bg-black/70 backdrop-blur-sm border border-white/15 text-white text-xs font-medium whitespace-nowrap pointer-events-none"
							style={{ animation: "traySlideUp 0.1s ease-out" }}
						>
							Theme
						</div>
					)}
					<Button
						type="button"
						variant="ghost"
						size="icon"
						className={cn(
							"rounded-full size-10 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150",
							themeOpen && "bg-white/20 text-white",
						)}
						onClick={() => setThemeOpen((o) => !o)}
						onMouseEnter={() => setHoveredIndex("theme")}
						onMouseLeave={() => setHoveredIndex(null)}
					>
						<Palette size={18} />
					</Button>
				</div>

				<div className="relative">
					{hoveredIndex === "diagram-mode" && (
						<div
							className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2.5 py-1 rounded-md bg-black/70 backdrop-blur-sm border border-white/15 text-white text-xs font-medium whitespace-nowrap pointer-events-none"
							style={{ animation: "traySlideUp 0.1s ease-out" }}
						>
							{diagramMode === "light" ? "Dark Diagram" : "Light Diagram"}
						</div>
					)}
					<Button
						type="button"
						variant="ghost"
						size="icon"
						className={cn(
							"rounded-full size-10 text-white/50 hover:text-white hover:bg-white/15 transition-all duration-150",
							diagramMode === "dark" && "bg-white/20 text-white",
						)}
						onClick={handleDiagramModeToggle}
						onMouseEnter={() => setHoveredIndex("diagram-mode")}
						onMouseLeave={() => setHoveredIndex(null)}
						aria-label={
							diagramMode === "light"
								? "Switch diagram to dark mode"
								: "Switch diagram to light mode"
						}
					>
						{diagramMode === "light" ? <Moon size={18} /> : <Sun size={18} />}
					</Button>
				</div>
			</div>

			<style>{`
				@keyframes traySlideUp {
					from { opacity: 0; transform: translateY(6px); }
					to   { opacity: 1; transform: translateY(0); }
				}
			`}</style>
		</div>
	);
}
