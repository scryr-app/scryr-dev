import { autocompletion } from "@codemirror/autocomplete";
import {
	defaultKeymap,
	history,
	historyKeymap,
	indentWithTab,
} from "@codemirror/commands";
import { python } from "@codemirror/lang-python";
import {
	defaultHighlightStyle,
	indentOnInput,
	syntaxHighlighting,
} from "@codemirror/language";
import { Compartment, EditorState } from "@codemirror/state";
import { oneDark, oneDarkHighlightStyle } from "@codemirror/theme-one-dark";
import {
	drawSelection,
	dropCursor,
	EditorView,
	highlightActiveLine,
	keymap,
	lineNumbers,
} from "@codemirror/view";
import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { pythonEnumCompletionSource } from "@/pyodide/pythonEnumCompletions";

interface PythonCodeEditorProps {
	value: string;
	onChange: (value: string) => void;
	theme: "dark" | "light";
}

export interface PythonCodeEditorHandle {
	moveToLine: (lineNumber: number) => void;
}

function createEditorTheme(mode: "dark" | "light") {
	const isDark = mode === "dark";

	return EditorView.theme(
		{
			"&": {
				height: "100%",
				backgroundColor: "transparent",
				color: isDark ? "#e2e8f0" : "#0f172a",
				fontSize: "12px",
			},
			".cm-scroller": {
				backgroundColor: "transparent",
				fontFamily:
					'"SFMono-Regular", "SF Mono", "Cascadia Code", "Fira Code", Menlo, Consolas, monospace',
				lineHeight: "20px",
				overflow: "auto",
			},
			".cm-content": {
				minHeight: "100%",
				padding: "0",
				caretColor: isDark ? "#f8fafc" : "#0f172a",
			},
			".cm-line": {
				padding: "0 0 0 8px",
			},
			".cm-focused": {
				outline: "none",
			},
			".cm-editor.cm-focused": {
				outline: "none",
			},
			".cm-gutters": {
				backgroundColor: "transparent",
				border: "none",
				color: isDark ? "rgba(226, 232, 240, 0.35)" : "rgba(15, 23, 42, 0.35)",
			},
			".cm-activeLine": {
				backgroundColor: isDark
					? "rgba(255, 255, 255, 0.04)"
					: "rgba(15, 23, 42, 0.04)",
			},
			".cm-activeLineGutter": {
				backgroundColor: "transparent",
			},
			".cm-selectionBackground, &.cm-focused .cm-selectionBackground, ::selection":
				{
					backgroundColor: isDark
						? "rgba(56, 189, 248, 0.22)"
						: "rgba(14, 165, 233, 0.2)",
				},
			".cm-cursor, .cm-dropCursor": {
				borderLeftColor: isDark ? "#f8fafc" : "#0f172a",
			},
			".cm-tooltip": {
				border: isDark
					? "1px solid rgba(255, 255, 255, 0.12)"
					: "1px solid rgba(15, 23, 42, 0.12)",
				backgroundColor: isDark
					? "rgba(3, 7, 18, 0.92)"
					: "rgba(255, 255, 255, 0.96)",
				backdropFilter: "blur(20px)",
				color: isDark ? "#e2e8f0" : "#0f172a",
			},
			".cm-tooltip-autocomplete ul li[aria-selected]": {
				backgroundColor: isDark
					? "rgba(255, 255, 255, 0.08)"
					: "rgba(15, 23, 42, 0.06)",
			},
			".cm-tooltip-autocomplete": {
				maxWidth: "320px",
			},
		},
		{ dark: isDark },
	);
}

function createThemeExtensions(mode: "dark" | "light") {
	if (mode === "dark") {
		return [
			oneDark,
			syntaxHighlighting(oneDarkHighlightStyle),
			createEditorTheme("dark"),
		];
	}

	return [
		syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
		createEditorTheme("light"),
	];
}

export const PythonCodeEditor = forwardRef<
	PythonCodeEditorHandle,
	PythonCodeEditorProps
>(function PythonCodeEditor({ value, onChange, theme }, ref) {
	const containerRef = useRef<HTMLDivElement>(null);
	const editorViewRef = useRef<EditorView | null>(null);
	const initialValueRef = useRef(value);
	const initialThemeRef = useRef(theme);
	const onChangeRef = useRef(onChange);
	const themeCompartmentRef = useRef(new Compartment());

	onChangeRef.current = onChange;

	useEffect(() => {
		if (!containerRef.current) {
			return;
		}

		const themeCompartment = themeCompartmentRef.current;
		const state = EditorState.create({
			doc: initialValueRef.current,
			extensions: [
				lineNumbers(),
				history(),
				drawSelection(),
				dropCursor(),
				indentOnInput(),
				highlightActiveLine(),
				EditorState.allowMultipleSelections.of(true),
				EditorView.lineWrapping,
				keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
				python(),
				autocompletion({
					activateOnTyping: true,
					override: [pythonEnumCompletionSource],
				}),
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						onChangeRef.current(update.state.doc.toString());
					}
				}),
				themeCompartment.of(createThemeExtensions(initialThemeRef.current)),
			],
		});

		const view = new EditorView({
			state,
			parent: containerRef.current,
		});

		editorViewRef.current = view;

		return () => {
			editorViewRef.current = null;
			view.destroy();
		};
	}, []);

	useEffect(() => {
		const view = editorViewRef.current;
		if (!view) {
			return;
		}

		const currentValue = view.state.doc.toString();
		if (currentValue === value) {
			return;
		}

		view.dispatch({
			changes: {
				from: 0,
				to: currentValue.length,
				insert: value,
			},
		});
	}, [value]);

	useEffect(() => {
		const view = editorViewRef.current;
		if (!view) {
			return;
		}

		view.dispatch({
			effects: themeCompartmentRef.current.reconfigure(
				createThemeExtensions(theme),
			),
		});
	}, [theme]);

	useImperativeHandle(
		ref,
		() => ({
			moveToLine: (lineNumber: number) => {
				const view = editorViewRef.current;
				if (!view || !Number.isFinite(lineNumber) || lineNumber < 1) {
					return;
				}

				const targetLine = Math.min(lineNumber, view.state.doc.lines);
				const line = view.state.doc.line(targetLine);
				view.dispatch({
					selection: { anchor: line.from },
					effects: EditorView.scrollIntoView(line.from, {
						y: "center",
					}),
				});
				view.focus();
			},
		}),
		[],
	);

	return <div ref={containerRef} className="h-full min-h-0 flex-1" />;
});
