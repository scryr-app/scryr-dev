import { Container, Image, Text } from "@react-three/uikit";
import { Layers } from "@react-three/uikit-lucide";
import { useMemo } from "react";
import { currentTheme } from "@/theme/theme";

// Dimensions match Block defaults: width(3) * 0.8, height(2) * 0.8
const CARD_SIZE_X = 2.8;
const CARD_SIZE_Y = 1.8;
const PIXEL_SIZE = 0.01;
const INSET_BG = "rgba(0,0,0,0.22)";
const LABEL_COLOR = "rgba(255,255,255,0.40)";

/**
 * Generates a blurry PNG data URL previewing 3 abstract block shapes.
 * Colors are seeded from the diagram name so each diagram has a distinct look.
 */
function useDiagramPreviewDataUrl(diagramName: string): string {
	return useMemo(() => {
		const W = 180;
		const H = 90;
		const surface = document.createElement("canvas");
		surface.width = W;
		surface.height = H;
		const ctx = surface.getContext("2d");
		if (!ctx) return "";

		// Seed hues from the diagram name for per-diagram colour variation
		const seed = diagramName
			.split("")
			.reduce((acc, ch) => acc + ch.charCodeAt(0), 0);
		const hue1 = (seed * 137) % 360;
		const hue2 = (seed * 73 + 120) % 360;
		const hue3 = (seed * 41 + 240) % 360;

		// Dark background
		ctx.fillStyle = "#0f1117";
		ctx.fillRect(0, 0, W, H);

		// Connection lines between blocks (drawn first, behind blocks)
		ctx.filter = "blur(5px)";
		ctx.strokeStyle = "rgba(255,255,255,0.18)";
		ctx.lineWidth = 2;
		ctx.beginPath();
		ctx.moveTo(60, 45);
		ctx.lineTo(90, 45);
		ctx.moveTo(135, 45);
		ctx.lineTo(155, 45);
		ctx.stroke();

		// Three block rectangles with blur
		const blocks = [
			{ x: 10, y: 28, w: 50, h: 32, hue: hue1 },
			{ x: 68, y: 22, w: 50, h: 32, hue: hue2 },
			{ x: 126, y: 30, w: 50, h: 32, hue: hue3 },
		];

		for (const b of blocks) {
			// Block body
			ctx.fillStyle = `hsla(${b.hue}, 55%, 40%, 0.9)`;
			ctx.fillRect(b.x, b.y, b.w, b.h);
			// Subtle top highlight
			ctx.fillStyle = `hsla(${b.hue}, 70%, 70%, 0.3)`;
			ctx.fillRect(b.x, b.y, b.w, 5);
		}

		// Second pass: heavier blur overlay for the "blurry preview" aesthetic
		ctx.filter = "blur(3px)";
		for (const b of blocks) {
			ctx.fillStyle = `hsla(${b.hue}, 60%, 50%, 0.25)`;
			ctx.fillRect(b.x - 4, b.y - 4, b.w + 8, b.h + 8);
		}

		return surface.toDataURL("image/png");
	}, [diagramName]);
}

/** Single row: diagram name chip (left) + blurry preview (right). */
function DiagramRow({ name }: { name: string }) {
	const previewUrl = useDiagramPreviewDataUrl(name);
	const c = currentTheme.cardTextColor;

	return (
		<Container
			flexDirection="row"
			alignItems="center"
			gap={8}
			backgroundColor={INSET_BG}
			borderRadius={5}
			padding={7}
		>
			{/* Left column: diagram name */}
			<Container flexGrow={1} flexDirection="column" gap={2}>
				<Container flexDirection="row" alignItems="center" gap={4}>
					<Layers width={9} height={9} color={LABEL_COLOR} />
					<Text fontSize={10} color={LABEL_COLOR}>
						DIAGRAM
					</Text>
				</Container>
				<Text fontSize={13} color={c}>
					{name}
				</Text>
			</Container>

			{/* Right column: blurry preview image */}
			{previewUrl && (
				<Container borderRadius={4} overflow="hidden" width={90} height={50}>
					<Image src={previewUrl} width={90} height={50} objectFit="cover" />
				</Container>
			)}
		</Container>
	);
}

export interface OtherDiagramCardProps {
	/** Names of the diagrams (regions/tags) this block belongs to */
	diagrams?: string[];
}

/**
 * OtherDiagramCard shows the diagrams a block participates in.
 * Each row has the diagram name on the left and a blurry 3-block preview on the right.
 */
export function OtherDiagramCard({ diagrams = [] }: OtherDiagramCardProps) {
	const c = currentTheme.cardTextColor;

	return (
		<Container
			sizeX={CARD_SIZE_X}
			sizeY={CARD_SIZE_Y}
			pixelSize={PIXEL_SIZE}
			flexDirection="column"
			padding={12}
			gap={6}
			alignItems="stretch"
		>
			{/* Header */}
			<Container flexDirection="row" alignItems="center" gap={5}>
				<Layers width={12} height={12} color={c} />
				<Text fontSize={14} color={c}>
					OTHER DIAGRAMS
				</Text>
			</Container>

			{diagrams.length === 0 ? (
				<Container flexGrow={1} alignItems="center" justifyContent="center">
					<Text fontSize={11} color={LABEL_COLOR}>
						No diagrams found
					</Text>
				</Container>
			) : (
				<Container
					flexDirection="column"
					gap={5}
					alignItems="stretch"
					flexGrow={1}
				>
					{diagrams.map((name) => (
						<DiagramRow key={name} name={name} />
					))}
				</Container>
			)}
		</Container>
	);
}
