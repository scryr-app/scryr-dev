import { RoundedBox } from "@react-three/drei/core/RoundedBox";
import { useState } from "react";
import { cameraStore } from "@/camera";
import { currentTheme } from "@/theme/theme";

interface ZoomButtonProps {
	blockId?: string;
	x: number;
	y: number;
	z: number;
	blockPosition: [number, number, number];
	blockWidth: number;
	blockHeight: number;
	blockDepth: number;
	cardWidth: number;
	cardHeight: number;
	onSelectBlock: () => void;
	isVisible: boolean;
	onHoverChange: (isHovered: boolean) => void;
}

export function ZoomButton({
	blockId,
	x,
	y,
	z,
	blockPosition,
	blockWidth,
	blockHeight,
	blockDepth,
	cardWidth,
	cardHeight,
	onSelectBlock,
	isVisible,
	onHoverChange,
}: ZoomButtonProps) {
	const [isHovered, setIsHovered] = useState(false);
	const color = currentTheme.isDarkDiagram ? "#ffffff" : "#0f172a";
	const opacity = isVisible ? (isHovered ? 0.14 : 0.1) : 0;
	const bubbleOpacity = isVisible ? (isHovered ? 0.035 : 0.02) : 0;
	const overlayWidth = cardWidth * 0.94;
	const overlayHeight = cardHeight * 0.94;
	// Keep the magnifier circular while filling most of the card's height.
	const glyphScale = (Math.min(cardWidth, cardHeight) * 0.88) / 0.123;

	const focusBlock = () => {
		onSelectBlock();
		cameraStore.focusBlock({
			id: blockId,
			position: blockPosition,
			width: blockWidth,
			height: blockHeight,
			depth: blockDepth,
		});
	};

	return (
		<group position={[x, y, z]}>
			{/* biome-ignore lint/a11y/noStaticElementInteractions: THREE.Mesh is not an HTML element; pointer handlers are R3F events. */}
			<mesh
				position={[0, 0, 0.004]}
				onPointerEnter={(event) => {
					event.stopPropagation();
					setIsHovered(true);
					onHoverChange(true);
				}}
				onPointerLeave={(event) => {
					event.stopPropagation();
					setIsHovered(false);
					onHoverChange(false);
				}}
				onPointerDown={(event) => event.stopPropagation()}
				onClick={(event) => {
					event.stopPropagation();
					if (isVisible) {
						focusBlock();
					}
				}}
			>
				<boxGeometry args={[overlayWidth, overlayHeight, 0.008]} />
				<meshBasicMaterial transparent opacity={0} depthWrite={false} />
			</mesh>
			<ZoomOverlay
				opacity={bubbleOpacity}
				color={color}
				width={overlayWidth}
				height={overlayHeight}
			/>
			<group scale={[glyphScale, glyphScale, 1]}>
				<ZoomGlyph opacity={opacity} color={color} />
			</group>
		</group>
	);
}

function ZoomOverlay({
	opacity,
	color,
	width,
	height,
}: {
	opacity: number;
	color: string;
	width: number;
	height: number;
}) {
	return (
		<group position={[0, 0, 0.011]} renderOrder={999}>
			<RoundedBox args={[width, height, 0.012]} radius={0.06} smoothness={10}>
				<ZoomGlyphMaterial opacity={opacity} color={color} />
			</RoundedBox>
		</group>
	);
}

function ZoomGlyph({ opacity, color }: { opacity: number; color: string }) {
	return (
		<group position={[0, 0, 0.014]} renderOrder={1000}>
			<mesh position={[-0.018, 0.018, 0]}>
				<torusGeometry args={[0.043, 0.004, 8, 48]} />
				<ZoomGlyphMaterial opacity={opacity} color={color} />
			</mesh>
			<mesh position={[0.03, -0.03, 0]} rotation={[0, 0, -Math.PI / 4]}>
				<boxGeometry args={[0.07, 0.009, 0.006]} />
				<ZoomGlyphMaterial opacity={opacity} color={color} />
			</mesh>
			<mesh position={[-0.018, 0.018, 0.002]}>
				<boxGeometry args={[0.042, 0.006, 0.006]} />
				<ZoomGlyphMaterial opacity={opacity} color={color} />
			</mesh>
			<mesh position={[-0.018, 0.018, 0.004]}>
				<boxGeometry args={[0.006, 0.042, 0.006]} />
				<ZoomGlyphMaterial opacity={opacity} color={color} />
			</mesh>
		</group>
	);
}

function ZoomGlyphMaterial({
	opacity,
	color,
}: {
	opacity: number;
	color: string;
}) {
	return (
		<meshBasicMaterial
			color={color}
			transparent
			opacity={opacity}
			depthWrite={false}
			toneMapped={false}
		/>
	);
}
