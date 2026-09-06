import type {
	Completion,
	CompletionContext,
	CompletionResult,
} from "@codemirror/autocomplete";
import { browserSdkFiles } from "@/pyodide/sdkSources";

type PythonEnumRegistry = Record<string, string[]>;

const CLASS_HEADER = /^class\s+([A-Za-z_][A-Za-z0-9_]*)\((?:Enum|StrEnum)\):$/;
const ENUM_MEMBER = /^\s{4}([A-Za-z_][A-Za-z0-9_]*)\s*=/;
const IDENTIFIER = /[A-Za-z_][A-Za-z0-9_.]*/;

function extractPythonEnums(source: string): PythonEnumRegistry {
	const registry: PythonEnumRegistry = {};
	const lines = source.split("\n");
	let currentEnumName: string | null = null;

	for (const line of lines) {
		const classMatch = line.match(CLASS_HEADER);
		if (classMatch) {
			currentEnumName = classMatch[1];
			registry[currentEnumName] = [];
			continue;
		}

		if (!currentEnumName) {
			continue;
		}

		if (line.trim().length === 0) {
			continue;
		}

		if (!line.startsWith("    ")) {
			currentEnumName = null;
			continue;
		}

		const memberMatch = line.match(ENUM_MEMBER);
		if (memberMatch) {
			registry[currentEnumName].push(memberMatch[1]);
		}
	}

	return registry;
}

function collectPythonEnums(): PythonEnumRegistry {
	const registry: PythonEnumRegistry = {};

	for (const source of Object.values(browserSdkFiles)) {
		const extracted = extractPythonEnums(source);
		for (const [enumName, members] of Object.entries(extracted)) {
			registry[enumName] = members;
		}
	}

	return registry;
}

const pythonEnumRegistry = collectPythonEnums();

function asEnumMemberCompletion(
	enumName: string,
	memberName: string,
	label = memberName,
): Completion {
	return {
		label,
		apply: label,
		detail: enumName,
		type: "enum",
		boost: 100,
	};
}

function getEnumNameCompletions(prefix: string): Completion[] {
	return Object.entries(pythonEnumRegistry)
		.filter(([enumName]) => enumName.startsWith(prefix))
		.flatMap(([enumName, members]) => {
			const completions: Completion[] = [
				{
					label: enumName,
					type: "class",
					detail: `${members.length} values`,
					boost: 90,
				},
			];

			for (const memberName of members) {
				completions.push(
					asEnumMemberCompletion(
						enumName,
						memberName,
						`${enumName}.${memberName}`,
					),
				);
			}

			return completions;
		});
}

function getEnumMemberCompletions(
	enumName: string,
	memberPrefix: string,
): Completion[] {
	const members = pythonEnumRegistry[enumName];
	if (!members) {
		return [];
	}

	return members
		.filter((memberName) => memberName.startsWith(memberPrefix))
		.map((memberName) => asEnumMemberCompletion(enumName, memberName));
}

export function pythonEnumCompletionSource(
	context: CompletionContext,
): CompletionResult | null {
	const before = context.matchBefore(IDENTIFIER);

	if (!before) {
		if (!context.explicit) {
			return null;
		}

		return {
			from: context.pos,
			options: getEnumNameCompletions(""),
			validFor: IDENTIFIER,
		};
	}

	const text = before.text;
	const dotIndex = text.lastIndexOf(".");

	if (dotIndex >= 0) {
		const enumName = text.slice(0, dotIndex);
		const memberPrefix = text.slice(dotIndex + 1);
		const options = getEnumMemberCompletions(enumName, memberPrefix);

		if (options.length === 0) {
			return null;
		}

		return {
			from: before.from + dotIndex + 1,
			options,
			validFor: /^[A-Za-z_][A-Za-z0-9_]*$/,
		};
	}

	const options = getEnumNameCompletions(text);
	if (!context.explicit && options.length === 0) {
		return null;
	}

	return {
		from: before.from,
		options,
		validFor: IDENTIFIER,
	};
}
