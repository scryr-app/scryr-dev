import { useEffect, useMemo } from "react";
import * as THREE from "three";
import { currentTheme } from "./theme";

/** Soft material depth with only a hint of mineral facets. */
export function useWallTexture(color: string): THREE.CanvasTexture | null {
	const { wall, wallBrightness } = currentTheme.appearance.textures;
	const texture = useMemo(() => {
		const size = 256;
		const surface = document.createElement("canvas");
		surface.width = surface.height = size;
		const ctx = surface.getContext("2d");
		if (!ctx) return null;
		ctx.fillStyle = new THREE.Color(color)
			.multiplyScalar(wallBrightness)
			.getStyle();
		ctx.fillRect(0, 0, size, size);
		if (wall === "ribbed") {
			// Fine grain and vertical ribs from the original industrial blocks.
			for (let i = 0; i < 4500; i++) {
				const x = (i * 73.31) % size,
					y = (i * 37.19) % size;
				ctx.fillStyle = "rgba(255,255,255,0.035)";
				ctx.fillRect(x, y, 1, 1);
			}
			for (let x = 0; x < size; x += 8) {
				ctx.fillStyle = "rgba(255,255,255,0.04)";
				ctx.fillRect(x, 0, 1, size);
				ctx.fillStyle = "rgba(0,0,0,0.03)";
				ctx.fillRect(x + 4, 0, 1, size);
			}
		} else if (wall === "leather") {
			// Fine pores and shallow creases, with restrained gilt tooling.
			for (let i = 0; i < 7000; i++) {
				const x = (i * 67.13) % size,
					y = (i * 31.79) % size;
				ctx.fillStyle =
					i % 3 === 0 ? "rgba(240,215,164,0.09)" : "rgba(10,6,3,0.1)";
				ctx.fillRect(x, y, 1.5, 0.7);
			}
			ctx.strokeStyle = "rgba(210,169,91,0.48)";
			ctx.lineWidth = 1.2;
			ctx.strokeRect(10, 10, size - 20, size - 20);
			ctx.strokeStyle = "rgba(210,169,91,0.22)";
			ctx.strokeRect(15, 15, size - 30, size - 30);
			for (const x of [24, size - 24]) {
				for (const y of [24, size - 24]) {
					ctx.beginPath();
					ctx.moveTo(x, y - 4);
					ctx.lineTo(x + 3, y);
					ctx.lineTo(x, y + 4);
					ctx.lineTo(x - 3, y);
					ctx.closePath();
					ctx.stroke();
				}
			}
		} else {
			const depth = ctx.createLinearGradient(0, size, size, 0);
			depth.addColorStop(0, "rgba(0, 0, 0, 0.16)");
			depth.addColorStop(0.48, "rgba(255, 255, 255, 0.02)");
			depth.addColorStop(1, "rgba(255, 255, 255, 0.035)");
			ctx.fillStyle = depth;
			ctx.fillRect(0, 0, size, size);
			for (const [points, alpha] of [
				[[0, 0, 92, 0, 174, 256, 0, 198], 0.025],
				[[92, 0, 256, 0, 256, 62, 174, 256], 0.04],
				[[0, 198, 174, 256, 0, 256], 0.03],
			] as const) {
				ctx.beginPath();
				ctx.moveTo(points[0], points[1]);
				for (let i = 2; i < points.length; i += 2)
					ctx.lineTo(points[i], points[i + 1]);
				ctx.closePath();
				ctx.fillStyle = `rgba(255, 255, 255, ${alpha})`;
				ctx.fill();
			}
		}
		const result = new THREE.CanvasTexture(surface);
		result.colorSpace = THREE.SRGBColorSpace;
		return result;
	}, [color, wall, wallBrightness]);
	useEffect(() => () => texture?.dispose(), [texture]);
	return texture;
}

/** Diffuse core and feathered inner rim, applied as emission instead of paint. */
export function useCrystalGlowTexture(): THREE.CanvasTexture | null {
	const { glowCore, glowRim } = currentTheme.appearance.textures;
	const texture = useMemo(() => {
		const size = 128;
		const surface = document.createElement("canvas");
		surface.width = surface.height = size;
		const ctx = surface.getContext("2d");
		if (!ctx) return null;
		const pixels = ctx.createImageData(size, size);
		for (let y = 0; y < size; y++) {
			for (let x = 0; x < size; x++) {
				const u = x / (size - 1),
					v = y / (size - 1);
				const edge = Math.min(u, v, 1 - u, 1 - v);
				const core = Math.exp(-((u - 0.5) ** 2 + (v - 0.42) ** 2) * 7);
				const rim = Math.exp(-edge * 14);
				const value = Math.round(
					255 * Math.min(1, core * glowCore + rim * glowRim),
				);
				const i = (y * size + x) * 4;
				pixels.data[i] = pixels.data[i + 1] = pixels.data[i + 2] = value;
				pixels.data[i + 3] = 255;
			}
		}
		ctx.putImageData(pixels, 0, 0);
		const result = new THREE.CanvasTexture(surface);
		result.colorSpace = THREE.SRGBColorSpace;
		return result;
	}, [glowCore, glowRim]);
	useEffect(() => () => texture?.dispose(), [texture]);
	return texture;
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
	const groundTexture = useMemo(() => {
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
	useEffect(() => () => groundTexture?.dispose(), [groundTexture]);
	return groundTexture;
}

/** Paper fibers, darkened margins and a fine manuscript border; no image downloads. */
export function useCardTexture(): THREE.CanvasTexture | null {
	const style = currentTheme.appearance.textures.card;
	const texture = useMemo(() => {
		if (style !== "parchment") return null;
		const size = 512;
		const surface = document.createElement("canvas");
		surface.width = surface.height = size;
		const ctx = surface.getContext("2d");
		if (!ctx) return null;
		// Neutral paper shading preserves the parent block’s hue.
		const wash = ctx.createRadialGradient(256, 225, 50, 256, 256, 350);
		wash.addColorStop(0, "#ffffff");
		wash.addColorStop(0.75, "#eeeeee");
		wash.addColorStop(1, "#b7b7b7");
		ctx.fillStyle = wash;
		ctx.fillRect(0, 0, size, size);
		for (let i = 0; i < 9000; i++) {
			const x = (i * 71.17) % size,
				y = (i * 43.73) % size;
			ctx.fillStyle = i % 2 ? "rgba(20,20,20,0.045)" : "rgba(255,255,255,0.14)";
			ctx.fillRect(x, y, 1 + (i % 4), 0.6);
		}
		ctx.strokeStyle = "rgba(86,59,28,0.4)";
		ctx.lineWidth = 1;
		ctx.strokeRect(10, 10, size - 20, size - 20);
		ctx.strokeStyle = "rgba(86,59,28,0.16)";
		ctx.strokeRect(14, 14, size - 28, size - 28);
		// Small corner ornaments remain outside the reading area.
		for (const x of [18, size - 18])
			for (const y of [18, size - 18]) {
				ctx.beginPath();
				ctx.arc(x, y, 3, 0, Math.PI * 2);
				ctx.stroke();
			}
		const paper = new THREE.CanvasTexture(surface);
		paper.colorSpace = THREE.SRGBColorSpace;
		return paper;
	}, [style]);
	useEffect(() => () => texture?.dispose(), [texture]);
	return texture;
}
