import {
	createContext,
	type ReactNode,
	useCallback,
	useContext,
	useState,
} from "react";
import { useSelectedMap } from "@/graphql/sampleStore";

export interface FocusCard {
	id: string;
	blockId: string;
	label: string;
	color: string;
	readingContent: ReactNode;
}

export function getFocusCardId(blockId: string, index: number) {
	return JSON.stringify([blockId, index]);
}

interface FocusState {
	cards: FocusCard[];
	focusedIds: string[];
	expandedId: string | null;
}
const emptyState: FocusState = { cards: [], focusedIds: [], expandedId: null };
interface CardFocusContextValue extends FocusState {
	registerCards: (cards: FocusCard[]) => void;
	focusCard: (id: string) => void;
	addCard: (id: string) => void;
	removeCard: (id: string) => void;
	expandCard: (id: string | null) => void;
	closeFocus: () => void;
}
const CardFocusContext = createContext<CardFocusContextValue | null>(null);

export function CardFocusProvider({ children }: { children: ReactNode }) {
	const selectedMap = useSelectedMap();
	const scope = JSON.stringify([selectedMap.id, selectedMap.key]);
	const [state, setState] = useState<FocusState>(emptyState);
	const [previousScope, setPreviousScope] = useState(scope);
	// Reset only focus state; editor drafts and diagram selection stay mounted.
	if (scope !== previousScope) {
		setPreviousScope(scope);
		setState(emptyState);
	}
	const registerCards = useCallback((cards: FocusCard[]) => {
		setState((current) => {
			const available = new Set(cards.map((card) => card.id));
			return {
				cards,
				focusedIds: current.focusedIds.filter((id) => available.has(id)),
				expandedId:
					current.expandedId && available.has(current.expandedId)
						? current.expandedId
						: null,
			};
		});
	}, []);
	const focusCard = useCallback((id: string) => {
		setState((current) =>
			current.cards.some((card) => card.id === id)
				? { ...current, focusedIds: [id], expandedId: null }
				: current,
		);
	}, []);
	const addCard = useCallback((id: string) => {
		setState((current) =>
			current.cards.some((card) => card.id === id) &&
			!current.focusedIds.includes(id)
				? {
						...current,
						focusedIds: [...current.focusedIds, id],
						expandedId: null,
					}
				: current,
		);
	}, []);
	const removeCard = useCallback((id: string) => {
		setState((current) => ({
			...current,
			focusedIds: current.focusedIds.filter((value) => value !== id),
			expandedId: current.expandedId === id ? null : current.expandedId,
		}));
	}, []);
	const expandCard = useCallback((id: string | null) => {
		setState((current) =>
			id === null || current.focusedIds.includes(id)
				? { ...current, expandedId: id }
				: current,
		);
	}, []);
	const closeFocus = useCallback(() => {
		setState((current) => ({ ...current, focusedIds: [], expandedId: null }));
	}, []);
	return (
		<CardFocusContext.Provider
			value={{
				...state,
				registerCards,
				focusCard,
				addCard,
				removeCard,
				expandCard,
				closeFocus,
			}}
		>
			{children}
		</CardFocusContext.Provider>
	);
}

export function useCardFocus() {
	const context = useContext(CardFocusContext);
	if (!context)
		throw new Error("useCardFocus must be used within CardFocusProvider");
	return context;
}
