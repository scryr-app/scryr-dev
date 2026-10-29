import { useContextBridge } from "@react-three/drei/core/useContextBridge";
import { QueryClientContext } from "@tanstack/react-query";
import { Html } from "@react-three/drei/web/Html";
import { useFrame, useThree } from "@react-three/fiber";
import { type CSSProperties, type ReactNode, useRef } from "react";
import { Color, FrontSide, type Group, NoBlending } from "three";
import { currentTheme } from "@/theme/theme";
import { useCardFocus } from "./CardFocusContext";
import { CARD_HTML_Z_RANGE, useCardForeground } from "./CardFocusScene";
import { getCardSurfaceLayout } from "./cardSurfaceLayout";
import { CardContentInteractiveContext, CardTitleContext } from "./CardTitle";
import styles from "./CardSurface.module.css";

/** The only card-content renderer: ordinary React DOM with native CSS layout. */
export function CardSurface({
	width,
	height,
	focusId,
	color,
	titleInteractive,
	children,
}: {
	width: number;
	height: number;
	focusId: string;
	color: string;
	titleInteractive: boolean;
	children: ReactNode;
}) {
	// Html uses a separate DOM root; keep card data hooks on the app query cache.
	const ContextBridge = useContextBridge(QueryClientContext);
	const { focusedIds, expandedId, isSelecting, cards, focusCard, closeFocus } = useCardFocus();
	const visibleIds = expandedId ? [expandedId] : focusedIds;
	const focused = visibleIds.includes(focusId);
	const interactive = focused && !isSelecting;
	const size = useThree((state) => state.size);
	const foreground = useCardForeground();
	const group = useRef<Group>(null);
	const html = useRef<HTMLDivElement>(null);
	const layout = getCardSurfaceLayout(
		width,
		height,
		size.width,
		size.height,
		focused ? visibleIds.length : 0,
	);
	const label = cards.find((card) => card.id === focusId)?.label ?? "Card";
	const background = new Color(color);
	const lightSurface =
		0.2126 * background.r + 0.7152 * background.g + 0.0722 * background.b > 0.179;
	// DOM is unlit: legacy white mesh text needs a dark counterpart on light cards.
	const content =
		currentTheme.appearance.content ??
		(lightSurface
			? {
					text: "#18212b",
					muted: "#374151",
					link: "#1e40af",
					inset: "rgba(255,255,255,0.28)",
				}
			: {
					text: "#ffffff",
					muted: "#e2e8f0",
					link: "#bfdbfe",
					inset: "rgba(0,0,0,0.22)",
				});
	const themeStyle = {
		"--card-text": content.text,
		"--card-muted": content.muted,
		"--card-link": content.link,
		"--card-inset": content.inset,
		backgroundColor: color,
		width: layout.pixelWidth,
		height: layout.pixelHeight,
		pointerEvents: interactive ? "auto" : "none",
	} as CSSProperties;

	useFrame(() => {
		const wrapper = html.current?.closest<HTMLElement>(`.${styles.layer}`);
		if (!wrapper || !group.current) return;
		// DOM ordering must mirror the foreground pass, including the return trip.
		if (foreground.contains(group.current)) wrapper.style.zIndex = focused ? "999999" : "999998";
		else wrapper.style.zIndex = String(Math.min(999000, Number(wrapper.style.zIndex)));
	}, 0.5);

	return (
		<group ref={group}>
			<Html
				ref={html}
				transform
				occlude="blending"
				distanceFactor={10}
				scale={layout.scale}
				pointerEvents="none"
				wrapperClass={styles.layer}
				zIndexRange={CARD_HTML_Z_RANGE}
				material={
					<shaderMaterial
						side={FrontSide}
						blending={NoBlending}
						depthWrite
						depthTest
						vertexShader="void main() { gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }"
						fragmentShader="void main() { gl_FragColor = vec4(0.0); }"
					/>
				}
			>
				<ContextBridge>
					<div
						className={styles.surface}
						style={themeStyle}
						data-card-id={focusId}
						data-card-focused={focused}
						data-card-layout-width={layout.pixelWidth}
						role="region"
						aria-label={label}
						aria-hidden={!interactive && !titleInteractive}
						inert={!interactive && !titleInteractive}
						tabIndex={interactive ? 0 : -1}
						onPointerDown={(event) => event.stopPropagation()}
						onPointerUp={(event) => event.stopPropagation()}
						onPointerMove={(event) => event.stopPropagation()}
						onClick={(event) => event.stopPropagation()}
						onDoubleClick={(event) => event.stopPropagation()}
						onWheel={(event) => event.stopPropagation()}
					>
						<CardTitleContext.Provider
							value={
								titleInteractive || interactive
									? () => {
											if (focusedIds.includes(focusId)) closeFocus();
											else focusCard(focusId);
										}
									: null
							}
						>
							<CardContentInteractiveContext.Provider value={interactive}>
								{children}
							</CardContentInteractiveContext.Provider>
						</CardTitleContext.Provider>
					</div>
				</ContextBridge>
			</Html>
		</group>
	);
}
