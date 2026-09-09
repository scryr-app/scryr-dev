import { useQuery } from "@tanstack/react-query";
import { graphqlFetcher } from "./client";
import type { GetBlocksQueryVariables } from "./generated";

export interface RuntimeMetricSnapshot {
	analytics?: RuntimeMetricSnapshot;
	source?: string;
	windowStart?: number;
	windowEnd?: number;
	status: "loading" | "ready" | "partial" | "no_data" | "stale" | "error";
	error?: string;
	fetchedAt?: string;
	dashboardUrl?: string;
	environment?: string;
	values: Record<
		string,
		{
			label?: string;
			value: number;
			unit?: string;
			evaluatedAt: number;
			samples: [number, number][];
		}
	>;
	missing?: string[];
}
export const diagramMetricsOptions = {
	staleTime: 0,
	refetchOnMount: "always" as const,
	refetchOnWindowFocus: false,
	refetchOnReconnect: false,
	refetchInterval: false as const,
	retry: false,
	gcTime: 0,
};
export function useDiagramMetrics(
	variables: GetBlocksQueryVariables | undefined,
	enabled: boolean,
) {
	return useQuery({
		queryKey: ["diagramMetrics", variables ?? {}],
		queryFn: graphqlFetcher<
			{ diagramMetrics: Record<string, RuntimeMetricSnapshot> },
			GetBlocksQueryVariables
		>(
			"query DiagramMetrics($scryIdentifier: String, $sample: String) { diagramMetrics(scryIdentifier: $scryIdentifier, sample: $sample) }",
			variables,
		),
		enabled,
		...diagramMetricsOptions,
	});
}
