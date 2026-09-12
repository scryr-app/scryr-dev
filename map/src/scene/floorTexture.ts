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
	pattern?: "wood",
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
