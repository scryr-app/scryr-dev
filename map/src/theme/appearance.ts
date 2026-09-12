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
		wall: "ribbed" | "crystal";
		wallBrightness: number;
		glowCore: number;
		glowRim: number;
	};
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
	};
	floor: {
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
