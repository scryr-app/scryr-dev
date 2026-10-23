export interface CardFocusRectangle {
	x: number;
	y: number;
	width: number;
	height: number;
}

/** Pixel sizes and center offsets in camera space: positive x right, positive y up. */
export function getCardFocusLayout(
	viewportWidth: number,
	viewportHeight: number,
	count: number,
	cardAspect: number,
): CardFocusRectangle[] {
	if (
		![viewportWidth, viewportHeight, count, cardAspect].every(Number.isFinite) ||
		viewportWidth <= 0 ||
		viewportHeight <= 0 ||
		cardAspect <= 0 ||
		count < 1
	)
		return [];
	const total = Math.floor(count);
	const mobile = viewportWidth < 600;
	const horizontalMargin = viewportWidth * 0.05;
	const verticalMargin = Math.min(
		viewportHeight * 0.2,
		Math.max(mobile ? 80 : 64, viewportHeight * 0.07),
	);
	const availableWidth = viewportWidth - horizontalMargin * 2;
	const availableHeight = viewportHeight - verticalMargin * 2;
	const preferredGap = Math.min(20, Math.min(viewportWidth, viewportHeight) * 0.025);
	let best = { columns: 1, rows: total, width: 0, height: 0, gap: 0 };
	for (let columns = 1; columns <= total; columns++) {
		// A pair follows the viewport orientation for predictable comparison.
		if (total === 2 && columns !== (viewportWidth >= viewportHeight ? 2 : 1)) continue;
		const rows = Math.ceil(total / columns);
		const gap = Math.min(
			preferredGap,
			availableWidth / (columns * 4),
			availableHeight / (rows * 4),
		);
		const cellWidth = (availableWidth - (columns - 1) * gap) / columns;
		const cellHeight = (availableHeight - (rows - 1) * gap) / rows;
		const height = Math.min(cellHeight, cellWidth / cardAspect);
		const width = height * cardAspect;
		if (height > best.height) best = { columns, rows, width, height, gap };
	}
	const { columns, rows, width, height, gap } = best;
	const groupHeight = rows * height + (rows - 1) * gap;
	return Array.from({ length: total }, (_, index) => {
		const row = Math.floor(index / columns);
		const column = index % columns;
		const rowCount = Math.min(columns, total - row * columns);
		const rowWidth = rowCount * width + (rowCount - 1) * gap;
		return {
			x: -rowWidth / 2 + width / 2 + column * (width + gap),
			y: groupHeight / 2 - height / 2 - row * (height + gap),
			width,
			height,
		};
	});
}
