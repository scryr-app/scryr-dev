import { expect, it } from "vitest";
import { runtimeMetricLines } from "./RuntimeMetricsCard";

it("shows real latency but leaves absent CPU and memory unavailable", () => {
	const snapshot = {
		status: "ready" as const,
		values: {
			responseTimeP95: { value: 125, unit: "ms", evaluatedAt: 10, samples: [] },
		},
	};
	expect(runtimeMetricLines(snapshot, false)).toContain(
		"P95 latency: 125.00 ms",
	);
	expect(runtimeMetricLines(snapshot, true)).toEqual([
		"CPU current: unavailable",
		"CPU average: unavailable",
		"CPU peak: unavailable",
		"Memory usage: unavailable",
	]);
});
