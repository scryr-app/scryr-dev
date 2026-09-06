import { useMemo } from "react";
import * as THREE from "three";

/**
 * Generates a bitmap texture for block outer walls.
 * Produces a fine-grain + vertical-ribbing pattern tinted to the given color.
 */
export function useWallTexture(color: string): THREE.CanvasTexture | null {
	return useMemo(() => {
		const size = 256;
		const surface = document.createElement("canvas");
		surface.width = size;
		surface.height = size;
		const ctx = surface.getContext("2d");
		if (!ctx) return null;

		ctx.fillStyle = color;
		ctx.fillRect(0, 0, size, size);

		// Fine grain for a slightly industrial block surface.
		for (let i = 0; i < 4500; i++) {
			const x = Math.random() * size;
			const y = Math.random() * size;
			const alpha = 0.015 + Math.random() * 0.04;
			ctx.fillStyle = `rgba(255, 255, 255, ${alpha})`;
			ctx.fillRect(x, y, 1, 1);
		}

		// Vertical ribbing.
		for (let x = 0; x < size; x += 8) {
			ctx.fillStyle = "rgba(255, 255, 255, 0.04)";
			ctx.fillRect(x, 0, 1, size);
			ctx.fillStyle = "rgba(0, 0, 0, 0.03)";
			ctx.fillRect(x + 4, 0, 1, size);
		}

		const texture = new THREE.CanvasTexture(surface);
		texture.wrapS = THREE.RepeatWrapping;
		texture.wrapT = THREE.RepeatWrapping;
		texture.repeat.set(1.5, 1.25);
		texture.colorSpace = THREE.SRGBColorSpace;
		texture.needsUpdate = true;
		return texture;
	}, [color]);
}

/**
 * Generates a bitmap texture for ground/region planes.
 * Produces a mottled gradient + tile-line pattern tinted to the given color.
 * The repeat is scaled by the plane's world dimensions.
 */
export function useGroundTexture(
	color: string,
	width: number,
	height: number,
): THREE.CanvasTexture | null {
	return useMemo(() => {
		const size = 512;
		const surface = document.createElement("canvas");
		surface.width = size;
		surface.height = size;
		const ctx = surface.getContext("2d");
		if (!ctx) return null;

		const baseColor = new THREE.Color(color);
		const dark = baseColor.clone().multiplyScalar(0.93);
		const light = baseColor.clone().multiplyScalar(1.01);
		const gradient = ctx.createLinearGradient(0, 0, size, size);
		gradient.addColorStop(0, `#${dark.getHexString()}`);
		gradient.addColorStop(1, `#${light.getHexString()}`);
		ctx.fillStyle = gradient;
		ctx.fillRect(0, 0, size, size);

		// Mottled texture noise.
		for (let i = 0; i < 4000; i++) {
			const x = Math.random() * size;
			const y = Math.random() * size;
			const a = 0.004 + Math.random() * 0.012;
			ctx.fillStyle = `rgba(255, 255, 255, ${a})`;
			ctx.fillRect(x, y, 1, 1);
			ctx.fillStyle = `rgba(0, 0, 0, ${a * 0.6})`;
			ctx.fillRect(x + 0.5, y + 0.5, 1, 1);
		}

		// Soft tile lines.
		for (let x = 0; x <= size; x += 64) {
			ctx.fillStyle = "rgba(220, 210, 185, 0.06)";
			ctx.fillRect(x, 0, 1, size);
		}
		for (let y = 0; y <= size; y += 64) {
			ctx.fillStyle = "rgba(220, 210, 185, 0.06)";
			ctx.fillRect(0, y, size, 1);
		}

		const texture = new THREE.CanvasTexture(surface);
		texture.wrapS = THREE.RepeatWrapping;
		texture.wrapT = THREE.RepeatWrapping;
		texture.repeat.set(Math.max(1, width / 2), Math.max(1, height / 2));
		texture.colorSpace = THREE.SRGBColorSpace;
		texture.needsUpdate = true;
		return texture;
	}, [color, width, height]);
}
