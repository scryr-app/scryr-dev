import { useMemo } from "react";
import {
	type GetBlocksQuery,
	type GetBlocksQueryVariables,
	useGetBlocksQuery,
} from "@/graphql/generated";
import { useRuntimePreviewBlocks } from "@/graphql/runtimePreviewStore";

function normalizeBlocks(data?: GetBlocksQuery) {
	return (data?.blocks ?? []).filter(Boolean).map((block) => ({
		...block,
		name: block.name?.replaceAll("…", "...").replaceAll("—", "-") ?? null,
		connections: (block.connections ?? []).filter(
			(value): value is string => typeof value === "string",
		),
		tags: (block.tags ?? []).filter(
			(value): value is string => typeof value === "string",
		),
	}));
}

export function useBlocksData(variables?: GetBlocksQueryVariables) {
	const preview = useRuntimePreviewBlocks();
	const query = useGetBlocksQuery(variables, {
		enabled: preview === null,
		refetchInterval: 30_000,
	});
	const blocks = useMemo(() => normalizeBlocks(query.data), [query.data]);
	return {
		...query,
		blocks: preview ?? blocks,
		error: preview ? null : query.error,
		isLoading: preview ? false : query.isLoading,
	};
}
