import {
	CanvasTexture,
	NearestFilter,
	RepeatWrapping,
	SRGBColorSpace,
} from "three";

const TEXTURE_SIZE = 256;
const TILE_COUNT = 8;
const TILE_SIZE = TEXTURE_SIZE / TILE_COUNT;

export function createFloorTexture(evenColor: string, oddColor: string) {
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

	const texture = new CanvasTexture(surface);
	texture.wrapS = RepeatWrapping;
	texture.wrapT = RepeatWrapping;
	texture.repeat.set(10, 10);
	texture.magFilter = NearestFilter;
	texture.minFilter = NearestFilter;
	texture.colorSpace = SRGBColorSpace;

	return texture;
}
