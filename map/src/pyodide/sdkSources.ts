// Vite embeds the same SDK sources used by the CLI, including its runtime serializer.
const sources = import.meta.glob("../../../manifest/scryr/src/scryr/**/*.py", {
	query: "?raw",
	import: "default",
	eager: true,
}) as Record<string, string>;

export const browserSdkFiles: Record<string, string> = Object.fromEntries(
	Object.entries(sources).map(([path, content]) => [
		`scryr/${path.split("/src/scryr/")[1]}`,
		content,
	]),
);
