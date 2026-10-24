import { Html } from "@react-three/drei/web/Html";
import { useFrame, useThree } from "@react-three/fiber";
import { useEffect, useRef, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { type HTMLTexture, type Mesh } from "three";
import { InteractionManager } from "three/addons/interaction/InteractionManager.js";
import { useCardFocus } from "./CardFocusContext";
import { HtmlGraphContent } from "./HtmlGraphContent";
import { createHtmlCardTexture, supportsNativeHtml } from "./htmlCardNative";
import styles from "./HtmlGraphCard.module.css";

/** Explicit opt-in; normal cards are untouched. Reload to change the renderer. */
export function htmlCardPrototypeMode() {
	const mode = new URLSearchParams(window.location.search).get("htmlCards");
	return mode === "native" || mode === "dom" ? mode : null;
}

export function HtmlGraphCard({
	width,
	height,
	focusId,
}: {
	width: number;
	height: number;
	focusId: string;
}) {
	const { gl, camera, invalidate } = useThree();
	const { focusedIds, expandedId, isSelecting, expandCard } = useCardFocus();
	const focused = (expandedId ? [expandedId] : focusedIds).includes(focusId);
	const interactive = focused && !isSelecting;
	const [host] = useState(() => document.createElement("div"));
	const [nativeFailed, setNativeFailed] = useState(false);
	const native =
		htmlCardPrototypeMode() === "native" &&
		supportsNativeHtml(gl.domElement, gl.getContext()) &&
		!nativeFailed;
	const [texture, setTexture] = useState<HTMLTexture | null>(null);
	const [ready, setReady] = useState(false);
	const mesh = useRef<Mesh>(null);
	const domRoot = useRef<Root | null>(null);
	const [interactions] = useState(() => new InteractionManager());
	const pixelHeight = Math.round((640 * height) / width);

	useEffect(() => {
		if (!native) return;
		host.className = styles.host;
		host.style.width = "1280px";
		host.style.height = `${pixelHeight * 2}px`;
		setReady(false);
		host.style.pointerEvents = "none";
		let painted = false;
		const resource = createHtmlCardTexture(gl.domElement, host, () => {
			painted = true;
			setReady(true);
			invalidate();
		});
		setTexture(resource.texture);
		// A disabled or incompatible experimental implementation must not leave a blank card.
		const timeout = window.setTimeout(() => {
			if (!painted) setNativeFailed(true);
		}, 2500);
		return () => {
			window.clearTimeout(timeout);
			resource.dispose();
		};
	}, [native, gl, host, pixelHeight, invalidate]);

	useEffect(() => {
		if (!native || !ready || !mesh.current) return;
		const surface = mesh.current;
		interactions.connect(gl, camera);
		interactions.add(surface);
		return () => {
			interactions.remove(surface);
			interactions.disconnect();
		};
	}, [native, ready, gl, camera, interactions]);

	useFrame(() => {
		if (!native || !ready || !mesh.current) return;
		mesh.current.updateWorldMatrix(true, false);
		camera.updateMatrixWorld();
		interactions.update();
		// Transitional Chromium builds require geometry registration for hit testing.
		// The manager already owns the CSS transform, so geometry stays identity.
		const canvas = gl.domElement as HTMLCanvasElement & {
			updateElementGeometry?: (
				element: HTMLElement,
				options: { canvasTransform: DOMMatrixInit },
			) => void;
		};
		canvas.updateElementGeometry?.(host, { canvasTransform: {} });
		host.style.pointerEvents = interactive ? "auto" : "none";
		host.inert = !interactive;
	}, 0.5); // After card transforms, before CardFocusScene's render pass.

	const content = (
		<div
			className={styles.host}
			style={{
				height: pixelHeight,
				zoom: native ? 2 : 1,
				pointerEvents: interactive ? "auto" : "none",
			}}
			inert={!interactive}
			onPointerDown={(event) => event.stopPropagation()}
			onPointerUp={(event) => event.stopPropagation()}
			onPointerMove={(event) => event.stopPropagation()}
			onClick={(event) => event.stopPropagation()}
			onDoubleClick={(event) => event.stopPropagation()}
			onWheel={(event) => event.stopPropagation()}
		>
			<HtmlGraphContent
				mode={native ? "Native HTML texture" : "DOM fallback"}
				onFocus={() => expandCard(focusId)}
			/>
		</div>
	);

	useEffect(() => {
		if (!native) return;
		const root = createRoot(host);
		domRoot.current = root;
		return () => {
			domRoot.current = null;
			root.unmount();
		};
	}, [native, host]);
	useEffect(() => {
		domRoot.current?.render(content);
	});

	if (!native) {
		return (
			<Html
				transform
				pointerEvents="none"
				distanceFactor={10}
				scale={width / 16}
				occlude={!focused}
				zIndexRange={focused ? [100, 90] : [80, 0]}
			>
				{content}
			</Html>
		);
	}
	return (
		<>
			{texture && ready && (
				<mesh ref={mesh}>
					<planeGeometry args={[width, height]} />
					<meshBasicMaterial
						map={texture}
						toneMapped={false}
						transparent
						alphaTest={0.01}
						onBeforeCompile={(shader) => {
							// r185 uploads HTML as RGBA8 rather than SRGB8_ALPHA8.
							shader.fragmentShader = shader.fragmentShader.replace(
								"#include <map_fragment>",
								"#include <map_fragment>\n diffuseColor = sRGBTransferEOTF(diffuseColor);",
							);
						}}
					/>
				</mesh>
			)}
		</>
	);
}
