import { createContext, type ReactNode, useContext, useState } from "react";
import type { CollectorEvidence, EvidenceSection } from "./evidence";

export interface EvidenceSelection {
	section: EvidenceSection;
	collectors: CollectorEvidence[];
	selectedId: string;
}
const EvidenceDetailsContext = createContext<{
	selection: EvidenceSelection | null;
	open: (selection: EvidenceSelection) => void;
	close: () => void;
} | null>(null);
export function EvidenceDetailsProvider({ children }: { children: ReactNode }) {
	const [selection, setSelection] = useState<EvidenceSelection | null>(null);
	return (
		<EvidenceDetailsContext.Provider
			value={{ selection, open: setSelection, close: () => setSelection(null) }}
		>
			{children}
		</EvidenceDetailsContext.Provider>
	);
}
export function useEvidenceDetails() {
	const context = useContext(EvidenceDetailsContext);
	if (!context)
		throw new Error("Evidence details require EvidenceDetailsProvider");
	return context;
}
