import { expect, it } from "vite-plus/test";
import type { GetBlocksQuery } from "./generated";
import { normalizeBlock } from "./useBlocksData";

function block(
	overrides: Partial<GetBlocksQuery["blocks"][number]> = {},
): GetBlocksQuery["blocks"][number] {
	return {
		name: "API",
		description: null,
		version: null,
		lineNumber: null,
		consumerType: null,
		icon: null,
		sourceCodeUrl: null,
		owners: null,
		cicdTool: null,
		connections: [],
		languages: [],
		frameworks: [],
		deployments: [],
		metricTools: [],
		docs: [],
		tags: [],
		links: [],
		rawJsonString: "{}",
		...overrides,
	};
}

it("preserves plural enum arrays supplied by GraphQL", () => {
	const normalized = normalizeBlock(
		block({
			languages: ["typescript", "python"],
			deployments: ["ec2", "terraform", "pulumi"],
			metricTools: ["datadog", "loki", "jaeger"],
		}),
	);
	expect(normalized.languages).toEqual(["typescript", "python"]);
	expect(normalized.deployments).toEqual(["ec2", "terraform", "pulumi"]);
	expect(normalized.metricTools).toEqual(["datadog", "loki", "jaeger"]);
});

it("does not repopulate explicitly empty lists from raw JSON", () => {
	const normalized = normalizeBlock(
		block({
			rawJsonString: JSON.stringify({
				info: {
					languages: ["python"],
					deployments: ["ec2"],
					metricTools: ["datadog"],
					language: "javascript",
					deployment: "lambda",
					authType: "jwt",
				},
			}),
		}),
	);
	expect(normalized.languages).toEqual([]);
	expect(normalized.deployments).toEqual([]);
	expect(normalized.metricTools).toEqual([]);
	expect(normalized).not.toHaveProperty("authType");
});
