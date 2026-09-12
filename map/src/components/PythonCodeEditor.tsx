import { autocompletion } from "@codemirror/autocomplete";
import {
	defaultKeymap,
	history,
	historyKeymap,
	indentWithTab,
} from "@codemirror/commands";
import { python } from "@codemirror/lang-python";
import { indentOnInput } from "@codemirror/language";
import { Compartment, EditorState } from "@codemirror/state";
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
import type { Theme } from "@/theme/theme";
import { createThemeExtensions } from "./pythonEditorTheme";

interface PythonCodeEditorProps {
	value: string;
	onChange: (value: string) => void;
	theme: Theme;
}

export interface PythonCodeEditorHandle {
	moveToLine: (lineNumber: number) => void;
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
