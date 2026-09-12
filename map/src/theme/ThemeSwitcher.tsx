import { Check, Moon, Sun } from "lucide-react";
import { setThemePreset, type ThemeId, ThemePresets, useTheme } from "./theme";

/** Shared chooser: brightness and the entire scene style travel together. */
export function ThemeOptions({ onSelect }: { onSelect?: () => void }) {
	const theme = useTheme();
	return (
		<fieldset aria-label="Diagram theme">
			<legend className="px-2 py-1 text-[10px] font-semibold uppercase tracking-widest text-white/50">
				Theme
			</legend>
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
						className={`flex w-full items-start gap-3 rounded-lg px-3 py-3 text-left text-white transition-colors ${selected ? "bg-white/15" : "hover:bg-white/10"}`}
					>
						<Icon size={16} className="mt-0.5 shrink-0 text-white/70" />
						<span className="flex-1">
							<span className="block text-xs font-semibold">{preset.name}</span>
							<span className="mt-1 block text-[10px] text-white/60">
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
