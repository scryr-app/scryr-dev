import { LoaderCircle } from "lucide-react";
import styles from "./Loading.module.css";

export function Loading() {
	return (
		<span role="status" className={styles.status}>
			<LoaderCircle size={18} aria-hidden="true" className={styles.spinner} />
			Loading…
		</span>
	);
}
