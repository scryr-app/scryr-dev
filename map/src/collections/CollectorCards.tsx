import { useEffect, useState } from "react";
import { isLocalAuthMode } from "@/auth/env";
import {
	type GetCollectorCardsQuery,
	useGetCollectorCardsQuery,
} from "@/graphql/generated";
import { useSelectedMap } from "@/graphql/sampleStore";
import { useBlocksData } from "@/graphql/useBlocksData";
import { useTheme } from "@/theme/theme";
import styles from "./CollectorCards.module.css";

type Card = GetCollectorCardsQuery["collectorCards"][number];

function display(value: unknown): string {
	if (value === null || value === undefined) return "—";
	return typeof value === "object" ? JSON.stringify(value) : String(value);
}

function Fields({ data }: { data: unknown }) {
	if (data === null || data === undefined) return null;
	const entries: Array<[string, unknown]> =
		typeof data === "object" && !Array.isArray(data)
			? Object.entries(data)
			: [["value", data]];
	return (
		<dl className={styles.fields}>
			{entries.map(([key, value]) => (
				<div key={key}>
					<dt>{key}</dt>
					<dd>{display(value)}</dd>
				</div>
			))}
		</dl>
	);
}

function Timestamp({ value }: { value: string }) {
	return <time dateTime={value}>{new Date(value).toLocaleString()}</time>;
}

function ObservationCard({ card, now }: { card: Card; now: number }) {
	const successful = card.lastSuccess;
	const failed = card.latest && card.latest.status !== "success";
	const stale =
		successful &&
		now - Date.parse(successful.finishedAt) > card.staleAfterSeconds * 1000;
	return (
		<article className={styles.card} aria-label={card.title}>
			<h3>{card.title}</h3>
			<p className={styles.meta}>
				{card.environment} · {card.collectorId}
			</p>
			<p className={styles.badge}>
				{failed
					? "Collection failed"
					: !successful
						? "Awaiting first observation"
						: stale
							? "Stale observation"
							: "Observation current"}
			</p>
			{failed && (
				<p className={styles.failure}>
					{card.latest?.status}:{" "}
					{card.latest?.error ?? "No valid observation was produced."}
				</p>
			)}
			{failed && card.latest && (
				<p className={styles.meta}>
					Latest attempt: <Timestamp value={card.latest.finishedAt} />
				</p>
			)}
			{successful && (
				<p className={styles.meta}>
					{failed ? "Last successful collection" : "Observed"}:{" "}
					<Timestamp value={successful.finishedAt} />
					{stale && failed ? " · Stale" : ""}
				</p>
			)}
			{card.kind === "history" ? (
				<ol className={styles.history}>
					{card.history.map((item) => (
						<li key={item.id}>
							<p>
								<Timestamp value={item.finishedAt} /> · {item.status}
							</p>
							{item.error && <p>{item.error}</p>}
							<Fields data={item.data} />
						</li>
					))}
				</ol>
			) : card.kind === "table" && successful ? (
				<table className={styles.table}>
					<caption>Last successful observation</caption>
					<thead>
						<tr>
							<th scope="col">Field</th>
							<th scope="col">Value</th>
						</tr>
					</thead>
					<tbody>
						{Object.entries(successful.data ?? {}).map(([key, value]) => (
							<tr key={key}>
								<th scope="row">{key}</th>
								<td>{display(value)}</td>
							</tr>
						))}
					</tbody>
				</table>
			) : (
				<Fields data={successful?.data} />
			)}
		</article>
	);
}

export function CollectorCards({
	isEditorOpen,
	onExpand,
}: {
	isEditorOpen: boolean;
	onExpand: () => void;
}) {
	const theme = useTheme();
	const [open, setOpen] = useState(false);
	const [now, setNow] = useState(Date.now);
	useEffect(() => {
		if (!isLocalAuthMode || !open || isEditorOpen) return;
		const timer = window.setInterval(() => setNow(Date.now()), 5_000);
		return () => window.clearInterval(timer);
	}, [open, isEditorOpen]);
	const selected = useSelectedMap();
	const { blocks } = useBlocksData(
		selected.id
			? { scryIdentifier: selected.id }
			: { sample: selected.key || undefined },
	);
	const query = useGetCollectorCardsQuery(undefined, {
		enabled: isLocalAuthMode,
		refetchInterval: 5_000,
	});
	const targets = new Set(blocks.map((block) => block.manifestId));
	const cards =
		query.data?.collectorCards.filter((card) => targets.has(card.target)) ?? [];
	if (
		!isLocalAuthMode ||
		(!query.isLoading && !query.error && cards.length === 0)
	)
		return null;
	const expanded = open && !isEditorOpen;
	return (
		<aside
			className={styles.panel}
			aria-label="Collector observations"
			style={{
				background: theme.console.background,
				color: theme.console.text,
			}}
		>
			<button
				className={styles.toggle}
				type="button"
				aria-expanded={expanded}
				aria-controls="collector-observations"
				onClick={() => {
					setNow(Date.now());
					setOpen(!expanded);
					if (!expanded) onExpand();
				}}
			>
				Observations{cards.length > 0 ? ` (${cards.length})` : ""}{" "}
				<span aria-hidden="true">{expanded ? "−" : "+"}</span>
			</button>
			{expanded && (
				<div id="collector-observations" className={styles.content}>
					<p className={styles.meta}>
						Collected evidence · refreshes every 5 seconds
					</p>
					<p className={styles.meta}>
						Definitions are pinned at server startup. Restart the server to
						activate edits.
					</p>
					{query.isLoading && <p role="status">Loading observations…</p>}
					{Boolean(query.error) && (
						<p role="alert">
							Could not refresh observations. Displayed results may be out of
							date.
						</p>
					)}
					{cards.map((card) => (
						<ObservationCard key={card.id} card={card} now={now} />
					))}
				</div>
			)}
		</aside>
	);
}
