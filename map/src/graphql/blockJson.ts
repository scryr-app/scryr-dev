export type RawRecord = Record<string, unknown>;

export function asRecord(value: unknown): RawRecord | undefined {
	return value && typeof value === "object" && !Array.isArray(value)
		? (value as RawRecord)
		: undefined;
}

export function parseRawJsonString(
	rawJsonString?: string | null,
): RawRecord | undefined {
	if (!rawJsonString) {
		return undefined;
	}

	try {
		return asRecord(JSON.parse(rawJsonString));
	} catch {
		return undefined;
	}
}

export function getValue(
	source: RawRecord | undefined,
	path: string[],
): unknown {
	let current: unknown = source;

	for (const part of path) {
		const record = asRecord(current);
		if (!record) {
			return undefined;
		}
		current = record[part];
	}

	return current;
}

export function firstString(
	source: RawRecord | undefined,
	paths: string[][],
): string | undefined {
	for (const path of paths) {
		const value = getValue(source, path);
		if (typeof value === "string" && value.length > 0) {
			return value;
		}
	}

	return undefined;
}

export function firstNumber(
	source: RawRecord | undefined,
	paths: string[][],
): number | undefined {
	for (const path of paths) {
		const value = getValue(source, path);
		if (typeof value === "number" && Number.isFinite(value)) {
			return value;
		}
	}

	return undefined;
}

export function firstBoolean(
	source: RawRecord | undefined,
	paths: string[][],
): boolean | undefined {
	for (const path of paths) {
		const value = getValue(source, path);
		if (typeof value === "boolean") {
			return value;
		}
	}

	return undefined;
}
