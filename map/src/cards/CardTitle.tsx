import { createContext, type ReactNode, useContext } from "react";
import { Maximize2, X } from "lucide-react";
import styles from "./CardContent.module.css";

export const CardContentInteractiveContext = createContext(true);

export const CardTitleContext = createContext<(() => void) | null>(null);

export const CardCloseContext = createContext<(() => void) | null>(null);

export function CardCloseButton() {
	const close = useContext(CardCloseContext);
	const enlarge = useContext(CardTitleContext);
	const action = close ?? enlarge;
	if (!action) return null;
	const label = close ? "Close focus mode" : "Enlarge card";
	return (
		<button
			type="button"
			className={styles.closeButton}
			onClick={action}
			aria-label={label}
			title={label}
		>
			{close ? <X size={18} aria-hidden="true" /> : <Maximize2 size={18} aria-hidden="true" />}
		</button>
	);
}

export function CardTitle({ children }: { children: ReactNode }) {
	const enlarge = useContext(CardTitleContext);
	return (
		<h2>
			{enlarge ? (
				<button
					type="button"
					className={styles.titleButton}
					onClick={enlarge}
					aria-label="Enlarge card"
				>
					{children}
				</button>
			) : (
				children
			)}
		</h2>
	);
}
