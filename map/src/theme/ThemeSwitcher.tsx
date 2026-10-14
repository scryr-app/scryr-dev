import { Check, Moon, Sun } from "lucide-react";
import styles from "./ThemeSwitcher.module.css";
import { setThemePreset, type ThemeId, ThemePresets, useTheme } from "./theme";

/** Shared chooser: brightness and the entire scene style travel together. */
export function ThemeOptions({ onSelect }: { onSelect?: () => void }) {
	const theme = useTheme();
	return (
		<fieldset aria-label="Diagram theme">
			<legend className={styles.legend}>Theme</legend>
			{(Object.keys(ThemePresets) as ThemeId[]).map((id) => {
				const preset = ThemePresets[id];
				const selected = theme.id === id;
				const Icon = preset.mode === "dark" ? Moon : Sun;
				return (
					<button
						key={id}
						type="button"
						aria-pressed={selected}
						onClick={() => {
							setThemePreset(id);
							onSelect?.();
						}}
						className={styles.option}
					>
						<Icon size={16} className={styles.icon} />
						<span className={styles.content}>
							<span className={styles.name}>{preset.name}</span>
							<span className={styles.description}>
								{preset.mode === "dark" ? "Dark Mode" : "Light Mode"} ·{" "}
								{preset.description}
							</span>
						</span>
						{selected && <Check size={14} aria-label="Selected" />}
					</button>
				);
			})}
		</fieldset>
	);
}
