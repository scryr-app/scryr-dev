import { currentTheme } from "@/theme/theme";

const REGION_COLOR_COUNT = 10;

export function getRegionColor(tag: string) {
	const tagHash = tag.split("").reduce((acc, c) => acc + c.charCodeAt(0), 0);
	return currentTheme.getRegionColorByIndex(tagHash % REGION_COLOR_COUNT);
}

export function darkenHexColor(color: string, ratio = 0.4): string {
	const hex = color.replace("#", "");
	if (!/^[0-9a-fA-F]{6}$/.test(hex)) {
		return color;
	}

	const red = Math.floor(parseInt(hex.substring(0, 2), 16) * ratio);
	const green = Math.floor(parseInt(hex.substring(2, 4), 16) * ratio);
	const blue = Math.floor(parseInt(hex.substring(4, 6), 16) * ratio);

	return `#${red.toString(16).padStart(2, "0")}${green.toString(16).padStart(2, "0")}${blue.toString(16).padStart(2, "0")}`;
}
