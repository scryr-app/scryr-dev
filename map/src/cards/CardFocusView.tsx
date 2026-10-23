import { ArrowLeft, Maximize2, Plus, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useCardFocus } from "./CardFocusContext";
import styles from "./CardFocusView.module.css";

export function CardFocusView() {
	const { focusedIds } = useCardFocus();
	return focusedIds.length > 0 ? <OpenCardFocusView /> : null;
}

function OpenCardFocusView() {
	const { cards, focusedIds, expandedId, addCard, removeCard, expandCard, closeFocus } =
		useCardFocus();
	const controls = useRef<HTMLDivElement>(null);
	const backButton = useRef<HTMLButtonElement>(null);
	const addButton = useRef<HTMLButtonElement>(null);
	const searchInput = useRef<HTMLInputElement>(null);
	const backToGroupButton = useRef<HTMLButtonElement>(null);
	const previousExpandedId = useRef<string | null>(null);
	const [pickerOpen, setPickerOpen] = useState(false);
	const [query, setQuery] = useState("");
	const selected = focusedIds.flatMap((id) => {
		const card = cards.find((candidate) => candidate.id === id);
		return card ? [card] : [];
	});
	const available = cards.filter(
		(card) =>
			!focusedIds.includes(card.id) &&
			`${card.label} ${card.blockId}`.toLowerCase().includes(query.toLowerCase()),
	);

	useEffect(() => {
		const previousFocus = document.activeElement;
		backButton.current?.focus();
		return () => {
			if (previousFocus instanceof HTMLElement && previousFocus.isConnected) previousFocus.focus();
		};
	}, []);

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
				controls.current?.querySelectorAll<HTMLButtonElement>("[data-expand-card]") ?? [],
			).find((button) => button.dataset.expandCard === previous);
			(expandButton ?? addButton.current)?.focus();
		}
	}, [expandedId, pickerOpen]);

	useEffect(() => {
		function handleEscape(event: KeyboardEvent) {
			if (event.key !== "Escape" || event.defaultPrevented) return;
			const target = event.target;
			if (
				target instanceof HTMLElement &&
				!controls.current?.contains(target) &&
				(target.isContentEditable || target.matches("input, textarea, select"))
			)
				return;
			event.preventDefault();
			event.stopPropagation();
			if (pickerOpen) {
				setPickerOpen(false);
				setQuery("");
				addButton.current?.focus();
			} else if (expandedId) expandCard(null);
			else closeFocus();
		}
		window.addEventListener("keydown", handleEscape);
		return () => window.removeEventListener("keydown", handleEscape);
	}, [pickerOpen, expandedId, expandCard, closeFocus]);

	function dismissPicker() {
		setPickerOpen(false);
		setQuery("");
		addButton.current?.focus();
	}

	return (
		<div ref={controls} className={styles.controls}>
			<header className={styles.toolbar} aria-label="Card focus controls">
				<button ref={backButton} type="button" onClick={closeFocus}>
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
			<ul className={styles.chips} aria-label="Focused cards">
				{selected.map((card) => (
					<li key={card.id} className={styles.chip} data-expanded={expandedId === card.id}>
						<span className={styles.swatch} style={{ backgroundColor: card.color }} />
						<span className={styles.cardLabel} title={`${card.blockId} — ${card.label}`}>
							{card.blockId} — {card.label}
						</span>
						{focusedIds.length > 1 && !expandedId && (
							<button
								type="button"
								data-expand-card={card.id}
								aria-label={`Expand ${card.blockId} — ${card.label}`}
								onClick={() => expandCard(card.id)}
							>
								<Maximize2 size={16} />
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
							<X size={16} />
						</button>
					</li>
				))}
			</ul>
		</div>
	);
}
