import { ArrowUpRight, X } from "lucide-react";
import { useEffect, useState } from "react";

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
		<aside
			aria-label="Install open-source Scryr"
			className="fixed bottom-4 right-4 z-[1200] w-80 max-w-[calc(100vw-2rem)] rounded-2xl border border-white/15 bg-slate-950/95 p-5 text-slate-100 shadow-2xl shadow-black/40 backdrop-blur-xl"
		>
			<button
				type="button"
				aria-label="Dismiss install prompt"
				onClick={dismiss}
				className="absolute right-2 top-2 rounded-lg p-2 text-slate-400 hover:bg-white/10 hover:text-white focus-visible:outline-2 focus-visible:outline-cyan-300"
			>
				<X size={16} />
			</button>
			<h2 className="pr-6 text-base font-semibold">
				Run Scryr on your machine
			</h2>
			<p className="mt-2 text-sm leading-relaxed text-slate-300">
				Scryr is open source. Install it locally and explore your own
				architecture.
			</p>
			<a
				href="https://scryr.dev/getting-started/#install"
				target="_blank"
				rel="noopener noreferrer"
				className="mt-4 inline-flex items-center gap-2 rounded-lg bg-cyan-300 px-3 py-2 text-sm font-semibold text-slate-950 hover:bg-cyan-200 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-cyan-300"
			>
				Install open-source Scryr <ArrowUpRight size={16} />
			</a>
		</aside>
	);
}
