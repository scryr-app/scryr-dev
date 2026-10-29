import { CardContentInteractiveContext, CardTitle } from "./CardTitle";
import { Activity, Code, ExternalLink, Info, Users } from "lucide-react";
import { type ReactNode, useContext } from "react";
import type { BlockLink } from "@/block/Block";
import { getClassificationMetadata } from "./ClassificationIcons";
import { LogosDictionary } from "./LogosDictionary";
import { MarkdownDescription } from "./MarkdownDescription";
import styles from "./CardContent.module.css";

// Let the browser render complete SVG files, including gradients, masks and filters.
const assets = import.meta.glob<string>("../icons/*.svg", {
	eager: true,
	import: "default",
	query: "?url",
});

function normalize(value: string) {
	return (value.split(".").pop() ?? value)
		.trim()
		.toLowerCase()
		.replace(/[\s-]+/g, "_");
}

function safeUrl(value: string | null | undefined): string | undefined {
	if (!value) return undefined;
	try {
		const url = new URL(value);
		return url.protocol === "https:" || url.protocol === "http:" ? url.href : undefined;
	} catch {
		return undefined;
	}
}

function enumLabel(value: string) {
	const key = normalize(value);
	return (
		LogosDictionary[key]?.label ??
		LogosDictionary[`iac_${key}`]?.label ??
		key
			.replace(/^(auth|iac)_/, "")
			.split("_")
			.filter(Boolean)
			.map((part) => part[0].toUpperCase() + part.slice(1))
			.join(" ")
	);
}

function Technology({ value }: { value: string }) {
	const key = normalize(value);
	const metadata = LogosDictionary[key] ?? LogosDictionary[`iac_${key}`];
	const source = metadata?.iconFile ? assets[`../icons/${metadata.iconFile}`] : undefined;
	const label = enumLabel(value);
	const url = safeUrl(metadata?.docUrl);
	const content = (
		<>
			{source && <img src={source} alt="" className={styles.technologyIcon} />}
			<span>{label}</span>
		</>
	);
	return url ? (
		<a className={styles.chip} href={url} target="_blank" rel="noopener noreferrer">
			{content}
		</a>
	) : (
		<span className={styles.chip}>{content}</span>
	);
}

function TechnologyList({ values }: { values: string[] }) {
	const unique = [...new Set(values.map((value) => value.trim()).filter(Boolean))];
	return (
		<ul className={styles.chips}>
			{unique.map((value) => (
				<li key={value}>
					<Technology value={value} />
				</li>
			))}
		</ul>
	);
}

function Panel({ title, icon, children }: { title: string; icon: ReactNode; children: ReactNode }) {
	return (
		<section className={styles.panel} aria-label={title}>
			<h3>
				{icon}
				{title}
			</h3>
			{children}
		</section>
	);
}

export interface InfoCardProps {
	/** Block classification used for architecture visualization */
	classification?: string;
	/** GitHub-flavored Markdown description, serialized as text by the SDK. */
	description?: string;
	/** Semver string e.g. "1.2.3" */
	version?: string;
	/** Programming languages in use */
	languages?: string[];
	/** Web frameworks in use */
	frameworks?: string[];
	/** Deployment targets and infrastructure tooling */
	deployments?: string[];
	/** Owners responsible for this service */
	owners?: string;
	/** Monitoring, tracing, and log aggregation tools */
	metricTools?: string[];
	/** Documentation URLs */
	docs?: string[];
	/** External links */
	links?: BlockLink[];
}

/** Ordinary React content; CardSurface owns its size, theme and 3D placement. */
export function InfoCard({
	classification,
	description,
	version,
	languages = [],
	frameworks = [],
	deployments = [],
	owners,
	metricTools = [],
	docs = [],
	links = [],
}: InfoCardProps) {
	const info = classification ? getClassificationMetadata(classification) : undefined;
	const ClassificationIcon = info?.icon;
	const stack = [...languages, ...frameworks, ...deployments];
	const operations = metricTools;
	const externalLinks = [
		...docs.map((url, index) => ({
			label: docs.length > 1 ? `Docs ${index + 1}` : "Docs",
			url: safeUrl(url),
		})),
		...links.map((link) => ({
			label: link.siteName?.trim() || "Link",
			url: safeUrl(link.httpUrl),
		})),
	].filter((link): link is { label: string; url: string } => Boolean(link.url));
	const hasDetails = Boolean(description?.trim() || info || owners || externalLinks.length);

	return (
		<article className={styles.card} aria-label="Service information">
			<header className={styles.header}>
				<CardTitle>
					<Info aria-hidden="true" />
					Info
				</CardTitle>
				{version && <span className={styles.version}>v{version}</span>}
			</header>
			<div
				inert={!useContext(CardContentInteractiveContext)}
				className={styles.sections}
				data-columns={hasDetails && (stack.length > 0 || operations.length > 0) ? "two" : "one"}
			>
				{hasDetails && (
					<section className={styles.panel} aria-label="Service details">
						{description?.trim() && (
							<MarkdownDescription key={description} source={description.trim()} />
						)}
						{info && (
							<div className={styles.metadata}>
								{info && ClassificationIcon && (
									<span className={styles.chip}>
										<span className={styles.caption}>Type</span>
										<ClassificationIcon width={info.iconWidth ?? 24} />
										<span>{info.label}</span>
									</span>
								)}
							</div>
						)}
						{owners && (
							<p className={styles.owner}>
								<Users aria-hidden="true" />
								<span>{owners}</span>
							</p>
						)}
						{externalLinks.length > 0 && (
							<nav className={styles.links} aria-label="Service resources">
								{externalLinks.map(({ label, url }, index) => (
									<a key={`${url}-${index}`} href={url} target="_blank" rel="noopener noreferrer">
										<ExternalLink aria-hidden="true" />
										{label}
									</a>
								))}
							</nav>
						)}
					</section>
				)}
				{(stack.length > 0 || operations.length > 0) && (
					<div className={styles.technicalPanels}>
						{stack.length > 0 && (
							<Panel title="Stack" icon={<Code aria-hidden="true" />}>
								<TechnologyList values={stack} />
							</Panel>
						)}
						{operations.length > 0 && (
							<Panel title="Metric tools" icon={<Activity aria-hidden="true" />}>
								<TechnologyList values={operations} />
							</Panel>
						)}
					</div>
				)}
			</div>
			{!hasDetails && stack.length === 0 && operations.length === 0 && (
				<p className={styles.empty}>No service details provided.</p>
			)}
		</article>
	);
}
