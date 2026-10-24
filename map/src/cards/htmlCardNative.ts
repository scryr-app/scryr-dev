import { HTMLTexture, SRGBColorSpace } from "three";

/** Three r185 implements the older experimental texElementImage2D API. */
export function supportsNativeHtml(canvas: HTMLCanvasElement, context: object) {
	return (
		"requestPaint" in canvas &&
		typeof canvas.requestPaint === "function" &&
		"texElementImage2D" in context &&
		typeof context.texElementImage2D === "function"
	);
}

export function createHtmlCardTexture(
	canvas: HTMLCanvasElement,
	host: HTMLDivElement,
	onPaint: () => void,
) {
	// Construct before parenting: HTMLTexture otherwise overwrites canvas.onpaint.
	const texture = new HTMLTexture(host);
	texture.colorSpace = SRGBColorSpace;
	const previousLayout = canvas.getAttribute("layoutsubtree");
	canvas.setAttribute("layoutsubtree", "");
	const previousContent = canvas.getAttribute("content");
	canvas.setAttribute("content", "drawable");
	host.setAttribute("drawable", "");
	canvas.appendChild(host);
	const paint = () => {
		texture.needsUpdate = true;
		onPaint();
	};
	canvas.addEventListener("paint", paint);
	(canvas as HTMLCanvasElement & { requestPaint: () => void }).requestPaint();
	return {
		texture,
		dispose() {
			canvas.removeEventListener("paint", paint);
			// Detach first: HTMLTexture.dispose otherwise clears canvas.onpaint.
			host.remove();
			texture.dispose();
			if (previousContent === null) canvas.removeAttribute("content");
			else canvas.setAttribute("content", previousContent);
			if (previousLayout === null) canvas.removeAttribute("layoutsubtree");
			else canvas.setAttribute("layoutsubtree", previousLayout);
		},
	};
}
