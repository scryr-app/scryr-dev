/**
 * Wraps text into multiple lines based on maximum length per line.
 * Attempts to break at word boundaries.
 */
export function wrapText(text: string, maxLength: number = 25): string[] {
	const words = text.split(" ");
	const lines: string[] = [];
	let currentLine = "";

	for (const word of words) {
		if ((currentLine + word).length > maxLength) {
			if (currentLine) lines.push(currentLine.trim());
			currentLine = `${word} `;
		} else {
			currentLine += `${word} `;
		}
	}
	if (currentLine) lines.push(currentLine.trim());
	return lines;
}
