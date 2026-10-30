import { createContext, type ReactNode, useContext } from "react";
import { Maximize2 } from "lucide-react";
import styles from "./CardContent.module.css";

export const CardContentInteractiveContext = createContext(true);

export const CardTitleContext = createContext<(() => void) | null>(null);

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
					<Maximize2 className={styles.enlargeIcon} aria-hidden="true" />
				</button>
			) : (
				children
			)}
		</h2>
	);
}
