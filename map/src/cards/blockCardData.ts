import type { GetBlocksQuery } from "@/graphql/generated";
import {
	type CollectorEvidence,
	EVIDENCE_SECTIONS,
	type EvidenceSection,
} from "./evidence";

export type BlockCardData = Record<EvidenceSection, CollectorEvidence[]>;
type EvidenceBlock = Pick<
	GetBlocksQuery["blocks"][number],
	"rawJsonString" | "evidence"
>;

/** Config controls presence/order; observations are read only from typed GraphQL. */
export function getBlockCardData(block: EvidenceBlock): BlockCardData {
	let raw: Record<string, unknown> = {};
	try {
		raw = JSON.parse(block.rawJsonString ?? "{}");
	} catch {
		/* An invalid preview has no executable collector declarations. */
	}
	return Object.fromEntries(
		EVIDENCE_SECTIONS.map((section) => {
			const declarations = raw?.[section];
			if (!Array.isArray(declarations)) return [section, []];
			const collectors = declarations.flatMap(
				(declaration): CollectorEvidence[] => {
					if (
						!declaration ||
						typeof declaration !== "object" ||
						typeof declaration.kind !== "string"
					)
						return [];
					const id =
						typeof declaration.id === "string"
							? declaration.id
							: declaration.kind;
					const matches = (block.evidence ?? []).filter(
						(item) =>
							item.section.toLowerCase() === section &&
							item.collectorId === id &&
							item.integration === declaration.kind,
					);
					if (matches.length) return matches;
					return [
						{
							manifestId:
								typeof raw.manifestId === "string" ? raw.manifestId : "",
							section: section.toUpperCase() as CollectorEvidence["section"],
							collectorId: id,
							integration: declaration.kind,
							workspaceId: "",
							state: "WAITING" as CollectorEvidence["state"],
							message:
								"Declared in index.scry. Collection runs in the local CLI.",
							freshnessSeconds:
								typeof declaration.freshness === "number"
									? declaration.freshness
									: 3600,
							stale: false,
							outdated: false,
							latest: null,
							updatedAt: null,
						},
					];
				},
			);
			return [section, collectors];
		}),
	) as BlockCardData;
}
