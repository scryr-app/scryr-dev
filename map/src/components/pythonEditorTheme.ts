import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { tags } from "@lezer/highlight";
import { consoleTextColor } from "@/theme/console";
import type { Theme } from "@/theme/theme";

export function createThemeExtensions(theme: Theme) {
	const palette = theme.console;
	const ink = consoleTextColor;
	return [
		EditorView.theme(
			{
				"&": {
					height: "100%",
					backgroundColor: palette.background,
					color: ink(palette.text),
					fontSize: "12px",
				},
				".cm-scroller": {
					fontFamily:
						'"SFMono-Regular", "SF Mono", "Cascadia Code", "Fira Code", Menlo, Consolas, monospace',
					lineHeight: "20px",
					overflow: "auto",
				},
				".cm-content": {
					minHeight: "100%",
					padding: "0",
					caretColor: palette.accent,
				},
				".cm-line": { padding: "0 0 0 8px" },
				"&.cm-focused": { outline: "none" },
				".cm-gutters": {
					backgroundColor: palette.background,
					border: "none",
					color: ink(palette.muted),
				},
				".cm-activeLine": { backgroundColor: palette.activeLine },
				".cm-activeLineGutter": { backgroundColor: "transparent" },
				".cm-selectionBackground, &.cm-focused .cm-selectionBackground, ::selection":
					{ backgroundColor: palette.selection },
				".cm-cursor, .cm-dropCursor": { borderLeftColor: palette.accent },
				".cm-tooltip": {
					border: `1px solid ${palette.muted}`,
					backgroundColor: palette.background,
					color: ink(palette.text),
				},
				".cm-tooltip-autocomplete ul li[aria-selected]": {
					backgroundColor: palette.selection,
					color: ink(palette.text),
				},
				".cm-tooltip-autocomplete": { maxWidth: "320px" },
			},
			{ dark: theme.isDarkDiagram },
		),
		syntaxHighlighting(
			HighlightStyle.define([
				{ tag: tags.comment, color: ink(palette.comment), fontStyle: "italic" },
				{ tag: tags.keyword, color: ink(palette.keyword) },
				{ tag: [tags.string, tags.regexp], color: ink(palette.string) },
				{
					tag: [tags.number, tags.bool, tags.null],
					color: ink(palette.number),
				},
				{
					tag: [tags.typeName, tags.className, tags.namespace],
					color: ink(palette.type),
				},
				{
					tag: [
						tags.function(tags.variableName),
						tags.function(tags.propertyName),
					],
					color: ink(palette.function),
				},
				{
					tag: [tags.operator, tags.punctuation, tags.meta],
					color: ink(palette.operator),
				},
				{
					tag: tags.invalid,
					color: ink(palette.number),
					textDecoration: "underline wavy",
				},
			]),
		),
	];
}
