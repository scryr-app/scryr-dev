import { Canvas } from "@react-three/fiber";
import { ArrowLeft, Maximize2, Plus, X } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useCardFocus } from "./CardFocusContext";
import styles from "./CardFocusView.module.css";

type Surface = {
	id: string;
	x: number;
	y: number;
	width: number;
	height: number;
	color: string;
};

export function CardFocusView() {
	const { focusedIds } = useCardFocus();
	return focusedIds.length > 0 ? <OpenCardFocusView /> : null;
}

function OpenCardFocusView() {
	const { cards, focusedIds, expandedId, addCard, removeCard, expandCard, closeFocus } =
		useCardFocus();
	const dialog = useRef<HTMLDialogElement>(null);
	const stage = useRef<HTMLDivElement>(null);
	const addButton = useRef<HTMLButtonElement>(null);
	const searchInput = useRef<HTMLInputElement>(null);
	const backToGroupButton = useRef<HTMLButtonElement>(null);
	const previousExpandedId = useRef<string | null>(null);
	const [pickerOpen, setPickerOpen] = useState(false);
	const [query, setQuery] = useState("");
	const [surfaces, setSurfaces] = useState<Surface[]>([]);
	const isOpen = focusedIds.length > 0;
	const selected = focusedIds.flatMap((id) => {
		const card = cards.find((candidate) => candidate.id === id);
		return card && (!expandedId || expandedId === id) ? [card] : [];
	});
	const available = cards.filter(
		(card) =>
			!focusedIds.includes(card.id) &&
			`${card.label} ${card.blockId}`.toLowerCase().includes(query.toLowerCase()),
	);

	useEffect(() => {
		const element = dialog.current;
		if (!element || !isOpen) return;
		const previousFocus = document.activeElement;
		element.showModal();
		return () => {
			element.close();
			if (previousFocus instanceof HTMLElement && previousFocus.isConnected) previousFocus.focus();
		};
	}, [isOpen]);

	useEffect(() => {
		if (pickerOpen) searchInput.current?.focus();
	}, [pickerOpen]);

	useEffect(() => {
		const previous = previousExpandedId.current;
		previousExpandedId.current = expandedId;
		if (previous === expandedId) return;
		if (pickerOpen) {
			searchInput.current?.focus();
		} else if (expandedId) {
			backToGroupButton.current?.focus();
		} else {
			const expandButton = Array.from(
				dialog.current?.querySelectorAll<HTMLButtonElement>("[data-expand-card]") ?? [],
			).find((button) => button.dataset.expandCard === previous);
			(expandButton ?? addButton.current)?.focus();
		}
	}, [expandedId, pickerOpen]);

	const measure = useCallback(() => {
		const element = stage.current;
		if (!element) return;
		const bounds = element.getBoundingClientRect();
		setSurfaces(
			Array.from(element.querySelectorAll<HTMLElement>("[data-focus-card]")).map((article) => {
				const rect = article.getBoundingClientRect();
				return {
					id: article.dataset.focusCard ?? "",
					color: article.dataset.color ?? "#777777",
					x: rect.left - bounds.left + rect.width / 2 - bounds.width / 2,
					y: bounds.height / 2 - (rect.top - bounds.top + rect.height / 2),
					width: rect.width,
					height: rect.height,
				};
			}),
		);
	}, []);

	// biome-ignore lint/correctness/useExhaustiveDependencies: Reobserve the article elements when the visible card set changes.
	useEffect(() => {
		if (!isOpen) return;
		const element = stage.current;
		if (!element) return;
		const observer = new ResizeObserver(measure);
		observer.observe(element);
		for (const article of element.querySelectorAll("[data-focus-card]")) observer.observe(article);
		measure();
		return () => observer.disconnect();
	}, [isOpen, focusedIds, expandedId, measure]);

	function dismissPicker() {
		setPickerOpen(false);
		setQuery("");
		addButton.current?.focus();
	}

	if (!isOpen) return null;

	return (
		<dialog
			ref={dialog}
			className={styles.dialog}
			aria-labelledby="card-focus-title"
			onCancel={(event) => {
				event.preventDefault();
				if (pickerOpen) dismissPicker();
				else if (expandedId) expandCard(null);
				else closeFocus();
			}}
		>
			<header className={styles.toolbar}>
				<button type="button" onClick={closeFocus}>
					<ArrowLeft size={18} />
					Back to diagram
				</button>
				<div className={styles.title}>
					<h2 id="card-focus-title">Focus mode</h2>
					<span>
						{focusedIds.length} {focusedIds.length === 1 ? "card" : "cards"}
					</span>
				</div>
				{expandedId && (
					<button ref={backToGroupButton} type="button" onClick={() => expandCard(null)}>
						Back to group
					</button>
				)}
				<button
					ref={addButton}
					type="button"
					onClick={() => setPickerOpen((value) => !value)}
					aria-expanded={pickerOpen}
					aria-controls="focus-card-picker"
				>
					<Plus size={18} />
					Add card
				</button>
			</header>
			{pickerOpen && (
				<section id="focus-card-picker" className={styles.picker} aria-label="Add cards to focus">
					<div className={styles.searchRow}>
						<label htmlFor="focus-card-search">Find a card</label>
						<button type="button" aria-label="Close card picker" onClick={dismissPicker}>
							<X size={18} />
						</button>
					</div>
					<input
						ref={searchInput}
						id="focus-card-search"
						type="search"
						value={query}
						onChange={(event) => setQuery(event.target.value)}
						placeholder="Search blocks and cards…"
					/>
					<ul>
						{available.map((card) => (
							<li key={card.id}>
								<button
									type="button"
									onClick={() => {
										addCard(card.id);
										expandCard(null);
										searchInput.current?.focus();
									}}
								>
									<Plus size={16} />
									<span>
										{card.blockId} — {card.label}
									</span>
								</button>
							</li>
						))}
					</ul>
					{available.length === 0 && (
						<p>{query ? "No matching cards." : "All cards are in focus."}</p>
					)}
				</section>
			)}
			<div ref={stage} className={styles.stage}>
				<div className={styles.canvas} aria-hidden="true">
					<Canvas
						orthographic
						camera={{ position: [0, 0, 100], zoom: 1, near: 0.1, far: 200 }}
						frameloop="demand"
						gl={{ alpha: true, antialias: true }}
					>
						{surfaces.map((surface) => (
							<mesh key={surface.id} position={[surface.x, surface.y, 0]}>
								<boxGeometry args={[surface.width + 6, surface.height + 6, 8]} />
								<meshBasicMaterial color={surface.color} />
							</mesh>
						))}
					</Canvas>
				</div>
				<div className={styles.grid} data-single={selected.length === 1} onScroll={measure}>
					{selected.map((card) => (
						<article
							key={card.id}
							data-focus-card={card.id}
							data-color={card.color}
							className={styles.card}
							aria-label={`${card.blockId} — ${card.label}`}
						>
							<header className={styles.cardHeader}>
								<h3>
									{card.blockId}
									<span className={styles.cardType}>{card.label}</span>
								</h3>
								<div className={styles.cardActions}>
									{focusedIds.length > 1 && !expandedId && (
										<button
											type="button"
											data-expand-card={card.id}
											aria-label={`Expand ${card.blockId} — ${card.label}`}
											onClick={() => expandCard(card.id)}
										>
											<Maximize2 size={18} />
										</button>
									)}
									<button
										type="button"
										aria-label={`Remove ${card.blockId} — ${card.label} from focus`}
										onClick={() => {
											removeCard(card.id);
											addButton.current?.focus();
										}}
									>
										<X size={18} />
									</button>
								</div>
							</header>
							<div className={styles.reading}>{card.readingContent}</div>
						</article>
					))}
				</div>
			</div>
			<footer className={styles.hint}>
				Add cards to compare. Press Esc to{" "}
				{pickerOpen
					? "close the picker"
					: expandedId
						? "return to the group"
						: "return to the diagram"}
				.
			</footer>
		</dialog>
	);
}
