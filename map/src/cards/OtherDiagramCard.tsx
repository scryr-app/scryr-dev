import { Container, Text } from "@react-three/uikit";
import { Layers } from "@react-three/uikit-lucide";
import { currentTheme } from "@/theme/theme";

// Dimensions match Block defaults: width(3) * 0.8, height(2) * 0.8
const CARD_SIZE_X = 2.8;
const CARD_SIZE_Y = 1.8;
const PIXEL_SIZE = 0.01;
const INSET_BG = "rgba(0,0,0,0.22)";
const LABEL_COLOR = "rgba(255,255,255,0.40)";

function DiagramRow({ name }: { name: string }) {
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
		</Container>
	);
}

export interface OtherDiagramCardProps {
	/** Names of the diagrams (regions/tags) this block belongs to */
	diagrams?: string[];
}

/**
 * OtherDiagramCard shows the diagrams a block participates in.
 * Each row displays a provided diagram name.
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
