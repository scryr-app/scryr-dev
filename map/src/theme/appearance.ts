/** Complete scene styling, independent of the palette and light/dark UI mode. */
export interface SurfaceStyle {
	metalness: number;
	roughness: number;
	clearcoat: number;
	clearcoatRoughness: number;
	transmission: number;
	envMapIntensity: number;
	emissiveIntensity: number;
}
export interface SceneLight {
	position: [number, number, number];
	color: string;
	intensity: number;
}
export interface ThemeAppearance {
	textures: {
		wall:
			| "ribbed"
			| "crystal"
			| "leather"
			| "celestial"
			| "porcelain"
			| "sunkenStone"
			| "velvet";
		card?: "parchment" | "porcelain" | "seaGlass" | "oracle";
		paperColor?: string;
		wallBrightness: number;
		glowCore: number;
		glowRim: number;
	};
	content?: { text: string; muted: string; link: string; inset: string };
	shapes: {
		flatFaces: boolean;
		cardRadius: number;
		frameFront: number;
		frameBack: number;
		frameCard: number;
	};
	walls: SurfaceStyle;
	innerWalls: SurfaceStyle;
	cards: SurfaceStyle & {
		brightness: number;
		useBlockColor: boolean;
		overlayOpacity: number;
		rimHighlight: number;
	};
	lighting: {
		ambient: number;
		hemisphere: { sky: string; ground: string; intensity: number };
		key: SceneLight;
		fill: SceneLight[];
		reflections: (SceneLight & { scale: [number, number, number] })[];
		motion?: "orbit" | "caustic";
	};
	floor: {
		pattern?: "wood" | "stars" | "porcelain" | "seabed" | "velvet";
		tiles: [string, string];
		opacity: number;
		roughness: number;
		metalness: number;
		gridMajor: string;
		gridMinor: string;
		gridOpacity: number;
	};
	regions: {
		tint: string | null;
		signColor: string | null;
		labelColor: string | null;
		roughness: number;
		frame: number;
	};
	connections: { color: string; luminous: boolean; haloOpacity: number };
	view: { fov: number; position: [number, number, number] };
}

const matte: SurfaceStyle = {
	metalness: 0.24,
	roughness: 0.52,
	clearcoat: 0,
	clearcoatRoughness: 0,
	transmission: 0,
	envMapIntensity: 0,
	emissiveIntensity: 0,
};

export const industrialAppearance: ThemeAppearance = {
	textures: { wall: "ribbed", wallBrightness: 1, glowCore: 0, glowRim: 0 },
	shapes: {
		flatFaces: false,
		cardRadius: 0.0125,
		frameFront: 0,
		frameBack: 0,
		frameCard: 0,
	},
	walls: { ...matte },
	innerWalls: { ...matte, metalness: 0.16, roughness: 0.62 },
	cards: {
		...matte,
		metalness: 0.16,
		brightness: 1,
		useBlockColor: false,
		overlayOpacity: 0.12,
		rimHighlight: 0.22,
	},
	lighting: {
		ambient: 0.4,
		hemisphere: { sky: "#ffffff", ground: "#ffffff", intensity: 0 },
		key: { position: [12, 18, 10], color: "#ffffff", intensity: 1.1 },
		fill: [],
		reflections: [],
	},
	floor: {
		tiles: ["rgba(255,255,255,0.72)", "rgba(232,233,237,0.72)"],
		opacity: 0.42,
		roughness: 1,
		metalness: 0,
		gridMajor: "#dbeafe",
		gridMinor: "#f8fafc",
		gridOpacity: 0.16,
	},
	regions: {
		tint: null,
		signColor: null,
		labelColor: null,
		roughness: 0.82,
		frame: 0,
	},
	connections: { color: "#1f2937", luminous: false, haloOpacity: 0 },
	view: { fov: 40, position: [8, 6, 8] },
};

export const neonAppearance: ThemeAppearance = {
	textures: {
		wall: "crystal",
		wallBrightness: 0.16,
		glowCore: 0.16,
		glowRim: 0.8,
	},
	shapes: {
		flatFaces: true,
		cardRadius: 0.0125,
		frameFront: 0.65,
		frameBack: 0.4,
		frameCard: 0.6,
	},
	walls: {
		metalness: 0.18,
		roughness: 0.38,
		clearcoat: 0.35,
		clearcoatRoughness: 0.48,
		transmission: 0.03,
		envMapIntensity: 0.2,
		emissiveIntensity: 0.42,
	},
	innerWalls: {
		...matte,
		metalness: 0.02,
		roughness: 0.55,
		clearcoat: 0.15,
		emissiveIntensity: 0.12,
	},
	cards: {
		...matte,
		metalness: 0.03,
		roughness: 0.4,
		clearcoat: 0.32,
		clearcoatRoughness: 0.5,
		envMapIntensity: 0.18,
		emissiveIntensity: 0.18,
		brightness: 0.12,
		useBlockColor: true,
		overlayOpacity: 0.04,
		rimHighlight: 0,
	},
	lighting: {
		ambient: 0.18,
		hemisphere: { sky: "#ddd6ff", ground: "#44365c", intensity: 0.35 },
		key: { position: [12, 18, 10], color: "#f1d7b2", intensity: 0.5 },
		fill: [
			{ position: [-10, 8, -8], color: "#b7a1ff", intensity: 0.45 },
			{ position: [4, 5, -12], color: "#a9e7ff", intensity: 0.3 },
		],
		reflections: [
			{
				position: [0, 8, 2],
				scale: [10, 3, 1],
				color: "#e9e1ff",
				intensity: 0.8,
			},
			{
				position: [-6, 3, -4],
				scale: [3, 8, 1],
				color: "#b5caff",
				intensity: 1,
			},
			{
				position: [6, 4, 3],
				scale: [2, 6, 1],
				color: "#ffebd6",
				intensity: 0.5,
			},
		],
	},
	floor: {
		tiles: ["#12101c", "#14111f"],
		opacity: 0.92,
		roughness: 0.65,
		metalness: 0.18,
		gridMajor: "#826b9b",
		gridMinor: "#665176",
		gridOpacity: 0.13,
	},
	regions: {
		tint: "#30243e",
		signColor: "#21172d",
		labelColor: "#e0cfef",
		roughness: 0.6,
		frame: 0.18,
	},
	connections: { color: "#b974ff", luminous: true, haloOpacity: 0.1 },
	view: { fov: 32, position: [10, 11, 16] },
};

/** A candlelit reading room: leather bindings, aged paper and antique brass. */
export const grimoireAppearance: ThemeAppearance = {
	textures: {
		wall: "leather",
		wallBrightness: 1,
		card: "parchment",
		glowCore: 0,
		glowRim: 0,
	},
	content: {
		text: "#fff5df",
		muted: "#e1d2b5",
		link: "#d9c8ff",
		inset: "rgba(20,14,9,0.16)",
	},
	shapes: {
		flatFaces: true,
		cardRadius: 0.007,
		frameFront: 0,
		frameBack: 0,
		frameCard: 0,
	},
	walls: {
		...matte,
		metalness: 0.08,
		roughness: 0.86,
		clearcoat: 0.08,
		clearcoatRoughness: 0.7,
		envMapIntensity: 0.25,
	},
	innerWalls: { ...matte, metalness: 0, roughness: 0.95 },
	cards: {
		...matte,
		metalness: 0,
		roughness: 0.96,
		brightness: 1,
		useBlockColor: true,
		overlayOpacity: 0,
		rimHighlight: 0,
	},
	lighting: {
		ambient: 0.7,
		hemisphere: { sky: "#f5e8cd", ground: "#483a2c", intensity: 0.7 },
		key: { position: [-5, 12, 9], color: "#ffe9c5", intensity: 1.5 },
		fill: [
			{ position: [8, 6, -10], color: "#b6c4cd", intensity: 0.35 },
			{ position: [5, 6, 12], color: "#fff1d9", intensity: 0.5 },
		],
		reflections: [
			{
				position: [-4, 6, 5],
				scale: [3, 5, 1],
				color: "#e8af61",
				intensity: 0.6,
			},
		],
	},
	floor: {
		pattern: "wood",
		tiles: ["#302319", "#3b2b1f"],
		opacity: 1,
		roughness: 0.8,
		metalness: 0.05,
		gridMajor: "#a58b57",
		gridMinor: "#796746",
		gridOpacity: 0.055,
	},
	regions: {
		tint: "#66543b",
		signColor: "#302218",
		labelColor: "#dfc797",
		roughness: 0.94,
		frame: 0,
	},
	connections: { color: "#d4b276", luminous: false, haloOpacity: 0 },
	view: { fov: 35, position: [10, 12, 17] },
};

/** Midnight observatory instruments in blue lacquer and brushed brass. */
export const celestialAppearance: ThemeAppearance = {
	textures: {
		wall: "celestial",
		wallBrightness: 0.9,
		glowCore: 0.035,
		glowRim: 0.16,
	},
	content: {
		text: "#f5f0d8",
		muted: "#b9c4d8",
		link: "#f0cf86",
		inset: "rgba(3,10,30,0.35)",
	},
	shapes: {
		flatFaces: true,
		cardRadius: 0.006,
		frameFront: 0.18,
		frameBack: 0.08,
		frameCard: 0.12,
	},
	walls: {
		...matte,
		metalness: 0.62,
		roughness: 0.34,
		clearcoat: 0.24,
		clearcoatRoughness: 0.45,
		envMapIntensity: 0.7,
		emissiveIntensity: 0.11,
	},
	innerWalls: { ...matte, metalness: 0.58, roughness: 0.4 },
	cards: {
		...matte,
		metalness: 0.34,
		roughness: 0.4,
		clearcoat: 0.18,
		envMapIntensity: 0.55,
		emissiveIntensity: 0.04,
		brightness: 0.44,
		useBlockColor: true,
		overlayOpacity: 0.12,
		rimHighlight: 0.42,
	},
	lighting: {
		ambient: 0.24,
		hemisphere: { sky: "#7f9dce", ground: "#070d1b", intensity: 0.45 },
		key: { position: [10, 16, 8], color: "#fff0bd", intensity: 1.15 },
		fill: [
			{ position: [-9, 7, -8], color: "#789bd8", intensity: 0.5 },
			{ position: [4, 4, -12], color: "#d3b46f", intensity: 0.22 },
		],
		reflections: [
			{
				position: [0, 9, 1],
				scale: [8, 2, 1],
				color: "#f4d590",
				intensity: 1.1,
			},
		],
		motion: "orbit",
	},
	floor: {
		pattern: "stars",
		tiles: ["#071329", "#091831"],
		opacity: 1,
		roughness: 0.58,
		metalness: 0.24,
		gridMajor: "#b89a5d",
		gridMinor: "#526582",
		gridOpacity: 0.13,
	},
	regions: {
		tint: "#152647",
		signColor: "#8f713c",
		labelColor: "#fff1c7",
		roughness: 0.48,
		frame: 0.08,
	},
	connections: { color: "#c9aa68", luminous: true, haloOpacity: 0.035 },
	view: { fov: 34, position: [11, 12, 17] },
};

/** Lightly glazed ivory porcelain with cobalt painting and gold kintsugi. */
export const porcelainAppearance: ThemeAppearance = {
	textures: {
		wall: "porcelain",
		wallBrightness: 1.04,
		card: "porcelain",
		paperColor: "#fffaf0",
		glowCore: 0,
		glowRim: 0,
	},
	content: {
		text: "#183a67",
		muted: "#68758a",
		link: "#205fa4",
		inset: "rgba(22,58,103,0.07)",
	},
	shapes: {
		flatFaces: false,
		cardRadius: 0.025,
		frameFront: 0,
		frameBack: 0,
		frameCard: 0,
	},
	walls: {
		...matte,
		metalness: 0.02,
		roughness: 0.22,
		clearcoat: 0.82,
		clearcoatRoughness: 0.18,
		envMapIntensity: 0.65,
	},
	innerWalls: {
		...matte,
		metalness: 0,
		roughness: 0.3,
		clearcoat: 0.6,
		clearcoatRoughness: 0.22,
	},
	cards: {
		...matte,
		metalness: 0.01,
		roughness: 0.2,
		clearcoat: 0.9,
		clearcoatRoughness: 0.15,
		envMapIntensity: 0.6,
		brightness: 1,
		useBlockColor: false,
		overlayOpacity: 0.08,
		rimHighlight: 0.25,
	},
	lighting: {
		ambient: 0.68,
		hemisphere: { sky: "#fffdf5", ground: "#c8d5df", intensity: 0.8 },
		key: { position: [10, 17, 9], color: "#fff8e9", intensity: 1.25 },
		fill: [{ position: [-8, 8, -6], color: "#c7dcf2", intensity: 0.45 }],
		reflections: [
			{
				position: [-2, 8, 4],
				scale: [8, 4, 1],
				color: "#ffffff",
				intensity: 1.2,
			},
		],
	},
	floor: {
		pattern: "porcelain",
		tiles: ["#f8f3e8", "#eef1ee"],
		opacity: 0.96,
		roughness: 0.32,
		metalness: 0,
		gridMajor: "#315c91",
		gridMinor: "#d8bd78",
		gridOpacity: 0.09,
	},
	regions: {
		tint: "#f2eadc",
		signColor: "#174f91",
		labelColor: "#fff9e8",
		roughness: 0.28,
		frame: 0,
	},
	connections: { color: "#b4934f", luminous: false, haloOpacity: 0 },
	view: { fov: 38, position: [9, 8, 13] },
};

/** Submerged carved stone, oxidized copper and translucent sea glass. */
export const sunkenAppearance: ThemeAppearance = {
	textures: {
		wall: "sunkenStone",
		wallBrightness: 0.76,
		card: "seaGlass",
		glowCore: 0.06,
		glowRim: 0.36,
	},
	content: {
		text: "#e7f5ec",
		muted: "#9dbfb8",
		link: "#a9e0d8",
		inset: "rgba(2,24,29,0.3)",
	},
	shapes: {
		flatFaces: true,
		cardRadius: 0.016,
		frameFront: 0.12,
		frameBack: 0.04,
		frameCard: 0.18,
	},
	walls: {
		...matte,
		metalness: 0.08,
		roughness: 0.82,
		clearcoat: 0.12,
		clearcoatRoughness: 0.72,
		envMapIntensity: 0.42,
		emissiveIntensity: 0.08,
	},
	innerWalls: {
		...matte,
		metalness: 0.24,
		roughness: 0.7,
		emissiveIntensity: 0.05,
	},
	cards: {
		...matte,
		metalness: 0.05,
		roughness: 0.2,
		clearcoat: 0.72,
		clearcoatRoughness: 0.18,
		transmission: 0.12,
		envMapIntensity: 0.75,
		emissiveIntensity: 0.12,
		brightness: 0.4,
		useBlockColor: true,
		overlayOpacity: 0.08,
		rimHighlight: 0.34,
	},
	lighting: {
		ambient: 0.32,
		hemisphere: { sky: "#8ee3da", ground: "#05242c", intensity: 0.72 },
		key: { position: [3, 18, 5], color: "#c8fff0", intensity: 1.2 },
		fill: [
			{ position: [-10, 5, -6], color: "#42b9bd", intensity: 0.68 },
			{ position: [10, 3, 10], color: "#e1b8a0", intensity: 0.34 },
			{ position: [2, 6, -12], color: "#648fd1", intensity: 0.3 },
		],
		reflections: [
			{
				position: [0, 9, 0],
				scale: [10, 3, 1],
				color: "#a9eee2",
				intensity: 1.08,
			},
		],
		motion: "caustic",
	},
	floor: {
		pattern: "seabed",
		tiles: ["#0d3e46", "#124b4d"],
		opacity: 1,
		roughness: 0.88,
		metalness: 0.05,
		gridMajor: "#83c7b8",
		gridMinor: "#3b7471",
		gridOpacity: 0.1,
	},
	regions: {
		tint: null,
		signColor: null,
		labelColor: "#e8e0c7",
		roughness: 0.62,
		frame: 0.08,
	},
	connections: { color: "#79c9bd", luminous: true, haloOpacity: 0.055 },
	view: { fov: 36, position: [11, 10, 16] },
};

/** Plush layered divination cards with embossed champagne-gold symbols. */
export const velvetAppearance: ThemeAppearance = {
	textures: {
		wall: "velvet",
		wallBrightness: 0.72,
		card: "oracle",
		paperColor: "#f8ead9",
		glowCore: 0,
		glowRim: 0,
	},
	content: {
		text: "#3d1737",
		muted: "#77586f",
		link: "#7d294f",
		inset: "rgba(91,28,64,0.08)",
	},
	shapes: {
		flatFaces: false,
		cardRadius: 0.035,
		frameFront: 0,
		frameBack: 0,
		frameCard: 0,
	},
	walls: {
		...matte,
		metalness: 0.03,
		roughness: 0.96,
		clearcoat: 0,
		envMapIntensity: 0.12,
	},
	innerWalls: { ...matte, metalness: 0.08, roughness: 0.82 },
	cards: {
		...matte,
		metalness: 0.08,
		roughness: 0.48,
		clearcoat: 0.25,
		clearcoatRoughness: 0.5,
		envMapIntensity: 0.4,
		brightness: 1,
		useBlockColor: false,
		overlayOpacity: 0.09,
		rimHighlight: 0.3,
	},
	lighting: {
		ambient: 0.62,
		hemisphere: { sky: "#fff2de", ground: "#6f304e", intensity: 0.68 },
		key: { position: [-7, 15, 10], color: "#ffe4b8", intensity: 1.35 },
		fill: [{ position: [10, 7, -8], color: "#b66a83", intensity: 0.38 }],
		reflections: [
			{
				position: [2, 8, 4],
				scale: [5, 6, 1],
				color: "#f5d49a",
				intensity: 0.8,
			},
		],
	},
	floor: {
		pattern: "velvet",
		tiles: ["#55203f", "#632544"],
		opacity: 0.98,
		roughness: 0.95,
		metalness: 0,
		gridMajor: "#d7b06d",
		gridMinor: "#8d536e",
		gridOpacity: 0.075,
	},
	regions: {
		tint: "#7d3a57",
		signColor: "#6d243f",
		labelColor: "#ffe6b7",
		roughness: 0.86,
		frame: 0,
	},
	connections: { color: "#c9a060", luminous: false, haloOpacity: 0 },
	view: { fov: 37, position: [10, 9, 15] },
};
