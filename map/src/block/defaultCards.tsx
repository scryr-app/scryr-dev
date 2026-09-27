import type { ReactNode } from "react";
import {
	ChecksCard,
	DependenciesCard,
	MetricsCard,
	PerformanceCard,
	RepositoryCard,
	TestsCard,
} from "@/cards";
import type { BlockCardData } from "@/cards/blockCardData";
import { EVIDENCE_SECTIONS } from "@/cards/evidence";
export type BlockCardGroup = Array<{ components: ReactNode[] }>;
const cards = {
	repository: RepositoryCard,
	checks: ChecksCard,
	metrics: MetricsCard,
	tests: TestsCard,
	dependencies: DependenciesCard,
	performance: PerformanceCard,
};
export function createBlockDataCards(data: BlockCardData): BlockCardGroup {
	return EVIDENCE_SECTIONS.map((section) => {
		const Card = cards[section];
		return {
			components: data[section].length
				? [<Card key={section} collectors={data[section]} />]
				: [],
		};
	});
}
