import { EvidenceCard, type EvidenceCardProps } from "./EvidenceCard";

export function TestsCard(props: EvidenceCardProps) {
	return <EvidenceCard section="tests" {...props} />;
}
