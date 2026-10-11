import { useMemo } from "react";
import {
	type GetBlocksQueryVariables,
	useGetBlocksQuery,
} from "@/graphql/generated";
import { useRuntimePreviewBlocks } from "@/graphql/runtimePreviewStore";
import { normalizeBlocksData } from "./blockViewModel";
import {
	type RuntimeMetricSnapshot,
	useDiagramMetrics,
} from "./useDiagramMetrics";

export function useBlocksData(variables?: GetBlocksQueryVariables) {
	const runtimePreviewBlocks = useRuntimePreviewBlocks();
	const query = useGetBlocksQuery(variables, {
		enabled: runtimePreviewBlocks === null,
		refetchInterval: 30_000,
	});
	const normalized = useMemo(
		() =>
			normalizeBlocksData(
				runtimePreviewBlocks === null
					? query.data
					: { blocks: runtimePreviewBlocks },
			),
		[query.data, runtimePreviewBlocks],
	);
	const configured = normalized.some(
		(block) =>
			block.runtimeConfiguration.metrics ||
			block.runtimeConfiguration.analytics,
	);
	const runtime = useDiagramMetrics(
		variables,
		runtimePreviewBlocks === null && configured,
	);
	const blocks = useMemo(
		() =>
			normalized.map((block) => {
				if (
					runtimePreviewBlocks !== null ||
					(!block.runtimeConfiguration.metrics &&
						!block.runtimeConfiguration.analytics)
				)
					return block;
				const unavailable: RuntimeMetricSnapshot = {
					status: runtime.error
						? "error"
						: runtime.isFetching
							? "loading"
							: "no_data",
					error: runtime.error
						? "Unable to fetch diagram observations"
						: undefined,
					values: {},
				};
				const snapshot = runtime.data?.diagramMetrics[block.manifestId];
				return {
					...block,
					observations: {
						metrics: block.runtimeConfiguration.metrics
							? (snapshot ?? unavailable)
							: undefined,
						analytics: block.runtimeConfiguration.analytics
							? { ...(snapshot?.analytics ?? unavailable), source: "posthog" }
							: undefined,
					},
				};
			}),
		[
			normalized,
			runtime.data,
			runtime.error,
			runtime.isFetching,
			runtimePreviewBlocks,
		],
	);

	return {
		...query,
		blocks,
		error: runtimePreviewBlocks ? null : query.error,
		isLoading: runtimePreviewBlocks ? false : query.isLoading,
	};
}
