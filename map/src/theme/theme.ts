import { useSyncExternalStore } from "react";
import { isLocalAuthMode } from "@/auth/env";
import {
	celestialAppearance,
	grimoireAppearance,
	industrialAppearance,
	neonAppearance,
	porcelainAppearance,
	sunkenAppearance,
	type ThemeAppearance,
	velvetAppearance,
} from "./appearance";
import {
	type ConsolePalette,
	celestialConsole,
	forestConsole,
	grimoireConsole,
	neonConsole,
	porcelainConsole,
	sunkenConsole,
	velvetConsole,
} from "./console";

// Theme attributes interface
export interface ThemeAttributes {
	id: string;
	name: string;
	description: string;
	mode: DiagramMode;
	appearance: ThemeAppearance;
	console: ConsolePalette;
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
	get console() {
		return this.attributes.console;
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
		return this.appearance.content?.text ?? "#FFFFFF";
	}

	get cardMutedTextColor() {
		return this.appearance.content?.muted ?? "rgba(255,255,255,0.40)";
	}
	get cardLinkColor() {
		return this.appearance.content?.link ?? "#93c5fd";
	}
	get cardInsetColor() {
		return this.appearance.content?.inset ?? "rgba(0,0,0,0.22)";
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
		console: forestConsole,
	},
	LightningNeon: {
		...originalPalette,
		id: "LightningNeon",
		name: "Luminous Crystal",
		description: "Dark crystal and luminous edges",
		mode: "dark",
		appearance: neonAppearance,
		console: neonConsole,
		backgroundColor: "#080611",
		surfaceColor: "#111827",
	},
	ScholarsGrimoire: {
		...originalPalette,
		id: "ScholarsGrimoire",
		name: "Scholarly Grimoire",
		description: "Parchment, leather and candlelight",
		mode: "dark",
		appearance: grimoireAppearance,
		console: grimoireConsole,
		backgroundColor: "#191611",
		surfaceColor: "#2b2118",
		fontColor: "#fff0cf",
		dawn: "#8a4c50",
		dusk: "#8a6649",
		tide: "#947846",
		grove: "#5c704f",
		pulse: "#985f4c",
		flare: "#96603d",
		ember: "#875361",
		mist: "#75815b",
		drift: "#4d727b",
		slate: "#75604b",
		regionAlpha: "#b79d70",
		regionBeta: "#ad987a",
		regionGamma: "#a8a28a",
		regionDelta: "#a2aa8a",
		regionEpsilon: "#c0a271",
		regionZeta: "#b39a77",
		regionEta: "#b09483",
		regionTheta: "#a99891",
		regionIota: "#94a399",
		regionKappa: "#c4b18b",
	},
	CelestialObservatory: {
		...originalPalette,
		id: "CelestialObservatory",
		name: "Celestial Observatory",
		description: "Brass instruments beneath a moving starfield",
		mode: "dark",
		appearance: celestialAppearance,
		console: celestialConsole,
		backgroundColor: "#030a18",
		surfaceColor: "#0b1830",
		fontColor: "#f5efd8",
		dawn: "#bd8550",
		dusk: "#d3a45f",
		tide: "#e0c170",
		grove: "#6389a7",
		pulse: "#ad8355",
		flare: "#9c5d72",
		ember: "#c78891",
		mist: "#88a2ae",
		drift: "#4e83b2",
		slate: "#70798b",
		regionAlpha: "#172b4c",
		regionBeta: "#1d3556",
		regionGamma: "#263b57",
		regionDelta: "#193f4e",
		regionEpsilon: "#3b3548",
		regionZeta: "#4b3d45",
		regionEta: "#373657",
		regionTheta: "#253b5b",
		regionIota: "#1f4558",
		regionKappa: "#403d52",
	},
	PorcelainReverie: {
		...originalPalette,
		id: "PorcelainReverie",
		name: "Porcelain Reverie",
		description: "Cobalt-painted porcelain with golden kintsugi",
		mode: "light",
		appearance: porcelainAppearance,
		console: porcelainConsole,
		backgroundColor: "#f8f4ea",
		surfaceColor: "#fffaf0",
		fontColor: "#183a67",
		dawn: "#315f9d",
		dusk: "#527aae",
		tide: "#c5a04f",
		grove: "#5e8292",
		pulse: "#24528d",
		flare: "#82617e",
		ember: "#b77975",
		mist: "#799b91",
		drift: "#49779c",
		slate: "#777c85",
		regionAlpha: "#e9edf1",
		regionBeta: "#e7e5e2",
		regionGamma: "#f1e8d8",
		regionDelta: "#e1ece7",
		regionEpsilon: "#f2e9d1",
		regionZeta: "#e8e1d7",
		regionEta: "#f0e0df",
		regionTheta: "#e8e0eb",
		regionIota: "#deebec",
		regionKappa: "#f1ecdf",
	},
	SunkenSanctuary: {
		...originalPalette,
		id: "SunkenSanctuary",
		name: "Sunken Sanctuary",
		description: "Ocean blues, sea-glass cards and rolling waves",
		mode: "dark",
		appearance: sunkenAppearance,
		console: sunkenConsole,
		backgroundColor: "#031d37",
		surfaceColor: "#084764",
		fontColor: "#e5f2e9",
		dawn: "#2995bc",
		dusk: "#427ac2",
		tide: "#8cddd9",
		grove: "#28a99e",
		pulse: "#39a9a0",
		flare: "#527dc7",
		ember: "#72cddc",
		mist: "#79bda7",
		drift: "#4f91bd",
		slate: "#7e82a8",
		regionAlpha: "#286f78",
		regionBeta: "#347f91",
		regionGamma: "#267b9e",
		regionDelta: "#2b8b78",
		regionEpsilon: "#388eaa",
		regionZeta: "#396e9f",
		regionEta: "#4e7eae",
		regionTheta: "#3f7896",
		regionIota: "#2c8e91",
		regionKappa: "#3996a0",
	},
	VelvetOracle: {
		...originalPalette,
		id: "VelvetOracle",
		name: "Velvet Oracle",
		description: "Champagne velvet, ethereal gold, and Greek sanctuary tones",
		mode: "light",
		appearance: velvetAppearance,
		console: velvetConsole,
		backgroundColor: "#f2f0e6",
		surfaceColor: "#faf8ef",
		fontColor: "#353b34",
		dawn: "#c2a568",
		dusk: "#8e845c",
		tide: "#658b99",
		grove: "#7e8b68",
		pulse: "#b69a58",
		flare: "#d0bc86",
		ember: "#a68e62",
		mist: "#a7b6ac",
		drift: "#7b91a0",
		slate: "#858c80",
		regionAlpha: "#e1d5b4",
		regionBeta: "#d5cfb6",
		regionGamma: "#c4d6d9",
		regionDelta: "#cfd6bd",
		regionEpsilon: "#e8ddbc",
		regionZeta: "#ded1ad",
		regionEta: "#d5cbb4",
		regionTheta: "#d3dfd5",
		regionIota: "#cbd6df",
		regionKappa: "#d6d9ce",
	},
} satisfies Record<string, ThemeAttributes>;
export type ThemeId = keyof typeof ThemePresets;
const defaultThemeId: ThemeId = isLocalAuthMode
	? "IndustrialForest"
	: "LightningNeon";

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
		: defaultThemeId;
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
	return defaultThemeId;
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
