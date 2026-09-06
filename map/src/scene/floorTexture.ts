import {
	CanvasTexture,
	NearestFilter,
	RepeatWrapping,
	SRGBColorSpace,
} from "three";

const TEXTURE_SIZE = 256;
const TILE_COUNT = 8;
const TILE_SIZE = TEXTURE_SIZE / TILE_COUNT;

function createCheckerTexture(evenColor: string, oddColor: string) {
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

export function createLightFloorCheckerTexture() {
	return createCheckerTexture(
		"rgba(255, 255, 255, 0.72)",
		"rgba(232, 233, 237, 0.72)",
	);
}

export function createDarkFloorCheckerTexture() {
	return createCheckerTexture(
		"rgba(255, 255, 255, 0.18)",
		"rgba(148, 163, 184, 0.18)",
	);
}
