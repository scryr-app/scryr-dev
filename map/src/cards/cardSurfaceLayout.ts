import { getCardFocusLayout } from "./cardFocusLayout";

/** Match CSS layout pixels to the focused footprint; preserve world-space proportions. */
export function getCardSurfaceLayout(
	width: number,
	height: number,
	viewportWidth: number,
	viewportHeight: number,
	focusCount: number,
) {
	const contentWidth = Math.max(0.01, width - 0.12);
	const contentHeight = Math.max(0.01, height - 0.12);
	const focused = getCardFocusLayout(viewportWidth, viewportHeight, focusCount, width / height)[0];
	const pixelWidth = focused
		? Math.max(240, Math.min(1440, Math.round(focused.width * (contentWidth / width) * 0.98)))
		: 420;
	return {
		pixelWidth,
		pixelHeight: (pixelWidth * contentHeight) / contentWidth,
		// Drei maps each CSS pixel to distanceFactor / 400 world units.
		scale: (contentWidth * 40) / pixelWidth,
	};
}
