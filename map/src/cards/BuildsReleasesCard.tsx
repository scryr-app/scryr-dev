import { Package } from "lucide-react";
import { type ReactNode, useContext } from "react";
import type { GetBlocksQuery } from "@/graphql/generated";
import { CardCloseButton, CardContentInteractiveContext, CardTitle } from "./CardTitle";
import styles from "./BuildsReleasesCard.module.css";

type DeliveryBlock = GetBlocksQuery["blocks"][number];
type Snapshot = NonNullable<DeliveryBlock["deliverySnapshot"]>;
type Assessment = DeliveryBlock["deliveryAssessments"][number];

interface BuildsReleasesCardProps {
	delivery: NonNullable<DeliveryBlock["delivery"]>;
	snapshot?: DeliveryBlock["deliverySnapshot"];
	assessments: DeliveryBlock["deliveryAssessments"];
}

function label(value: string) {
	return value.toLowerCase().replaceAll("_", " ");
}

function time(value: string) {
	const date = new Date(value);
	return Number.isNaN(date.getTime())
		? value
		: date.toISOString().replace("T", " ").replace(".000Z", " UTC");
}

function newest<T>(values: readonly T[], timestamp: (value: T) => string): T[] {
	return [...values].sort((a, b) => timestamp(b).localeCompare(timestamp(a)));
}

function EvidenceLink({ url, children }: { url?: string | null; children: ReactNode }) {
	let href: string | undefined;
	try {
		const parsed = new URL(url ?? "");
		if (parsed.protocol === "https:" || parsed.protocol === "http:") href = parsed.href;
	} catch {
		// Invalid evidence links remain readable text.
	}
	return href ? (
		<a href={href} target="_blank" rel="noopener noreferrer">
			{children}
		</a>
	) : (
		<span>{children}</span>
	);
}

function Artifact({ artifact }: { artifact: Snapshot["artifacts"][number] }) {
	return (
		<li>
			<EvidenceLink url={artifact.url}>{artifact.name}</EvidenceLink>
			<div className={styles.state}>
				{label(artifact.kind)} · {artifact.expired ? "Expired" : "Listed at collection"}
			</div>
			{artifact.digest ? (
				<code>{artifact.digest}</code>
			) : (
				<span className={styles.muted}>Digest unavailable</span>
			)}
			<div className={styles.state}>Provenance: {label(artifact.provenance)}</div>
		</li>
	);
}

function Environment({
	name,
	snapshot,
	assessment,
}: {
	name: string;
	snapshot: Snapshot;
	assessment?: Assessment;
}) {
	const deployments = newest(
		snapshot.deployments.filter((entry) => entry.environment === name),
		(entry) => entry.createdAt,
	);
	const current = deployments[0];
	const link = snapshot.links.find(
		(entry) => entry.environment === name && entry.deploymentId === current?.id,
	);
	const runtime = newest(
		snapshot.runtime.filter((entry) => entry.environment === name),
		(entry) => entry.observedAt,
	)[0];
	const runtimeAvailable = assessment?.runtimeState === "AVAILABLE";
	const runtimeLabel = !runtimeAvailable
		? `Not verified${assessment ? ` · ${label(assessment.runtimeState)}` : ""}`
		: assessment.runningMatchesDeployment === true
			? "Observed artifacts match successful deployment"
			: assessment.runningMatchesDeployment === false
				? "Observed artifacts differ from successful deployment"
				: "Observed · deployment match unknown";
	const rollbackLabel =
		assessment?.eligibility === "ELIGIBLE"
			? "Eligible under policy"
			: assessment?.eligibility === "BLOCKED"
				? "Blocked"
				: "Unknown";
	return (
		<section className={styles.panel} aria-label={`${name} delivery`}>
			<h3>{name}</h3>
			<dl className={styles.facts}>
				<dt>Latest deployment</dt>
				<dd>
					{current ? (
						<EvidenceLink url={current.url}>
							#{current.id} · {label(current.status)} reported
						</EvidenceLink>
					) : (
						"No recorded deployment"
					)}
				</dd>
				<dt>Requested commit</dt>
				<dd>{current ? <code>{current.commitSha}</code> : "Unknown"}</dd>
				<dt>Artifact</dt>
				<dd>{link ? <code>{link.artifactDigest}</code> : "No deployment artifact link"}</dd>
				<dt>Running code</dt>
				<dd>{runtimeLabel}</dd>
				<dt>Rollback</dt>
				<dd>
					{rollbackLabel}
					{assessment?.candidateDeploymentId &&
						` → deployment #${assessment.candidateDeploymentId}`}
				</dd>
			</dl>
			{link && (
				<details>
					<summary>Delivery chain</summary>
					<p>
						Source <code>{link.sourceRevision}</code> → build #{link.buildRunId}, attempt{" "}
						{link.buildAttempt} → artifact <code>{link.artifactDigest}</code> → deployment #
						{link.deploymentId}
					</p>
					<p>
						Configuration <code>{link.configurationRevision}</code>
					</p>
					{link.releaseId && <p>Linked release #{link.releaseId}</p>}
				</details>
			)}
			{runtime && (
				<details>
					<summary>Runtime evidence{runtimeAvailable ? "" : " (not current)"}</summary>
					<p className={styles.muted}>
						Deployment #{runtime.deploymentId} · observed {time(runtime.observedAt)}
					</p>
					<ul className={styles.history}>
						{runtime.workloads.map((workload) => (
							<li key={`${workload.name}:${workload.artifactDigest}`}>
								{workload.name}: {workload.readyReplicas}/{workload.replicas} ready ·{" "}
								<code>{workload.artifactDigest}</code>
							</li>
						))}
					</ul>
					<p>
						Schema <code>{runtime.schemaRevision}</code> · configuration{" "}
						<code>{runtime.configurationRevision}</code>
					</p>
				</details>
			)}
			{assessment && (
				<details>
					<summary>Rollback checks</summary>
					{assessment.candidateDigest && (
						<p>
							Candidate <code>{assessment.candidateDigest}</code>
						</p>
					)}
					<ul className={styles.history}>
						{assessment.checks.map((check, index) => (
							<li key={`${check.name}:${index}`}>
								{check.name}: {label(check.outcome)}
								{check.message && ` — ${check.message}`}
							</li>
						))}
					</ul>
					<p className={styles.muted}>
						Eligibility reflects the configured evidence checks. No rollback is executed by this
						card.
					</p>
				</details>
			)}
			{deployments.length > 0 && (
				<details>
					<summary>Deployment history ({deployments.length})</summary>
					<ol className={styles.history}>
						{deployments.map((deployment) => (
							<li key={deployment.id}>
								<EvidenceLink url={deployment.url}>
									#{deployment.id} · {label(deployment.status)} reported
								</EvidenceLink>
								<div>
									<code>{deployment.commitSha}</code>
								</div>
								<div className={styles.state}>
									Created {time(deployment.createdAt)} · status {time(deployment.updatedAt)}
								</div>
								<ul>
									{deployment.statuses.map((event, index) => (
										<li key={`${event.createdAt}:${index}`}>
											<EvidenceLink url={event.url}>
												{label(event.status)} · {time(event.createdAt)}
											</EvidenceLink>
										</li>
									))}
								</ul>
							</li>
						))}
					</ol>
				</details>
			)}
		</section>
	);
}

/** Display GitHub observations without treating a recorded deployment as runtime proof. */
export function BuildsReleasesCard({ delivery, snapshot, assessments }: BuildsReleasesCardProps) {
	const interactive = useContext(CardContentInteractiveContext);
	const build = snapshot ? newest(snapshot.builds, (entry) => entry.createdAt)[0] : undefined;
	const releases = snapshot ? newest(snapshot.releases, (entry) => entry.publishedAt ?? "") : [];
	const repository = `${delivery.repository.owner}/${delivery.repository.name}`;
	return (
		<article className={styles.card} aria-label="Builds and releases">
			<header className={styles.header}>
				<CardTitle>
					<Package aria-hidden="true" />
					Builds & releases
				</CardTitle>
				<CardCloseButton />
			</header>
			<div inert={!interactive}>
				<p className={styles.muted}>
					<EvidenceLink url={`https://${delivery.repository.host}/${repository}`}>
						{repository}
					</EvidenceLink>
					{snapshot && ` · Collected ${time(snapshot.observedAt)}`}
				</p>
				{!snapshot ? (
					<p role="status">
						No delivery observations collected yet. Collect delivery observations to load GitHub
						builds, releases, and deployments.
					</p>
				) : (
					<div className={styles.sections}>
						<section className={styles.panel} aria-label="Build and release evidence">
							<h3>Build & publication</h3>
							<dl className={styles.facts}>
								<dt>Latest build</dt>
								<dd>
									{build ? (
										<EvidenceLink url={build.url}>
											#{build.id} · {label(build.conclusion ?? build.status)} · attempt{" "}
											{build.attempt}
										</EvidenceLink>
									) : (
										"No build collected"
									)}
								</dd>
								<dt>Workflow commit</dt>
								<dd>{build ? <code>{build.commitSha}</code> : "Unknown"}</dd>
								<dt>Published release</dt>
								<dd>
									{releases[0] ? (
										<EvidenceLink url={releases[0].url}>
											{releases[0].tag}
											{releases[0].draft
												? " (draft)"
												: releases[0].prerelease
													? " (prerelease)"
													: ""}
										</EvidenceLink>
									) : (
										"No release collected"
									)}
								</dd>
								<dt>Reproducibility</dt>
								<dd>Not tested</dd>
							</dl>
							<p className={styles.muted}>
								Publication and workflow success do not establish which artifact is running.
							</p>
							{snapshot.builds.length > 0 && (
								<details>
									<summary>Build history ({snapshot.builds.length})</summary>
									<ol className={styles.history}>
										{newest(snapshot.builds, (entry) => entry.createdAt).map((entry) => (
											<li key={`${entry.id}:${entry.attempt}`}>
												<EvidenceLink url={entry.url}>
													{entry.name} · #{entry.id} · attempt {entry.attempt} ·{" "}
													{label(entry.conclusion ?? entry.status)}
												</EvidenceLink>
												<div className={styles.state}>{time(entry.createdAt)}</div>
												<ul>
													{entry.jobs.map((job) => (
														<li key={job.id}>
															<EvidenceLink url={job.url}>
																{job.name}: {label(job.conclusion ?? job.status)}
															</EvidenceLink>
														</li>
													))}
												</ul>
											</li>
										))}
									</ol>
								</details>
							)}
							{snapshot.artifacts.length > 0 && (
								<details>
									<summary>Artifacts ({snapshot.artifacts.length})</summary>
									<p className={styles.muted}>
										An Actions archive digest identifies the archive, not each contained binary.
									</p>
									<ul className={styles.history}>
										{snapshot.artifacts.map((artifact) => (
											<Artifact key={`${artifact.kind}:${artifact.id}`} artifact={artifact} />
										))}
									</ul>
								</details>
							)}
							{releases.length > 0 && (
								<details>
									<summary>Release history ({releases.length})</summary>
									<ul className={styles.history}>
										{releases.map((release) => (
											<li key={release.id}>
												<EvidenceLink url={release.url}>{release.name || release.tag}</EvidenceLink>{" "}
												·{" "}
												{release.draft ? "Draft" : release.prerelease ? "Prerelease" : "Published"}
												{release.publishedAt && (
													<div className={styles.state}>{time(release.publishedAt)}</div>
												)}
											</li>
										))}
									</ul>
								</details>
							)}
						</section>
						{delivery.environments.map((environment) => (
							<Environment
								key={environment.name}
								name={environment.name}
								snapshot={snapshot}
								assessment={assessments.find((entry) => entry.environment === environment.name)}
							/>
						))}
						<section className={styles.panel} aria-label="Collection status">
							<h3>Evidence sources</h3>
							{snapshot.sources.length === 0 ? (
								<p>No source status reported.</p>
							) : (
								<ul className={styles.history}>
									{snapshot.sources.map((source, index) => (
										<li key={`${source.category}:${source.environment}:${index}`}>
											{label(source.category)}
											{source.environment && ` (${source.environment})`}: {label(source.state)}
											{source.message && <div className={styles.state}>{source.message}</div>}
										</li>
									))}
								</ul>
							)}
						</section>
					</div>
				)}
			</div>
		</article>
	);
}
