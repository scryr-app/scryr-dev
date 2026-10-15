export interface FocusViewport {
	minX: number;
	maxX: number;
	minY: number;
	maxY: number;
}

/** Reserve breathing room around the card, including space for the toolbar. */
export const DEFAULT_FOCUS_VIEWPORT: FocusViewport = {
	minX: -0.82,
	maxX: 0.82,
	minY: -0.82,
	maxY: 0.82,
};

/** Find free canvas space around floating editor/toolbar panels at focus time. */
export function getFocusViewport(
	canvas: HTMLElement | null | undefined,
): FocusViewport {
	if (!canvas) return DEFAULT_FOCUS_VIEWPORT;
	const bounds = canvas.getBoundingClientRect();
	if (bounds.width <= 0 || bounds.height <= 0) return DEFAULT_FOCUS_VIEWPORT;
	let regions = [
		{
			left: bounds.left,
			right: bounds.right,
			top: bounds.top,
			bottom: bounds.bottom,
		},
	];
	for (const element of canvas.ownerDocument.querySelectorAll<HTMLElement>(
		"[data-camera-occluder]",
	)) {
		const panel = element.getBoundingClientRect();
		if (!panel.width || !panel.height) continue;
		regions = regions.flatMap((region) => {
			const left = Math.max(region.left, panel.left - 12);
			const right = Math.min(region.right, panel.right + 12);
			const top = Math.max(region.top, panel.top - 12);
			const bottom = Math.min(region.bottom, panel.bottom + 12);
			if (left >= right || top >= bottom) return [region];
			return [
				{ ...region, right: left },
				{ ...region, left: right },
				{ ...region, bottom: top },
				{ ...region, top: bottom },
			].filter((part) => part.right > part.left && part.bottom > part.top);
		});
	}
	const [region] = regions.sort(
		(a, b) =>
			(b.right - b.left) * (b.bottom - b.top) -
			(a.right - a.left) * (a.bottom - a.top),
	);
	if (!region) return DEFAULT_FOCUS_VIEWPORT;
	const marginX = (region.right - region.left) * 0.09;
	const marginY = (region.bottom - region.top) * 0.09;
	return {
		minX: (2 * (region.left + marginX - bounds.left)) / bounds.width - 1,
		maxX: (2 * (region.right - marginX - bounds.left)) / bounds.width - 1,
		minY: 1 - (2 * (region.bottom - marginY - bounds.top)) / bounds.height,
		maxY: 1 - (2 * (region.top + marginY - bounds.top)) / bounds.height,
	};
}
