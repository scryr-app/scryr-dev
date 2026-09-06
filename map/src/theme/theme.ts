// Theme attributes interface
export interface ThemeAttributes {
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

function resolveInitialDiagramMode(): DiagramMode {
	if (typeof window === "undefined") {
		return "light";
	}

	return window.localStorage.getItem("diagramMode") === "dark"
		? "dark"
		: "light";
}

export class Theme {
	private attributes: ThemeAttributes;

	constructor(attributes: ThemeAttributes) {
		this.attributes = attributes;
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
		return currentDiagramMode === "dark"
			? "#000000"
			: this.attributes.backgroundColor;
	}

	get surface(): string {
		return currentDiagramMode === "dark"
			? "#111827"
			: this.attributes.surfaceColor;
	}

	get fontFace(): string {
		return this.attributes.fontFace;
	}

	get fontColor(): string {
		return currentDiagramMode === "dark"
			? "#FFFFFF"
			: this.attributes.fontColor;
	}

	get connectionColor(): string {
		return currentDiagramMode === "dark" ? "#FFFFFF" : "#1F2937";
	}

	get diagramMode(): DiagramMode {
		return currentDiagramMode;
	}

	get isDarkDiagram(): boolean {
		return currentDiagramMode === "dark";
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

export const AutumnOfficeTheme: ThemeAttributes = {
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

export const SteelBlueTheme: ThemeAttributes = {
	// Dark modern tech palette with cool blues
	dawn: "#3B82F6", // Sky blue (was near-black, invisible on dark diagram)
	dusk: "#6366F1", // Indigo (was deep navy, invisible on dark diagram)
	tide: "#2563EB", // Cornflower blue
	grove: "#10B981", // Emerald
	pulse: "#06B6D4", // Cyan (more distinct from other blues)
	flare: "#F59E0B", // Amber
	ember: "#EF4444", // Red
	mist: "#A78BFA", // Lavender (distinct from blue group)
	drift: "#34D399", // Mint (distinct from grove)
	slate: "#94A3B8", // Steel gray

	// Region colors - light steel/sky pastel tones
	regionAlpha: "#A8C8E8", // Light steel blue
	regionBeta: "#9BB5D5", // Light slate
	regionGamma: "#8AC0E8", // Light ocean
	regionDelta: "#88D4A8", // Light emerald
	regionEpsilon: "#A8D8E0", // Light teal
	regionZeta: "#90C8F0", // Light sky
	regionEta: "#C8A8E0", // Light purple
	regionTheta: "#A8C0D8", // Light blue-gray
	regionIota: "#80C8E0", // Light petrol
	regionKappa: "#A8C0D4", // Light gunmetal

	// Diagram basics
	backgroundColor: "#0B1120", // Dark navy (distinct from block colors above)
	surfaceColor: "#1E293B", // Dark slate
	fontFace: "Inter, system-ui, sans-serif",
	fontColor: "#FFFFFF", // Pure white
};

export const SummerGazeboTheme: ThemeAttributes = {
	// Vibrant tropical light palette with neon accents
	dawn: "#FF4D88", // Hot pink
	dusk: "#9B59B6", // Vibrant purple
	tide: "#3A86FF", // Electric blue
	grove: "#00C875", // Vivid green
	pulse: "#FFB300", // Gold amber
	flare: "#FF5722", // Deep orange (distinct from dawn)
	ember: "#E91E8C", // Deep magenta (was duplicate of dawn, now distinct)
	mist: "#00BCD4", // Turquoise (was duplicate of flare, now cool complement)
	drift: "#76BC21", // Lime yellow-green (was duplicate of grove, now distinct)
	slate: "#7B6FA0", // Deep purple-slate (was near-white, now visible on cream)

	// Region colors - light tropical pastel shades
	regionAlpha: "#FFB3D1", // Light pink
	regionBeta: "#DDB8FF", // Light purple
	regionGamma: "#A8C8F8", // Light blue
	regionDelta: "#A0ECD8", // Light mint
	regionEpsilon: "#FFF0A8", // Light yellow
	regionZeta: "#FFD4A0", // Light orange
	regionEta: "#FFB8D4", // Light magenta
	regionTheta: "#FFD4B8", // Light coral
	regionIota: "#C0F4C0", // Light lime
	regionKappa: "#FFF0F8", // Pale pink

	// Diagram basics
	backgroundColor: "#FFFEF4", // Cream white
	surfaceColor: "#FFF8E7", // Pale yellow
	fontFace: "Inter, system-ui, sans-serif",
	fontColor: "#FFFFFF", // Pure white
};

export const IndustryOceanTheme: ThemeAttributes = {
	// Deep dark ocean/tech fusion palette
	dawn: "#164E63", // Deep teal
	dusk: "#0C4A6E", // Ocean depth
	tide: "#0369A1", // Dark sky blue
	grove: "#15803D", // Deep green
	pulse: "#F59E0B", // Warm amber
	flare: "#EA580C", // Deep orange
	ember: "#DC2626", // Deep red
	mist: "#7C3AED", // Purple accent
	drift: "#06B6D4", // Cyan
	slate: "#D1D5DB", // Light gray

	// Region colors - light ocean/industrial pastel tones
	regionAlpha: "#88C8D8", // Light teal
	regionBeta: "#80B8D0", // Light navy
	regionGamma: "#90C4E4", // Light steel blue
	regionDelta: "#88D0A8", // Light forest
	regionEpsilon: "#F0C870", // Light amber
	regionZeta: "#E0A888", // Light saddle
	regionEta: "#F08888", // Light red
	regionTheta: "#C0A8E0", // Light purple
	regionIota: "#80C8D8", // Light teal
	regionKappa: "#C0C8D0", // Light gray

	// Diagram basics
	backgroundColor: "#0F172A", // Almost black
	surfaceColor: "#1E293B", // Dark slate
	fontFace: "Inter, system-ui, sans-serif",
	fontColor: "#FFFFFF", // Pure white
};

export const ForestFactoryTheme: ThemeAttributes = {
	// Natural earthy light palette with forest tones
	dawn: "#7C2D12", // Dark oak
	dusk: "#92400E", // Brown wood
	tide: "#4B5563", // Slate gray
	grove: "#22C55E", // Fresh green
	pulse: "#FCD34D", // Golden sun
	flare: "#F97316", // Warm orange
	ember: "#DC2626", // Forest red
	mist: "#D4A574", // Sage tan
	drift: "#14B8A6", // Mint green
	slate: "#F5E6D3", // Cream

	// Region colors - light natural earthy tones
	regionAlpha: "#D4C0B0", // Light walnut
	regionBeta: "#D8BCA8", // Light cedar
	regionGamma: "#B8C8D8", // Light storm
	regionDelta: "#98D898", // Light moss
	regionEpsilon: "#DDD0B8", // Light caramel
	regionZeta: "#EDD888", // Light goldenrod
	regionEta: "#DDB0A0", // Light rust
	regionTheta: "#D8C8B0", // Light taupe
	regionIota: "#A0C8A8", // Light sage
	regionKappa: "#E8D4B8", // Light brown

	// Diagram basics
	backgroundColor: "#FEFCE8", // Soft cream
	surfaceColor: "#F5F3FF", // Off-white
	fontFace: "Inter, system-ui, sans-serif",
	fontColor: "#FFFFFF", // Pure white
};

export const VibrantRainbowTheme: ThemeAttributes = {
	// Maximum contrast dark rainbow neon palette
	dawn: "#FF2D55", // Rose red
	dusk: "#FF6B00", // Vibrant orange (was duplicate of dawn, now distinct)
	tide: "#0080FF", // Royal blue
	grove: "#00C85A", // Vivid green
	pulse: "#FFD700", // Gold yellow
	flare: "#FF8000", // Neon orange
	ember: "#FF0000", // Pure red
	mist: "#BF00FF", // Violet (was magenta, now distinct purple hue)
	drift: "#00FFFF", // Cyan
	slate: "#E040FB", // Magenta-pink (was white; covers pink spectrum distinctly)

	// Region colors - light pastel neon tones
	regionAlpha: "#FF88CC", // Light magenta
	regionBeta: "#FFB888", // Light orange
	regionGamma: "#88C4FF", // Light blue
	regionDelta: "#88FFB8", // Light green
	regionEpsilon: "#FFFF88", // Light yellow
	regionZeta: "#FF9898", // Light red
	regionEta: "#CC88FF", // Light purple
	regionTheta: "#FF88FF", // Light magenta
	regionIota: "#88FFFF", // Light cyan
	regionKappa: "#CCCCCC", // Light gray

	// Diagram basics
	backgroundColor: "#0D0D0D", // Very dark (slightly off pure black)
	surfaceColor: "#1A1A2E", // Dark navy tint
	fontFace: "Inter, system-ui, sans-serif",
	fontColor: "#FFFFFF", // Pure white
};

// Active theme instance - this is the single source of truth
let activeTheme = new Theme(AutumnOfficeTheme);
let currentDiagramMode: DiagramMode = resolveInitialDiagramMode();

// Export the current theme
export const currentTheme = activeTheme;

export function getDiagramMode(): DiagramMode {
	return currentDiagramMode;
}

// Available theme presets
export const ThemePresets = {
	AutumnOffice: AutumnOfficeTheme,
	SteelBlue: SteelBlueTheme,
	SummerGazebo: SummerGazeboTheme,
	IndustryOcean: IndustryOceanTheme,
	ForestFactory: ForestFactoryTheme,
	VibrantRainbow: VibrantRainbowTheme,
} as const;

// Function to change the active theme
export function setTheme(themeAttributes: ThemeAttributes): void {
	activeTheme = new Theme(themeAttributes);
	Object.assign(currentTheme, activeTheme);
}

export function setDiagramMode(mode: DiagramMode): void {
	currentDiagramMode = mode;
	Object.assign(currentTheme, activeTheme);
}

// Convenience function to set theme by preset name
export function setThemePreset(presetName: keyof typeof ThemePresets): void {
	setTheme(ThemePresets[presetName]);
}
