import { createContext, type ReactNode, useContext, useState } from "react";

interface SelectedBlockState {
	name: string;
	lineNumber: number | null;
	selectionVersion: number;
}

interface MapTrayContextValue {
	activeCardIndex: number | null;
	toggleCard: (index: number) => void;
	selectedBlock: SelectedBlockState | null;
	selectBlock: (name: string, lineNumber: number | null) => void;
}

const MapTrayContext = createContext<MapTrayContextValue | null>(null);

/** Provides global active-card state shared across all Blocks and the MapTray. */
export function MapTrayProvider({ children }: { children: ReactNode }) {
	const [activeCardIndex, setActiveCardIndex] = useState<number | null>(0);
	const [selectedBlock, setSelectedBlock] = useState<SelectedBlockState | null>(
		null,
	);

	const toggleCard = (index: number) => {
		setActiveCardIndex((prev) => (prev === index ? 0 : index));
	};

	const selectBlock = (name: string, lineNumber: number | null) => {
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
				selectedBlock,
				selectBlock,
			}}
		>
			{children}
		</MapTrayContext.Provider>
	);
}

/** Returns the global active-card index and toggle function. Must be used inside MapTrayProvider. */
export function useMapTray() {
	const ctx = useContext(MapTrayContext);
	if (!ctx) throw new Error("useMapTray must be used within MapTrayProvider");
	return ctx;
}
