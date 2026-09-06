import { Check, Palette } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { setThemePreset, ThemePresets } from "./theme";

/**
 * Theme Switcher Component
 * Options menu that opens to select themes
 * Positioned at bottom right and reloads page after theme change
 */
export function ThemeSwitcher() {
	const [isOpen, setIsOpen] = useState(false);
	const [isAnimating, setIsAnimating] = useState(false);
	const menuRef = useRef<HTMLDivElement>(null);

	// Get current theme from localStorage or default
	const getCurrentTheme = (): keyof typeof ThemePresets => {
		const stored = localStorage.getItem("selectedTheme");
		if (stored && stored in ThemePresets) {
			return stored as keyof typeof ThemePresets;
		}
		return "AutumnOffice";
	};

	const currentSelectedTheme = getCurrentTheme();

	const handleThemeSelect = (themeName: keyof typeof ThemePresets) => {
		setIsAnimating(true);
		setIsOpen(false);

		// Save to localStorage
		localStorage.setItem("selectedTheme", themeName);

		// Set theme
		setThemePreset(themeName);

		// Reload page after short delay for visual feedback
		setTimeout(() => {
			window.location.reload();
		}, 300);
	};

	const toggleMenu = () => {
		setIsOpen(!isOpen);
	};

	// Load saved theme on mount
	useEffect(() => {
		const stored = localStorage.getItem("selectedTheme");
		if (stored && stored in ThemePresets) {
			setThemePreset(stored as keyof typeof ThemePresets);
		}
	}, []);

	// Close menu when clicking outside
	useEffect(() => {
		const handleClickOutside = (event: MouseEvent) => {
			if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
				setIsOpen(false);
			}
		};

		if (isOpen) {
			document.addEventListener("mousedown", handleClickOutside);
		}

		return () => {
			document.removeEventListener("mousedown", handleClickOutside);
		};
	}, [isOpen]);

	return (
		<div
			ref={menuRef}
			style={{
				position: "fixed",
				bottom: "1.5rem",
				right: "1.5rem",
				zIndex: 1000,
			}}
		>
			{/* Options menu */}
			{isOpen && (
				<div
					style={{
						position: "absolute",
						bottom: "70px",
						right: 0,
						background: "#ffffff",
						border: "2px solid #000000",
						borderRadius: "12px",
						padding: "0.5rem",
						minWidth: "180px",
						boxShadow: "0 8px 24px rgba(0,0,0,0.2)",
						animation: "slideUp 0.2s ease-out",
					}}
				>
					<div
						style={{
							fontSize: "11px",
							fontWeight: "bold",
							padding: "0.5rem 0.75rem",
							color: "#666",
							textTransform: "uppercase",
							letterSpacing: "0.5px",
						}}
					>
						Select Theme
					</div>
					{Object.keys(ThemePresets).map((themeName) => {
						const isSelected = themeName === currentSelectedTheme;
						return (
							<button
								type="button"
								key={themeName}
								onClick={() =>
									handleThemeSelect(themeName as keyof typeof ThemePresets)
								}
								style={{
									display: "block",
									width: "100%",
									padding: "0.625rem 0.75rem",
									background: isSelected ? "#000000" : "#ffffff",
									color: isSelected ? "#ffffff" : "#000000",
									border: "none",
									textAlign: "left",
									cursor: "pointer",
									fontSize: "13px",
									fontWeight: isSelected ? "600" : "normal",
									borderRadius: "6px",
									marginBottom: "0.25rem",
									transition: "all 0.15s ease",
								}}
								onMouseEnter={(e) => {
									if (!isSelected) {
										e.currentTarget.style.background = "#f5f5f5";
									}
								}}
								onMouseLeave={(e) => {
									if (!isSelected) {
										e.currentTarget.style.background = "#ffffff";
									}
								}}
							>
								{themeName}
								{isSelected && <Check size={12} style={{ float: "right" }} />}
							</button>
						);
					})}
				</div>
			)}

			{/* Icon button */}
			<button
				type="button"
				onClick={toggleMenu}
				style={{
					width: "56px",
					height: "56px",
					borderRadius: "50%",
					background: "#ffffff",
					border: "2px solid #000000",
					cursor: "pointer",
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					padding: 0,
					boxShadow: isOpen
						? "0 6px 16px rgba(0,0,0,0.25)"
						: "0 4px 12px rgba(0,0,0,0.15)",
					transition: "all 0.3s ease",
					transform: isAnimating
						? "rotate(180deg) scale(0.9)"
						: isOpen
							? "scale(1.05)"
							: "rotate(0deg) scale(1)",
				}}
				onMouseEnter={(e) => {
					if (!isAnimating && !isOpen) {
						e.currentTarget.style.transform = "scale(1.1)";
						e.currentTarget.style.boxShadow = "0 6px 16px rgba(0,0,0,0.25)";
					}
				}}
				onMouseLeave={(e) => {
					if (!isAnimating && !isOpen) {
						e.currentTarget.style.transform = "scale(1)";
						e.currentTarget.style.boxShadow = "0 4px 12px rgba(0,0,0,0.15)";
					}
				}}
				title="Theme Options"
			>
				{/* Palette icon */}
				<Palette size={28} stroke="#000000" strokeWidth={2} />
			</button>

			{/* CSS animation */}
			<style>
				{`
					@keyframes slideUp {
						from {
							opacity: 0;
							transform: translateY(10px);
						}
						to {
							opacity: 1;
							transform: translateY(0);
						}
					}
				`}
			</style>
		</div>
	);
}
