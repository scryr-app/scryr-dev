import { ArrowUpRight, X } from "lucide-react";
import { useEffect, useState } from "react";
import styles from "./OpenSourcePrompt.module.css";

const DISMISSED_KEY = "scryr.openSourcePrompt.dismissed";

export function OpenSourcePrompt() {
	const [visible, setVisible] = useState(false);

	useEffect(() => {
		try {
			if (sessionStorage.getItem(DISMISSED_KEY)) return;
		} catch {
			// The prompt still works when browser storage is unavailable.
		}
		let remaining = 20_000;
		let startedAt = 0;
		let timer: ReturnType<typeof setTimeout> | undefined;
		const updateTimer = () => {
			if (timer !== undefined) {
				clearTimeout(timer);
				remaining -= Date.now() - startedAt;
				timer = undefined;
			}
			if (document.visibilityState === "visible") {
				startedAt = Date.now();
				timer = setTimeout(
					() => {
						setVisible(true);
						document.removeEventListener("visibilitychange", updateTimer);
					},
					Math.max(0, remaining),
				);
			}
		};
		updateTimer();
		document.addEventListener("visibilitychange", updateTimer);
		return () => {
			clearTimeout(timer);
			document.removeEventListener("visibilitychange", updateTimer);
		};
	}, []);

	function dismiss() {
		setVisible(false);
		try {
			sessionStorage.setItem(DISMISSED_KEY, "true");
		} catch {
			// Dismissal lasts for this mount if storage is disabled.
		}
	}

	if (!visible) return null;

	return (
		<aside aria-label="Install open-source Scryr" className={styles.prompt}>
			<button
				type="button"
				aria-label="Dismiss install prompt"
				onClick={dismiss}
				className={styles.dismiss}
			>
				<X size={16} />
			</button>
			<h2 className={styles.title}>Run Scryr on your machine</h2>
			<p className={styles.description}>
				Scryr is open source. Install it locally and explore your own architecture.
			</p>
			<a
				href="https://scryr.dev/getting-started/#install"
				target="_blank"
				rel="noopener noreferrer"
				className={styles.install}
			>
				Install open-source Scryr <ArrowUpRight size={16} />
			</a>
		</aside>
	);
}
