import type {
	Completion,
	CompletionContext,
	CompletionResult,
} from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import { browserSdkFiles } from "./sdkSources";

interface SdkClass {
	bases: string[];
	fields: Map<string, string>;
	constructor: Map<string, string>;
}
/** Read the same bundled Python classes executed by preview; never maintain a second SDK schema. */
export function sdkCompletionRegistry(files: Record<string, string>) {
	const classes = new Map<string, SdkClass>();
	const exported = new Set(
		Array.from(
			(
				files["scryr/__init__.py"]?.match(
					/__all__\s*=\s*\[([\s\S]*?)\]/,
				)?.[1] ?? ""
			).matchAll(/["'](\w+)["']/g),
			(match) => match[1],
		),
	);
	for (const source of Object.values(files)) {
		let current: SdkClass | undefined;
		let inConstructor = false;
		for (const line of source.split("\n")) {
			const header = line.match(/^class\s+(\w+)(?:\(([^)]*)\))?:/);
			if (header) {
				current = {
					bases: header[2]?.split(",").map((base) => base.trim()) ?? [],
					fields: new Map(),
					constructor: new Map(),
				};
				classes.set(header[1], current);
				inConstructor = false;
				continue;
			}
			if (!current) continue;
			if (/^\S/.test(line)) {
				current = undefined;
				continue;
			}
			if (/^ {4}def __init__\(/.test(line)) {
				inConstructor = true;
				continue;
			}
			if (inConstructor && /^ {4}\S/.test(line)) inConstructor = false;
			const field = line.match(
				inConstructor
					? /^ {8}(\w+):\s*([^=\n]+?)(?:\s*=|,?$)/
					: /^ {4}(\w+):\s*([^=\n]+?)(?:\s*=|$)/,
			);
			if (
				field &&
				!field[1].startsWith("_") &&
				!["kind", "model_config"].includes(field[1])
			)
				(inConstructor ? current.constructor : current.fields).set(
					field[1],
					field[2].replace(/,$/, "").trim(),
				);
		}
	}
	const fieldsFor = (
		name: string,
		seen = new Set<string>(),
	): Map<string, string> => {
		if (seen.has(name)) return new Map();
		seen.add(name);
		const definition = classes.get(name);
		if (!definition) return new Map();
		if (definition.constructor.size) return definition.constructor;
		return new Map([
			...definition.bases.flatMap((base) => Array.from(fieldsFor(base, seen))),
			...definition.fields,
		]);
	};
	return new Map(
		Array.from(classes.keys())
			.filter((name) => exported.has(name))
			.map((name) => [name, fieldsFor(name)]),
	);
}
const registry = sdkCompletionRegistry(browserSdkFiles);
export function pythonSdkCompletionSource(
	context: CompletionContext,
): CompletionResult | null {
	const before = context.matchBefore(/[A-Za-z_][A-Za-z0-9_]*/);
	if (!before && !context.explicit) return null;
	let node = syntaxTree(context.state).resolveInner(context.pos, -1);
	if (["String", "Comment"].includes(node.name)) return null;
	let className: string | undefined;
	while (node.parent) {
		if (node.name === "ArgList" && node.parent.name === "CallExpression") {
			const target = node.parent.firstChild;
			if (target) className = context.state.sliceDoc(target.from, target.to);
			break;
		}
		node = node.parent;
	}
	const prefix = before?.text ?? "";
	const options: Completion[] = [];
	if (className && registry.has(className)) {
		for (const [name, annotation] of registry.get(className) ?? []) {
			if (name.startsWith(prefix))
				options.push({
					label: name,
					apply: `${name}=`,
					type: "property",
					detail: annotation,
					boost: 110,
				});
		}
	}
	for (const name of registry.keys())
		if (name.startsWith(prefix))
			options.push({
				label: name,
				type: "class",
				detail: "Scryr SDK",
				boost: 95,
			});
	return options.length
		? {
				from: before?.from ?? context.pos,
				options,
				validFor: /^[A-Za-z_][A-Za-z0-9_]*$/,
			}
		: null;
}
