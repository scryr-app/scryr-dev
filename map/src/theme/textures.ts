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
		} else if (wall === "celestial") {
			// Radial instrument engraving, fine brass rulings and fixed star points.
			ctx.strokeStyle = "rgba(222,184,105,0.34)";
			ctx.lineWidth = 0.8;
			for (const radius of [34, 62, 94, 118]) {
				ctx.beginPath();
				ctx.ellipse(128, 128, radius, radius * 0.62, -0.34, 0, Math.PI * 2);
				ctx.stroke();
			}
			for (let i = 0; i < 80; i++) {
				const angle = (i * 2.39996) % (Math.PI * 2);
				const radius = 12 + ((i * 47) % 112);
				const x = 128 + Math.cos(angle) * radius;
				const y = 128 + Math.sin(angle) * radius * 0.72;
				ctx.fillStyle =
					i % 9 === 0 ? "rgba(255,239,184,0.9)" : "rgba(210,224,244,0.55)";
				ctx.beginPath();
				ctx.arc(x, y, i % 9 === 0 ? 1.25 : 0.55, 0, Math.PI * 2);
				ctx.fill();
			}
			for (let y = 0; y < size; y += 3) {
				ctx.fillStyle = "rgba(232,199,126,0.022)";
				ctx.fillRect(0, y, size, 1);
			}
		} else if (wall === "porcelain") {
			const glaze = ctx.createRadialGradient(86, 64, 8, 128, 128, 190);
			glaze.addColorStop(0, "rgba(255,255,255,0.32)");
			glaze.addColorStop(1, "rgba(177,192,207,0.08)");
			ctx.fillStyle = glaze;
			ctx.fillRect(0, 0, size, size);
			ctx.strokeStyle = "rgba(30,77,139,0.52)";
			ctx.lineWidth = 2;
			ctx.strokeRect(8, 8, size - 16, size - 16);
			ctx.strokeStyle = "rgba(192,151,66,0.32)";
			ctx.lineWidth = 0.8;
			ctx.beginPath();
			ctx.moveTo(168, 8);
			ctx.lineTo(160, 34);
			ctx.lineTo(172, 54);
			ctx.lineTo(165, 78);
			ctx.lineTo(183, 101);
			ctx.stroke();
			for (const [x, y] of [
				[28, 30],
				[222, 222],
				[34, 218],
				[220, 35],
			]) {
				ctx.strokeStyle = "rgba(32,80,145,0.42)";
				ctx.beginPath();
				ctx.arc(x, y, 7, 0, Math.PI * 2);
				ctx.moveTo(x - 12, y);
				ctx.quadraticCurveTo(x, y - 10, x + 12, y);
				ctx.stroke();
			}
		} else if (wall === "sunkenStone") {
			for (let i = 0; i < 5200; i++) {
				const x = (i * 71.91) % size;
				const y = (i * 29.47) % size;
				ctx.fillStyle = i % 5 ? "rgba(0,12,13,0.11)" : "rgba(119,181,165,0.1)";
				ctx.fillRect(x, y, 1 + (i % 3), 1);
			}
		} else if (wall === "velvet") {
			// A soft, unpatterned pile without borders or divination flourishes.
			for (let i = 0; i < 5200; i++) {
				const x = (i * 73.17) % size;
				const y = (i * 41.39) % size;
				ctx.fillStyle =
					i % 3 ? "rgba(255,220,230,0.035)" : "rgba(22,3,15,0.045)";
				ctx.fillRect(x, y, 0.8, 0.8);
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
	const wallStyle = currentTheme.appearance.textures.wall;
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

		// Carry the selected theme's surface language onto every region plane.
		if (wallStyle === "ribbed") {
			for (let x = 0; x < size; x += 16) {
				ctx.fillStyle = "rgba(255,255,255,0.045)";
				ctx.fillRect(x, 0, 1, size);
				ctx.fillStyle = "rgba(0,0,0,0.035)";
				ctx.fillRect(x + 7, 0, 1, size);
			}
		} else if (wallStyle === "crystal") {
			ctx.strokeStyle = "rgba(219,211,255,0.12)";
			ctx.lineWidth = 1;
			for (let x = -128; x < size; x += 96) {
				ctx.beginPath();
				ctx.moveTo(x, 0);
				ctx.lineTo(x + 160, size);
				ctx.lineTo(x + 230, 0);
				ctx.stroke();
			}
		} else if (wallStyle === "leather") {
			ctx.strokeStyle = "rgba(210,169,91,0.2)";
			ctx.lineWidth = 2;
			ctx.strokeRect(14, 14, size - 28, size - 28);
			ctx.strokeRect(22, 22, size - 44, size - 44);
			for (let i = 0; i < 1800; i++) {
				ctx.fillStyle = i % 3 ? "rgba(10,6,3,0.055)" : "rgba(240,215,164,0.04)";
				ctx.fillRect((i * 67.13) % size, (i * 31.79) % size, 1.4, 0.7);
			}
		} else if (wallStyle === "celestial") {
			ctx.strokeStyle = "rgba(222,184,105,0.24)";
			ctx.lineWidth = 1;
			for (const radius of [68, 122, 182, 232]) {
				ctx.beginPath();
				ctx.ellipse(256, 256, radius, radius * 0.58, -0.34, 0, Math.PI * 2);
				ctx.stroke();
			}
			for (let i = 0; i < 76; i++) {
				ctx.fillStyle =
					i % 9 ? "rgba(220,231,246,0.42)" : "rgba(250,222,151,0.72)";
				ctx.fillRect(
					(i * 157) % 503,
					(i * 89) % 499,
					i % 9 ? 1 : 2,
					i % 9 ? 1 : 2,
				);
			}
		} else if (wallStyle === "porcelain") {
			ctx.strokeStyle = "rgba(30,77,139,0.24)";
			ctx.lineWidth = 2;
			ctx.strokeRect(12, 12, size - 24, size - 24);
			for (const [x, y] of [
				[54, 54],
				[458, 458],
				[54, 458],
				[458, 54],
			]) {
				ctx.beginPath();
				ctx.arc(x, y, 13, 0, Math.PI * 2);
				ctx.moveTo(x - 22, y);
				ctx.quadraticCurveTo(x, y - 18, x + 22, y);
				ctx.stroke();
			}
			ctx.strokeStyle = "rgba(193,151,62,0.07)";
			ctx.beginPath();
			ctx.moveTo(350, 12);
			ctx.lineTo(338, 70);
			ctx.lineTo(354, 116);
			ctx.stroke();
		} else if (wallStyle === "velvet") {
			for (let i = 0; i < 4400; i++) {
				const x = (i * 73.17) % size;
				const y = (i * 41.39) % size;
				ctx.fillStyle =
					i % 3 ? "rgba(255,220,230,0.025)" : "rgba(22,3,15,0.035)";
				ctx.fillRect(x, y, 1, 1);
			}
		}

		const texture = new THREE.CanvasTexture(surface);
		texture.wrapS = THREE.RepeatWrapping;
		texture.wrapT = THREE.RepeatWrapping;
		texture.repeat.set(Math.max(1, width / 2), Math.max(1, height / 2));
		texture.colorSpace = THREE.SRGBColorSpace;
		texture.needsUpdate = true;
		return texture;
	}, [color, width, height, wallStyle]);
	useEffect(() => () => groundTexture?.dispose(), [groundTexture]);
	return groundTexture;
}

/** Paper fibers, darkened margins and a fine manuscript border; no image downloads. */
export function useCardTexture(): THREE.CanvasTexture | null {
	const style = currentTheme.appearance.textures.card;
	const texture = useMemo(() => {
		if (!style) return null;
		const size = 512;
		const surface = document.createElement("canvas");
		surface.width = surface.height = size;
		const ctx = surface.getContext("2d");
		if (!ctx) return null;
		// Neutral surface shading preserves the material color applied by Three.js.
		const wash = ctx.createRadialGradient(256, 225, 50, 256, 256, 350);
		wash.addColorStop(0, "#ffffff");
		wash.addColorStop(0.75, style === "seaGlass" ? "#d9eeee" : "#eeeeee");
		wash.addColorStop(1, style === "seaGlass" ? "#80aaa7" : "#b7b7b7");
		ctx.fillStyle = wash;
		ctx.fillRect(0, 0, size, size);
		const grainCount = style === "oracle" ? 3500 : 9000;
		for (let i = 0; i < grainCount; i++) {
			const x = (i * 71.17) % size,
				y = (i * 43.73) % size;
			ctx.fillStyle = i % 2 ? "rgba(20,20,20,0.045)" : "rgba(255,255,255,0.14)";
			ctx.fillRect(x, y, 1 + (i % 4), 0.6);
		}
		if (style === "parchment" || style === "porcelain") {
			ctx.strokeStyle =
				style === "porcelain" ? "rgba(24,71,140,0.68)" : "rgba(86,59,28,0.4)";
			ctx.lineWidth = 1;
			ctx.strokeRect(10, 10, size - 20, size - 20);
			ctx.strokeStyle =
				style === "porcelain" ? "rgba(190,145,55,0.52)" : "rgba(86,59,28,0.16)";
			ctx.strokeRect(14, 14, size - 28, size - 28);
			// Small corner ornaments remain outside the reading area.
			for (const x of [18, size - 18])
				for (const y of [18, size - 18]) {
					ctx.beginPath();
					ctx.arc(x, y, 3, 0, Math.PI * 2);
					ctx.stroke();
				}
		}
		if (style === "porcelain") {
			// Painted cobalt sprigs and restrained gold-filled cracks.
			ctx.strokeStyle = "rgba(28,77,145,0.52)";
			for (const side of [-1, 1]) {
				ctx.beginPath();
				ctx.moveTo(256 + side * 210, 390);
				ctx.bezierCurveTo(
					256 + side * 160,
					350,
					256 + side * 190,
					280,
					256 + side * 135,
					245,
				);
				ctx.stroke();
				for (let i = 0; i < 4; i++) {
					ctx.beginPath();
					ctx.ellipse(
						256 + side * (180 - i * 12),
						350 - i * 30,
						12,
						5,
						side * 0.6,
						0,
						Math.PI * 2,
					);
					ctx.stroke();
				}
			}
			ctx.strokeStyle = "rgba(193,151,62,0.1)";
			ctx.beginPath();
			ctx.moveTo(350, 10);
			ctx.lineTo(342, 53);
			ctx.lineTo(356, 84);
			ctx.lineTo(347, 120);
			ctx.stroke();
		}
		const paper = new THREE.CanvasTexture(surface);
		paper.colorSpace = THREE.SRGBColorSpace;
		return paper;
	}, [style]);
	useEffect(() => () => texture?.dispose(), [texture]);
	return texture;
}
