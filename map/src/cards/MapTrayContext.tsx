import {
	createContext,
	type ReactNode,
	useCallback,
	useContext,
	useState,
} from "react";

interface SelectedBlockState {
	name: string;
	lineNumber: number | null;
	selectionVersion: number;
}

interface MapTrayContextValue {
	activeCardIndex: number | null;
	toggleCard: (index: number) => void;
	getActiveCardIndex: (blockId: string) => number | null;
	selectCardForBlock: (blockId: string, index: number) => void;
	previewCardForBlock: (blockId: string, index: number) => void;
	clearCardPreview: (blockId: string, index?: number) => void;
	selectedBlock: SelectedBlockState | null;
	selectBlock: (name: string, lineNumber: number | null) => void;
}

const MapTrayContext = createContext<MapTrayContextValue | null>(null);

/** Shares toolbar card selection with per-block choices from direct card clicks. */
export function MapTrayProvider({ children }: { children: ReactNode }) {
	const [sharedCardIndex, setSharedCardIndex] = useState<number | null>(0);
	const [blockCardIndexes, setBlockCardIndexes] = useState(
		() => new Map<string, number>(),
	);
	const [selectedBlock, setSelectedBlock] = useState<SelectedBlockState | null>(
		null,
	);

	const [activeBlockId, setActiveBlockId] = useState<string | null>(null);
	const [hoveredCard, setHoveredCard] = useState<{
		blockId: string;
		index: number;
	} | null>(null);
	const previewCardForBlock = useCallback((blockId: string, index: number) => {
		setHoveredCard({ blockId, index });
	}, []);
	const clearCardPreview = useCallback((blockId: string, index?: number) => {
		setHoveredCard((current) =>
			current?.blockId === blockId &&
			(index === undefined || current.index === index)
				? null
				: current,
		);
	}, []);
	const activeCardIndex =
		hoveredCard?.index ??
		(activeBlockId === null
			? undefined
			: blockCardIndexes.get(activeBlockId)) ??
		sharedCardIndex;

	const toggleCard = (index: number) => {
		setSharedCardIndex(activeCardIndex === index ? 0 : index);
		setBlockCardIndexes(new Map());
		setHoveredCard(null);
	};

	const getActiveCardIndex = (blockId: string) =>
		blockCardIndexes.get(blockId) ?? sharedCardIndex;

	const selectCardForBlock = (blockId: string, index: number) => {
		setBlockCardIndexes((previous) => new Map(previous).set(blockId, index));
		setActiveBlockId(blockId);
	};

	const selectBlock = (name: string, lineNumber: number | null) => {
		setActiveBlockId(name);
		setSelectedBlock((prev) => ({
			name,
			lineNumber,
			selectionVersion: (prev?.selectionVersion ?? 0) + 1,
		}));
	};

	return (
		<MapTrayContext.Provider
			value={{
				activeCardIndex,
				toggleCard,
				getActiveCardIndex,
				selectCardForBlock,
				previewCardForBlock,
				clearCardPreview,
				selectedBlock,
				selectBlock,
			}}
		>
			{children}
		</MapTrayContext.Provider>
	);
}

/** Returns the toolbar selection and per-block card controls. Must be used inside MapTrayProvider. */
export function useMapTray() {
	const ctx = useContext(MapTrayContext);
	if (!ctx) throw new Error("useMapTray must be used within MapTrayProvider");
	return ctx;
}
