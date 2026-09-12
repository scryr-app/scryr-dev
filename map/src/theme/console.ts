/** Console ink and surfaces belong to the full diagram theme. */
export interface ConsolePalette {
	background: string;
	panel: string;
	text: string;
	muted: string;
	comment: string;
	keyword: string;
	string: string;
	number: string;
	type: string;
	function: string;
	operator: string;
	accent: string;
	selection: string;
	activeLine: string;
}

/** Three percent transparent ink, without fading editor surfaces or selection. */
export function consoleTextColor(hex: string): string {
	const value = Number.parseInt(hex.slice(1), 16);
	return `rgba(${(value >> 16) & 255}, ${(value >> 8) & 255}, ${value & 255}, 0.97)`;
}

export const forestConsole: ConsolePalette = {
	background: "#f7f4e9",
	panel: "rgba(247,244,233,0.92)",
	text: "#293a2e",
	muted: "#637060",
	comment: "#657454",
	keyword: "#87472f",
	string: "#3e672d",
	number: "#8b5723",
	type: "#32636c",
	function: "#675338",
	operator: "#4f6055",
	accent: "#3c6849",
	selection: "rgba(90,130,68,0.22)",
	activeLine: "rgba(90,130,68,0.07)",
};

export const neonConsole: ConsolePalette = {
	background: "#141020",
	panel: "rgba(20,16,32,0.9)",
	text: "#e9e1ff",
	muted: "#a79bbf",
	comment: "#998cac",
	keyword: "#ce9eff",
	string: "#9edbd1",
	number: "#f4bf8b",
	type: "#91d7f2",
	function: "#b6bcff",
	operator: "#d8b4ef",
	accent: "#c3a0ff",
	selection: "rgba(177,126,242,0.26)",
	activeLine: "rgba(177,126,242,0.08)",
};

export const grimoireConsole: ConsolePalette = {
	background: "#272119",
	panel: "rgba(39,33,25,0.92)",
	text: "#f3e5c8",
	muted: "#beac8c",
	comment: "#a8b08b",
	keyword: "#e4bb7e",
	string: "#bdcc9c",
	number: "#dfa28a",
	type: "#a9c8c4",
	function: "#ddc69b",
	operator: "#cbb7d8",
	accent: "#e2c084",
	selection: "rgba(207,173,105,0.24)",
	activeLine: "rgba(207,173,105,0.08)",
};
