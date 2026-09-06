import { RoundedBox } from "@react-three/drei/core/RoundedBox";
import { useState } from "react";
import { cameraStore } from "@/camera";

const CARD_HEADER_TITLES = [
	"INFO",
	"GITHUB",
	"METRICS",
	"CI/CD",
	"TESTS",
	"DEPENDENCIES",
	"PERFORMANCE",
	"OTHER DIAGRAMS",
];

export function getCardHeaderZoomX(
	activeCardIndex: number | null,
	cardWidth: number,
) {
	const title = CARD_HEADER_TITLES[activeCardIndex ?? 0] ?? "CARD";
	const titleTextStartX = -cardWidth / 2 + 0.28;
	const estimatedTitleWidth = title.length * 0.076;
	const gapAfterTitle = 0.13;
	const maxX = cardWidth / 2 - 0.18;

	return Math.min(maxX, titleTextStartX + estimatedTitleWidth + gapAfterTitle);
}

interface ZoomButtonProps {
	blockId?: string;
	x: number;
	y: number;
	z: number;
	blockPosition: [number, number, number];
	blockWidth: number;
	blockHeight: number;
	blockDepth: number;
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
	onSelectBlock,
	isVisible,
	onHoverChange,
}: ZoomButtonProps) {
	const [isHovered, setIsHovered] = useState(false);
	const opacity = isVisible ? (isHovered ? 0.9 : 0.72) : 0;
	const bubbleOpacity = isVisible ? (isHovered ? 0.3 : 0.2) : 0;

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
				<boxGeometry args={[0.2, 0.2, 0.008]} />
				<meshBasicMaterial transparent opacity={0} depthWrite={false} />
			</mesh>
			<ZoomTooltipBubble opacity={bubbleOpacity} />
			<ZoomGlyph opacity={opacity} />
		</group>
	);
}

function ZoomTooltipBubble({ opacity }: { opacity: number }) {
	return (
		<group position={[0, 0, 0.011]} renderOrder={999}>
			<RoundedBox args={[0.17, 0.17, 0.012]} radius={0.04} smoothness={10}>
				<meshBasicMaterial
					color="#0f172a"
					transparent
					opacity={opacity}
					depthTest={false}
					depthWrite={false}
					toneMapped={false}
				/>
			</RoundedBox>
		</group>
	);
}

function ZoomGlyph({ opacity }: { opacity: number }) {
	return (
		<group position={[0, 0, 0.014]} renderOrder={1000}>
			<mesh position={[-0.018, 0.018, 0]}>
				<torusGeometry args={[0.043, 0.004, 8, 48]} />
				<ZoomGlyphMaterial opacity={opacity} />
			</mesh>
			<mesh position={[0.03, -0.03, 0]} rotation={[0, 0, -Math.PI / 4]}>
				<boxGeometry args={[0.07, 0.009, 0.006]} />
				<ZoomGlyphMaterial opacity={opacity} />
			</mesh>
			<mesh position={[-0.018, 0.018, 0.002]}>
				<boxGeometry args={[0.042, 0.006, 0.006]} />
				<ZoomGlyphMaterial opacity={opacity} />
			</mesh>
			<mesh position={[-0.018, 0.018, 0.004]}>
				<boxGeometry args={[0.006, 0.042, 0.006]} />
				<ZoomGlyphMaterial opacity={opacity} />
			</mesh>
		</group>
	);
}

function ZoomGlyphMaterial({ opacity }: { opacity: number }) {
	return (
		<meshBasicMaterial
			color="#ffffff"
			transparent
			opacity={opacity}
			depthTest={false}
			depthWrite={false}
			toneMapped={false}
		/>
	);
}
