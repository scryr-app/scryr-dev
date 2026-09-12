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

export const celestialConsole: ConsolePalette = {
	background: "#071225",
	panel: "rgba(7,18,37,0.93)",
	text: "#edf2e7",
	muted: "#91a2bd",
	comment: "#7f91ab",
	keyword: "#e7c77f",
	string: "#a8d7cf",
	number: "#e8ad78",
	type: "#9dbde8",
	function: "#d9c990",
	operator: "#bac9dc",
	accent: "#d8b66d",
	selection: "rgba(216,182,109,0.24)",
	activeLine: "rgba(130,160,204,0.09)",
};

export const porcelainConsole: ConsolePalette = {
	background: "#fffaf0",
	panel: "rgba(255,250,240,0.94)",
	text: "#17385f",
	muted: "#68778a",
	comment: "#778779",
	keyword: "#1f5592",
	string: "#477a68",
	number: "#9c672d",
	type: "#356f9b",
	function: "#7d4f74",
	operator: "#8c713d",
	accent: "#b28a3f",
	selection: "rgba(37,91,148,0.18)",
	activeLine: "rgba(37,91,148,0.055)",
};

export const sunkenConsole: ConsolePalette = {
	background: "#09343b",
	panel: "rgba(9,52,59,0.93)",
	text: "#e1f3eb",
	muted: "#9fc4be",
	comment: "#8fba9e",
	keyword: "#e0b68f",
	string: "#8fe0bd",
	number: "#efbd80",
	type: "#80cbea",
	function: "#d7a8bd",
	operator: "#b4c8df",
	accent: "#83dfd0",
	selection: "rgba(91,190,175,0.22)",
	activeLine: "rgba(91,190,175,0.075)",
};

export const velvetConsole: ConsolePalette = {
	background: "#fff5e8",
	panel: "rgba(255,245,232,0.94)",
	text: "#421936",
	muted: "#806276",
	comment: "#817263",
	keyword: "#852b53",
	string: "#557246",
	number: "#a45d36",
	type: "#654d87",
	function: "#76365e",
	operator: "#96713f",
	accent: "#a27635",
	selection: "rgba(121,43,78,0.18)",
	activeLine: "rgba(121,43,78,0.055)",
};
