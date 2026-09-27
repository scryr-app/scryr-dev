import { CompletionContext } from "@codemirror/autocomplete";
import { python } from "@codemirror/lang-python";
import { EditorState } from "@codemirror/state";
import { expect, it, vi } from "vitest";

vi.mock("./sdkSources", () => ({
	browserSdkFiles: {
		"scryr/__init__.py":
			'__all__ = ["Manifest", "SyftInventoryCollector", "OpenMetricsCollector"]',
		"scryr/collectors.py":
			'class Collector(BaseModel):\n    id: str = ""\n    schedule: Schedule = Field(default_factory=Schedule)\nclass SyftInventoryCollector(Collector):\n    kind: str = "syft_inventory"\n    directory: str = "."\nclass OpenMetricsCollector(Collector):\n    endpoint: str\n',
		"scryr/manifest.py":
			"class Manifest(BaseModel):\n    def __init__(\n        self,\n        dependencies: list[DependencyCollector] = [],\n        metrics: list[MetricCollector] = [],\n    ):\n        pass\n",
	},
}));

import {
	pythonSdkCompletionSource,
	sdkCompletionRegistry,
} from "./pythonSdkCompletions";

function complete(source: string) {
	const state = EditorState.create({ doc: source, extensions: [python()] });
	return pythonSdkCompletionSource(
		new CompletionContext(state, source.length, true),
	);
}
it("derives concrete collector names from SDK exports, retaining inherited typed fields", () => {
	expect(complete("SyftInv")?.options.map((item) => item.label)).toContain(
		"SyftInventoryCollector",
	);
	const fields = complete("SyftInventoryCollector(sch")?.options;
	expect(fields?.find((item) => item.label === "schedule")).toMatchObject({
		apply: "schedule=",
		detail: "Schedule",
	});
	expect(complete("SyftInventoryCollector(kind")?.options ?? []).toEqual([]);
});
it("offers section lists and integration parameters at their specific constructor", () => {
	expect(
		complete("Manifest(dep")?.options.find(
			(item) => item.label === "dependencies",
		),
	).toMatchObject({ detail: "list[DependencyCollector]" });
	expect(
		complete("Manifest(metrics=[OpenMetricsCollector(end")?.options.find(
			(item) => item.label === "endpoint",
		),
	).toMatchObject({ detail: "str" });
});
it("does not suggest removed classes that are no longer exported by the SDK", () => {
	const registry = sdkCompletionRegistry({
		"scryr/__init__.py": '__all__ = ["NewCollector"]',
		"scryr/old.py":
			"class RemovedCollector(BaseModel):\n    endpoint: str\nclass NewCollector(BaseModel):\n    path: str\n",
	});
	expect(Array.from(registry.keys())).toEqual(["NewCollector"]);
});
