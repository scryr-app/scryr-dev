import { useContext, useState } from "react";
import {
	useGetEvidenceObservationQuery,
	useGetInventoryPackagesQuery,
} from "@/graphql/generated";
import { EditorScope } from "@/graphql/useManifestEditor";
import {
	type DependencyPanel,
	type DetailPage,
	EvidenceResultDetails,
	matchingInventoryObservation,
} from "./EvidenceResultDetails";
import type { CollectorEvidence, Observation } from "./evidence";

/** Detail pages are pinned to one immutable observation, never a changing latest result. */
export function DependencyDetails({
	observation,
	collectors,
	panel,
}: {
	observation: Observation;
	collectors: CollectorEvidence[];
	panel: DependencyPanel;
}) {
	const scope = useContext(EditorScope);
	const [page, setPage] = useState<DetailPage>({
		search: "",
		filter: "",
		offset: 0,
		limit: 25,
	});
	const variables = {
		observationId: observation.observationId,
		workspaceId: observation.workspaceId,
		...page,
	};
	const detail = useGetEvidenceObservationQuery(variables, {
		queryKey: ["GetEvidenceObservation", variables, scope],
		staleTime: Infinity,
		placeholderData: (previous) => previous,
	});

	const selected = detail.data?.evidenceObservation;
	const result = selected?.result;
	const matchingTotal =
		result?.__typename === "InventoryResult"
			? result.matchingPackages
			: result?.__typename === "LicenseResult"
				? result.matchingItems
				: result?.__typename === "VulnerabilityResult"
					? result.matchingFindings
					: 0;
	const inventory = matchingInventoryObservation(observation, collectors);
	const ids = Array.from(
		new Set(
			result?.__typename === "LicenseResult" ||
				result?.__typename === "VulnerabilityResult"
				? result.items.map((item) => item.packageId)
				: [],
		),
	);
	const labelVariables = {
		observationId: inventory?.observationId ?? "",
		workspaceId: observation.workspaceId,
		ids,
	};
	const labels = useGetInventoryPackagesQuery(labelVariables, {
		queryKey: ["GetInventoryPackages", labelVariables, scope],
		enabled: !!inventory && ids.length > 0,
		staleTime: Infinity,
	});

	const labelObservation = labels.data?.evidenceObservation;
	const inventoryHash =
		observation.result.__typename === "LicenseResult" ||
		observation.result.__typename === "VulnerabilityResult"
			? observation.result.inventoryHash
			: undefined;
	const labelResult = labelObservation?.result;
	const matchingLabels =
		labelResult?.__typename === "InventoryResult" &&
		labelObservation?.workspaceId === observation.workspaceId &&
		labelObservation.environment === observation.environment &&
		labelResult.artifactHash === inventoryHash;
	const packageLabels = matchingLabels
		? new Map(
				labelResult.packages.map((pkg) => [
					pkg.id,
					`${pkg.name}@${pkg.version} (${pkg.ecosystem})`,
				]),
			)
		: undefined;
	if (detail.error)
		return (
			<p role="alert" className="text-red-300">
				Detail page could not be loaded.{" "}
				<button
					type="button"
					className="underline"
					onClick={() => void detail.refetch()}
				>
					Retry
				</button>
			</p>
		);
	if (!selected)
		return (
			<p role="status" className="text-slate-300">
				{detail.isLoading
					? "Loading observation details…"
					: "This observation is no longer retained."}
			</p>
		);
	return (
		<EvidenceResultDetails
			observation={selected}
			collectors={collectors}
			dependencyPanel={panel}
			packageLabels={packageLabels}
			remote={{
				...page,
				total: matchingTotal,
				loading: detail.isFetching,
				onChange: setPage,
			}}
		/>
	);
}
