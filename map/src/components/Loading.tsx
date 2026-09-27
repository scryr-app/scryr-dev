import { LoaderCircle } from "lucide-react";

export function Loading() {
	return (
		<span role="status" className="inline-flex items-center gap-2">
			<LoaderCircle
				size={18}
				aria-hidden="true"
				className="motion-safe:animate-spin"
			/>
			Loading…
		</span>
	);
}
