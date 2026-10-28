import { useEffect, useId, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import styles from "./MarkdownDescription.module.css";

/** Render the original Markdown intact; only the visual preview is clipped. */
export function MarkdownDescription({ source }: { source: string }) {
	const id = useId();
	const content = useRef<HTMLDivElement>(null);
	const [expanded, setExpanded] = useState(false);
	const [overflowing, setOverflowing] = useState(false);

	useEffect(() => {
		const element = content.current;
		if (!element) return;
		const measure = () => {
			const lineHeight = Number.parseFloat(getComputedStyle(element).lineHeight);
			setOverflowing(element.scrollHeight > lineHeight * 8 + 1);
		};
		measure();
		if (typeof ResizeObserver === "undefined") return;
		const observer = new ResizeObserver(measure);
		observer.observe(element);
		return () => observer.disconnect();
	}, [source]);

	return (
		<div className={styles.description}>
			<div
				id={id}
				className={styles.preview}
				data-expanded={expanded}
				onFocusCapture={() => setExpanded(true)}
			>
				<div ref={content} className={styles.markdown}>
					<ReactMarkdown
						remarkPlugins={[remarkGfm]}
						remarkRehypeOptions={{ clobberPrefix: `${id}-` }}
						skipHtml
						components={{
							a: ({ node: _node, href, children, ...props }) => (
								<a
									{...props}
									href={href}
									target={href?.startsWith("#") ? undefined : "_blank"}
									rel="noopener noreferrer"
								>
									{children}
								</a>
							),
						}}
					>
						{source}
					</ReactMarkdown>
				</div>
			</div>
			{overflowing && (
				<div className={styles.controls}>
					{!expanded && <span aria-hidden="true">…</span>}
					<button
						type="button"
						aria-controls={id}
						aria-expanded={expanded}
						onClick={() => setExpanded(!expanded)}
					>
						{expanded ? "Show less" : "Show more"}
					</button>
				</div>
			)}
		</div>
	);
}
