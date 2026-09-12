import {
	CanvasTexture,
	NearestFilter,
	RepeatWrapping,
	SRGBColorSpace,
} from "three";

const TEXTURE_SIZE = 256;
const TILE_COUNT = 8;
const TILE_SIZE = TEXTURE_SIZE / TILE_COUNT;

export function createFloorTexture(
	evenColor: string,
	oddColor: string,
	pattern?: "wood" | "stars" | "porcelain" | "seabed" | "velvet",
) {
	if (typeof document === "undefined") {
		return null;
	}

	const surface = document.createElement("canvas");
	surface.width = TEXTURE_SIZE;
	surface.height = TEXTURE_SIZE;

	const context = surface.getContext("2d");
	if (!context) {
		return null;
	}

	for (let row = 0; row < TILE_COUNT; row += 1) {
		for (let column = 0; column < TILE_COUNT; column += 1) {
			context.fillStyle = (row + column) % 2 === 0 ? evenColor : oddColor;
			context.fillRect(
				column * TILE_SIZE,
				row * TILE_SIZE,
				TILE_SIZE,
				TILE_SIZE,
			);
		}
	}

	if (pattern === "wood") {
		// Long walnut boards, with subtle grain and staggered joins.
		for (let plank = 0; plank < 4; plank++) {
			context.fillStyle = plank % 2 ? oddColor : evenColor;
			context.fillRect(plank * 64, 0, 64, TEXTURE_SIZE);
			for (let grain = 0; grain < 28; grain++) {
				const x = plank * 64 + grain * 2.3;
				context.strokeStyle =
					grain % 2 ? "rgba(187,142,79,0.045)" : "rgba(0,0,0,0.12)";
				context.beginPath();
				context.moveTo(x, 0);
				context.bezierCurveTo(x + 3, 80, x - 2, 170, x + 1, 256);
				context.stroke();
			}
			context.fillStyle = "rgba(0,0,0,0.35)";
			context.fillRect(plank * 64, 0, 1, 256);
			context.fillRect(plank * 64, plank % 2 ? 160 : 64, 64, 1);
		}
	} else if (pattern === "stars") {
		context.strokeStyle = "rgba(201,170,104,0.22)";
		context.lineWidth = 0.6;
		for (const radius of [32, 61, 95, 119]) {
			context.beginPath();
			context.ellipse(128, 128, radius, radius * 0.58, -0.32, 0, Math.PI * 2);
			context.stroke();
		}
		for (let i = 0; i < 90; i++) {
			context.fillStyle =
				i % 11 ? "rgba(220,231,246,0.5)" : "rgba(250,222,151,0.85)";
			context.fillRect(
				(i * 79) % 251,
				(i * 43) % 247,
				i % 11 ? 1 : 2,
				i % 11 ? 1 : 2,
			);
		}
	} else if (pattern === "porcelain") {
		context.strokeStyle = "rgba(37,82,145,0.2)";
		context.lineWidth = 1;
		for (let i = 0; i <= TEXTURE_SIZE; i += TILE_SIZE) {
			context.beginPath();
			context.moveTo(i, 0);
			context.lineTo(i, TEXTURE_SIZE);
			context.moveTo(0, i);
			context.lineTo(TEXTURE_SIZE, i);
			context.stroke();
		}
		context.strokeStyle = "rgba(190,150,64,0.2)";
		for (let i = 16; i < TEXTURE_SIZE; i += 64) {
			context.beginPath();
			context.arc(i, i, 9, 0, Math.PI * 2);
			context.stroke();
		}
	} else if (pattern === "seabed") {
		for (let i = 0; i < 850; i++) {
			const x = (i * 67.31) % TEXTURE_SIZE;
			const y = (i * 31.73) % TEXTURE_SIZE;
			context.fillStyle =
				i % 4 ? "rgba(2,18,19,0.09)" : "rgba(126,188,168,0.08)";
			context.fillRect(x, y, 1 + (i % 3), 1);
		}
	} else if (pattern === "velvet") {
		for (let i = 0; i < 2200; i++) {
			const x = (i * 73.17) % TEXTURE_SIZE;
			const y = (i * 41.39) % TEXTURE_SIZE;
			context.fillStyle =
				i % 3 ? "rgba(255,219,228,0.025)" : "rgba(25,2,15,0.035)";
			context.fillRect(x, y, 1, 1);
		}
	}

	const texture = new CanvasTexture(surface);
	texture.wrapS = RepeatWrapping;
	texture.wrapT = RepeatWrapping;
	texture.repeat.set(10, 10);
	texture.magFilter = NearestFilter;
	texture.minFilter = NearestFilter;
	texture.colorSpace = SRGBColorSpace;

	return texture;
}
