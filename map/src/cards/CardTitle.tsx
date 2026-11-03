import { createContext, type ReactNode, useContext } from "react";
import { X } from "lucide-react";
import styles from "./CardContent.module.css";

export const CardContentInteractiveContext = createContext(true);

export const CardTitleContext = createContext<(() => void) | null>(null);

export const CardCloseContext = createContext<(() => void) | null>(null);

export function CardCloseButton() {
	const close = useContext(CardCloseContext);
	if (!close) return null;
	return (
		<button
			type="button"
			className={styles.closeButton}
			onClick={close}
			aria-label="Close focus mode"
			title="Close focus mode"
		>
			<X size={18} aria-hidden="true" />
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
