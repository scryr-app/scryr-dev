import type { ReactNode } from "react";
import type { Block as GraphqlBlock } from "@/graphql/generated";
import styles from "./CardReadingContent.module.css";
import { getClassificationMetadata } from "./ClassificationIcons";
import { LogosDictionary } from "./LogosDictionary";

function WebLink({ url, children }: { url: string; children: ReactNode }) {
	let isWebUrl = false;
	try {
		isWebUrl = ["http:", "https:"].includes(new URL(url).protocol);
	} catch {
		// Preserve the supplied text without making invalid URLs actionable.
	}
	return isWebUrl ? (
		<a href={url} target="_blank" rel="noopener noreferrer">
			{children}
		</a>
	) : (
		<span>{children}</span>
	);
}

function EnumValue({ value, prefix = "" }: { value: string; prefix?: string }) {
	const normalized = (value.split(".").pop() ?? value)
		.trim()
		.toLowerCase()
		.replace(/[\s-]+/g, "_");
	const metadata = LogosDictionary[`${prefix}${normalized}`];
	const label =
		metadata?.label ??
		normalized
			.split("_")
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(" ");
	return metadata?.docUrl ? (
		<WebLink url={metadata.docUrl}>{label}</WebLink>
	) : (
		label
	);
}

type Field = { label: string; value: ReactNode };

function Fields({ title, fields }: { title: string; fields: Field[] }) {
	const populated = fields.filter(({ value }) => value != null && value !== "");
	if (populated.length === 0) return null;
	return (
		<section className={styles.section} aria-label={title}>
			<h3>{title}</h3>
			<dl className={styles.fields}>
				{populated.map(({ label, value }) => (
					<div key={label}>
						<dt>{label}</dt>
						<dd>{value}</dd>
					</div>
				))}
			</dl>
		</section>
	);
}

function enumField(label: string, value?: string | null, prefix = ""): Field {
	return {
		label,
		value: value ? <EnumValue value={value} prefix={prefix} /> : null,
	};
}

export function InfoCardReadingContent({ block }: { block: GraphqlBlock }) {
	return (
		<div className={styles.content}>
			{block.description && (
				<p className={styles.description}>{block.description}</p>
			)}
			<Fields
				title="Overview"
				fields={[
					{
						label: "Type",
						value: block.consumerType
							? getClassificationMetadata(block.consumerType).label
							: null,
					},
					{ label: "Version", value: block.version },
					{ label: "Owner team", value: block.ownerTeam },
					enumField("Authentication", block.authType, "auth_"),
				]}
			/>
			<Fields
				title="Stack"
				fields={[
					enumField("Language", block.language),
					{
						label: "Frameworks",
						value: block.frameworks.length ? (
							<ul className={styles.list}>
								{[...new Set(block.frameworks)].map((framework) => (
									<li key={framework}>
										<EnumValue value={framework} />
									</li>
								))}
							</ul>
						) : null,
					},
					enumField("Deployment", block.deployment),
					enumField("Cloud provider", block.deploymentProvider),
					enumField("Infrastructure as code", block.iacTool, "iac_"),
				]}
			/>
			<Fields
				title="Operations"
				fields={[
					enumField("Monitoring", block.monitoring),
					enumField("Tracing", block.tracing),
					enumField("Log aggregation", block.logAggregation),
					enumField("CI/CD", block.cicdTool),
					{ label: "Minimum replicas", value: block.minReplicas },
					{ label: "Maximum replicas", value: block.maxReplicas },
				]}
			/>
			{(block.docs.length > 0 ||
				block.links.length > 0 ||
				block.sourceCodeUrl) && (
				<section
					className={styles.section}
					aria-label="Links and documentation"
				>
					<h3>Links and documentation</h3>
					<ul className={styles.links}>
						{[...new Set(block.docs)].map((url) => (
							<li key={`doc-${url}`}>
								<WebLink url={url}>{url}</WebLink>
							</li>
						))}
						{block.links.map((link) => (
							<li key={`${link.siteName}-${link.httpUrl}`}>
								{link.httpUrl ? (
									<WebLink url={link.httpUrl}>
										{link.siteName || link.httpUrl}
									</WebLink>
								) : (
									link.siteName
								)}
							</li>
						))}
						{block.sourceCodeUrl && (
							<li>
								<WebLink url={block.sourceCodeUrl}>Source code</WebLink>
							</li>
						)}
					</ul>
				</section>
			)}
		</div>
	);
}

export function DiagramCardReadingContent({
	diagrams,
}: {
	diagrams: string[];
}) {
	return (
		<div className={styles.content}>
			{diagrams.length ? (
				<ul className={styles.diagrams} aria-label="Other diagrams">
					{[...new Set(diagrams)].map((name) => (
						<li key={name}>{name}</li>
					))}
				</ul>
			) : (
				<p>No diagrams found</p>
			)}
		</div>
	);
}
