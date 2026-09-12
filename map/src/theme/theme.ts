import { useSyncExternalStore } from "react";
import {
	industrialAppearance,
	neonAppearance,
	type ThemeAppearance,
} from "./appearance";

// Theme attributes interface
export interface ThemeAttributes {
	id: string;
	name: string;
	description: string;
	mode: DiagramMode;
	appearance: ThemeAppearance;
	// Block colors (10 named colors)
	dawn: string;
	dusk: string;
	tide: string;
	grove: string;
	pulse: string;
	flare: string;
	ember: string;
	mist: string;
	drift: string;
	slate: string;

	// Region colors (10 named regions)
	regionAlpha: string;
	regionBeta: string;
	regionGamma: string;
	regionDelta: string;
	regionEpsilon: string;
	regionZeta: string;
	regionEta: string;
	regionTheta: string;
	regionIota: string;
	regionKappa: string;

	// Diagram/UI properties
	backgroundColor: string;
	surfaceColor: string;
	fontFace: string;
	fontColor: string;
}

export type DiagramMode = "light" | "dark";

export class Theme {
	private attributes: ThemeAttributes;

	constructor(attributes: ThemeAttributes) {
		this.attributes = attributes;
	}

	get id() {
		return this.attributes.id;
	}
	get name() {
		return this.attributes.name;
	}
	get appearance() {
		return this.attributes.appearance;
	}

	// Basic colors
	get dawnColor(): string {
		return this.attributes.dawn;
	}

	get duskColor(): string {
		return this.attributes.dusk;
	}

	get tideColor(): string {
		return this.attributes.tide;
	}

	get groveColor(): string {
		return this.attributes.grove;
	}

	get pulseColor(): string {
		return this.attributes.pulse;
	}

	get flareColor(): string {
		return this.attributes.flare;
	}

	get emberColor(): string {
		return this.attributes.ember;
	}

	get mistColor(): string {
		return this.attributes.mist;
	}

	get driftColor(): string {
		return this.attributes.drift;
	}

	get slateColor(): string {
		return this.attributes.slate;
	}

	getColorByIndex(index: number): string {
		switch (index % 10) {
			case 9:
				return this.dawnColor;
			case 8:
				return this.duskColor;
			case 2:
				return this.tideColor;
			case 3:
				return this.groveColor;
			case 0:
				return this.pulseColor;
			case 5:
				return this.flareColor;
			case 6:
				return this.emberColor;
			case 7:
				return this.mistColor;
			case 1:
				return this.driftColor;
			case 4:
				return this.slateColor;
			default:
				return this.dawnColor;
		}
	}

	// Region colors
	get regionAlphaColor(): string {
		return this.attributes.regionAlpha;
	}

	get regionBetaColor(): string {
		return this.attributes.regionBeta;
	}

	get regionGammaColor(): string {
		return this.attributes.regionGamma;
	}

	get regionDeltaColor(): string {
		return this.attributes.regionDelta;
	}

	get regionEpsilonColor(): string {
		return this.attributes.regionEpsilon;
	}

	get regionZetaColor(): string {
		return this.attributes.regionZeta;
	}

	get regionEtaColor(): string {
		return this.attributes.regionEta;
	}

	get regionThetaColor(): string {
		return this.attributes.regionTheta;
	}

	get regionIotaColor(): string {
		return this.attributes.regionIota;
	}

	get regionKappaColor(): string {
		return this.attributes.regionKappa;
	}

	getRegionColorByIndex(index: number): string {
		switch (index % 10) {
			case 0:
				return this.regionAlphaColor;
			case 1:
				return this.regionBetaColor;
			case 2:
				return this.regionGammaColor;
			case 3:
				return this.regionDeltaColor;
			case 4:
				return this.regionEpsilonColor;
			case 5:
				return this.regionZetaColor;
			case 6:
				return this.regionEtaColor;
			case 7:
				return this.regionThetaColor;
			case 8:
				return this.regionIotaColor;
			case 9:
				return this.regionKappaColor;
			default:
				return this.regionAlphaColor;
		}
	}

	// Diagram basics
	get background(): string {
		return this.attributes.backgroundColor;
	}

	get surface(): string {
		return this.attributes.surfaceColor;
	}

	get fontFace(): string {
		return this.attributes.fontFace;
	}

	get fontColor(): string {
		return this.attributes.fontColor;
	}

	get connectionColor(): string {
		return this.appearance.connections.color;
	}

	get diagramMode(): DiagramMode {
		return this.attributes.mode;
	}

	get isDarkDiagram(): boolean {
		return this.attributes.mode === "dark";
	}

	// Additional derived colors for UI elements
	get cardTextColor(): string {
		return "#FFFFFF";
	}

	get cardHighlightColor(): string {
		return "#ffffff";
	}

	get innerWallColor(): string {
		return "#ffffff";
	}

	get errorColor(): string {
		return "#ff0000";
	}

	// Get contrasting text color based on background lightness
	getContrastingTextColor(backgroundColor: string): string {
		void backgroundColor;
		return "#FFFFFF";
	}

	toCSSVariables(): Record<string, string> {
		return {
			// Basic colors
			"--alpha": this.dawnColor,
			"--beta": this.duskColor,
			"--gamma": this.tideColor,
			"--delta": this.groveColor,
			"--epsilon": this.pulseColor,
			"--zeta": this.flareColor,
			"--eta": this.emberColor,
			"--theta": this.mistColor,
			"--iota": this.driftColor,
			"--kappa": this.slateColor,

			// Diagram basics
			"--background": this.background,
			"--surface": this.surface,
			"--font-face": this.fontFace,
			"--font-color": this.fontColor,
		};
	}
}

const originalPalette = {
	// Colors for blocks, warm autumn palette with rich earthy tones
	dawn: "#B5451B", // Deep brick red
	dusk: "#C47A3A", // Amber-brown
	tide: "#E8A630", // Golden harvest
	grove: "#6B8C3F", // Olive green
	pulse: "#D4652A", // Pumpkin orange
	flare: "#9E3B30", // Cranberry
	ember: "#CC5566", // Dusty rose
	mist: "#8A9E6A", // Sage green
	drift: "#5B8095", // Slate blue
	slate: "#6B574A", // Warm dark taupe (was near-white, now visible)

	// Region colors - warm light pastel tones
	regionAlpha: "#F5CDB4", // Light peach
	regionBeta: "#F2B89A", // Light coral
	regionGamma: "#EDD0A8", // Light apricot
	regionDelta: "#C8E6A0", // Light sage green
	regionEpsilon: "#F0DFC0", // Light caramel
	regionZeta: "#F2C9A0", // Light tan
	regionEta: "#F5B8B8", // Light rose
	regionTheta: "#EDB8C8", // Light mauve
	regionIota: "#B8E0D8", // Light sea green
	regionKappa: "#F5ECD0", // Light cream

	// Diagram basics
	backgroundColor: "#FFFBF5", // Warm cream
	surfaceColor: "#FFF0E6", // Light peach
	fontFace: "Inter, system-ui, sans-serif",
	fontColor: "#FFFFFF", // Pure white
};

export const ThemePresets = {
	IndustrialForest: {
		...originalPalette,
		id: "IndustrialForest",
		name: "Industrial Forest",
		description: "Textured blocks and soft daylight",
		mode: "light",
		appearance: industrialAppearance,
	},
	LightningNeon: {
		...originalPalette,
		id: "LightningNeon",
		name: "Lightning Neon",
		description: "Dark crystal and luminous edges",
		mode: "dark",
		appearance: neonAppearance,
		backgroundColor: "#080611",
		surfaceColor: "#111827",
	},
} satisfies Record<string, ThemeAttributes>;
export type ThemeId = keyof typeof ThemePresets;

/** Old palettes and brightness settings migrate into one complete theme. */
export function resolveThemeId(
	selected: string | null,
	legacyMode: string | null,
): ThemeId {
	if (selected && Object.hasOwn(ThemePresets, selected))
		return selected as ThemeId;
	if (legacyMode === "dark") return "LightningNeon";
	if (legacyMode === "light") return "IndustrialForest";
	return selected &&
		["SteelBlue", "IndustryOcean", "VibrantRainbow"].includes(selected)
		? "LightningNeon"
		: "IndustrialForest";
}
function readInitialTheme(): ThemeId {
	try {
		if (typeof window !== "undefined")
			return resolveThemeId(
				window.localStorage.getItem("selectedTheme"),
				window.localStorage.getItem("diagramMode"),
			);
	} catch {
		/* Storage may be disabled; the default still renders. */
	}
	return "IndustrialForest";
}
export let currentTheme = new Theme(ThemePresets[readInitialTheme()]);
const listeners = new Set<() => void>();
export function subscribeTheme(listener: () => void) {
	listeners.add(listener);
	return () => {
		listeners.delete(listener);
	};
}
export function getTheme() {
	return currentTheme;
}
export function useTheme() {
	return useSyncExternalStore(subscribeTheme, getTheme, getTheme);
}

export function setThemePreset(id: ThemeId): void {
	try {
		window.localStorage.setItem("selectedTheme", id);
		window.localStorage.removeItem("diagramMode");
	} catch {
		/* Theme switching also works without persistent storage. */
	}
	if (currentTheme.id === id) return;
	currentTheme = new Theme(ThemePresets[id]);
	for (const listener of listeners) listener();
}
