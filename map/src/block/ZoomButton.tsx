import { RoundedBox } from "@react-three/drei/core/RoundedBox";
import { useState } from "react";
import { cameraStore } from "@/camera";
import { currentTheme } from "@/theme/theme";

interface ZoomButtonProps {
	blockId?: string;
	isVisible: boolean;
	blockPosition: [number, number, number];
	blockWidth: number;
	blockHeight: number;
	blockDepth: number;
	onSelectBlock: () => void;
}

export function ZoomButton({
	blockId,
	isVisible,
	blockPosition,
	blockWidth,
	blockHeight,
	blockDepth,
	onSelectBlock,
}: ZoomButtonProps) {
	const [isHovered, setIsHovered] = useState(false);
	const color = currentTheme.isDarkDiagram ? "#ffffff" : "#0f172a";
	const opacity = isVisible ? (isHovered ? 0.85 : 0.5) : 0;
	const overlayWidth = blockWidth * 0.96;
	const overlayHeight = blockDepth * 0.96;
	// Leave the centered block label clear, with a compact glyph toward the front.
	const glyphScale = (Math.min(blockWidth, blockDepth) * 0.26) / 0.123;

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
		// biome-ignore lint/a11y/noStaticElementInteractions: THREE.Group uses R3F pointer events.
		<group
			position={[0, blockHeight / 2 + 0.015, 0]}
			rotation={[-Math.PI / 2, 0, 0]}
			onPointerEnter={() => setIsHovered(true)}
			onPointerLeave={() => setIsHovered(false)}
			onPointerDown={(event) => event.stopPropagation()}
			onClick={(event) => {
				event.stopPropagation();
				focusBlock();
			}}
		>
			{/* A single top-facing hit plane leaves the card faces unobstructed. */}
			<mesh>
				<planeGeometry args={[overlayWidth, overlayHeight]} />
				<meshBasicMaterial transparent opacity={0} depthWrite={false} />
			</mesh>
			<ZoomOverlay
				opacity={isVisible && isHovered ? 0.06 : 0}
				color={color}
				width={overlayWidth}
				height={overlayHeight}
			/>
			<group
				position={[0, -blockDepth * 0.3, 0]}
				scale={[glyphScale, glyphScale, 1]}
			>
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
