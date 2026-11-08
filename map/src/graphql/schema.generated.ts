export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  /**
   * Implement the DateTime<Utc> scalar
   *
   * The input/output is a string in RFC3339 format.
   */
  DateTime: { input: string; output: string; }
  /** A scalar that can represent any JSON value. */
  JSON: { input: unknown; output: unknown; }
  /**
   * A UUID is a unique 128-bit number, stored as 16 octets. UUIDs are parsed as
   * Strings within GraphQL. UUIDs are used to assign unique identifiers to
   * entities without requiring a central allocating authority.
   *
   * # References
   *
   * * [Wikipedia: Universally Unique Identifier](http://en.wikipedia.org/wiki/Universally_unique_identifier)
   * * [RFC4122: A Universally Unique Identifier (UUID) URN Namespace](http://tools.ietf.org/html/rfc4122)
   */
  UUID: { input: unknown; output: unknown; }
};

/** Typed `ActionsBuildSource` record. */
export type ActionsBuildSource = {
  __typename?: 'ActionsBuildSource';
  /** Selected Actions artifact names; empty selects all. */
  artifactNames: Array<Scalars['String']['output']>;
  /** Workflow path or numeric identifier. */
  workflow: Scalars['String']['output'];
};

/** Typed `ActionsReportSource` record. */
export type ActionsReportSource = {
  __typename?: 'ActionsReportSource';
  /** Artifact containing the report JSON. */
  artifactName: Scalars['String']['output'];
  /** Maximum evidence age in seconds. */
  maxAgeSeconds: Scalars['Int']['output'];
  /** Expected versioned report schema. */
  schema: Scalars['String']['output'];
  /** Trusted reporting workflow path. */
  workflow: Scalars['String']['output'];
};

/** Logical kind for a generated artifact persisted to storage. */
export enum ArtifactKind {
  /** Generated schema or type-definition output. */
  Schema = 'schema',
  /** Generated instance/value output. */
  Value = 'value'
}

export type Block = {
  __typename?: 'Block';
  /** CI/CD pipeline platform (`github_actions`, `jenkins`, `circleci`, etc.). */
  cicdTool?: Maybe<Scalars['String']['output']>;
  /** Named connections to other components in the graph. */
  connections: Array<Scalars['String']['output']>;
  /** Service classification (e.g. `public_api`, `worker`, `database`). */
  consumerType?: Maybe<Scalars['String']['output']>;
  /**
   * Declarative GitHub delivery sources, without credentials or network access.
   * # Errors
   * Rejects an invalid persisted configuration.
   */
  delivery?: Maybe<GitHubDelivery>;
  /**
   * Freshness and rollback eligibility evaluated at query time.
   * # Errors
   * Rejects malformed configuration or evidence.
   */
  deliveryAssessments: Array<DeliveryAssessment>;
  /**
   * Latest organization-scoped delivery observation attached by the server.
   * # Errors
   * Rejects malformed persisted evidence.
   */
  deliverySnapshot?: Maybe<DeliverySnapshot>;
  /** Hosting targets and Infrastructure as Code tools used for deployment. */
  deployments: Array<Scalars['String']['output']>;
  /** Short description of the component. */
  description?: Maybe<Scalars['String']['output']>;
  /** Documentation URLs. */
  docs: Array<Scalars['String']['output']>;
  /** Web frameworks in use. */
  frameworks: Array<Scalars['String']['output']>;
  /** Recent GitHub workflow runs and their observed status history. */
  githubActions?: Maybe<Scalars['JSON']['output']>;
  /** Emoji or small icon string for the component. */
  icon?: Maybe<Scalars['String']['output']>;
  /** Programming languages used by the component. */
  languages: Array<Scalars['String']['output']>;
  /** 1-based line number of the manifest assignment in the source file. */
  lineNumber?: Maybe<Scalars['Int']['output']>;
  /** Helpful external links with display name and URL. */
  links: Array<Link>;
  /** Stable Manifest identity used to attach operational history. */
  manifestId?: Maybe<Scalars['String']['output']>;
  /** Monitoring, tracing, and log aggregation tools. */
  metricTools: Array<Scalars['String']['output']>;
  /** Display name for the component (used as the block label). */
  name?: Maybe<Scalars['String']['output']>;
  /** Owners responsible for this service. */
  owners?: Maybe<Scalars['String']['output']>;
  /** Returns the raw JSON as a string for debugging. */
  rawJsonString: Scalars['String']['output'];
  /** Repository or source URL (also used by `GithubCard`). */
  sourceCodeUrl?: Maybe<Scalars['String']['output']>;
  /** Arbitrary tags for categorization and filtering. */
  tags: Array<Scalars['String']['output']>;
  /** Semver-like version string (e.g. "1.2.3"). */
  version?: Maybe<Scalars['String']['output']>;
};

/** Typed `DeliveryArtifact` record. */
export type DeliveryArtifact = {
  __typename?: 'DeliveryArtifact';
  /** Digest of this artifact, distinct from contained files. */
  digest?: Maybe<Scalars['String']['output']>;
  /** Provider reports artifact expired. */
  expired: Scalars['Boolean']['output'];
  /** Provider artifact/asset identity. */
  id: Scalars['String']['output'];
  /** Artifact storage kind. */
  kind: DeliveryArtifactKind;
  /** Artifact display name. */
  name: Scalars['String']['output'];
  /** Verified cryptographic provenance, never inferred from build success. */
  provenance: DeliveryVerification;
  /** Owning GitHub release when provided. */
  releaseId?: Maybe<Scalars['String']['output']>;
  /** Producing workflow run when provided. */
  runId?: Maybe<Scalars['String']['output']>;
  /** Artifact reference URL. */
  url?: Maybe<Scalars['String']['output']>;
};

/** Values for `DeliveryArtifactKind`. */
export enum DeliveryArtifactKind {
  /** Actions artifact. */
  ActionsArtifact = 'ACTIONS_ARTIFACT',
  /** Oci image. */
  OciImage = 'OCI_IMAGE',
  /** Release asset. */
  ReleaseAsset = 'RELEASE_ASSET'
}

/** Typed `DeliveryAssessment` record. */
export type DeliveryAssessment = {
  __typename?: 'DeliveryAssessment';
  /** Previous successful deployment candidate. */
  candidateDeploymentId?: Maybe<Scalars['String']['output']>;
  /** Explicit candidate artifact digest. */
  candidateDigest?: Maybe<Scalars['String']['output']>;
  /** Assessment checks with explicit unknown outcomes. */
  checks: Array<DeliveryCompatibilityCheck>;
  /** Whether configured checks establish rollback eligibility. */
  eligibility: DeliveryEligibility;
  /** Configured environment name. */
  environment: Scalars['String']['output'];
  /** Whether every observed workload matches the deployment link. */
  runningMatchesDeployment?: Maybe<Scalars['Boolean']['output']>;
  /** Freshness and availability of runtime evidence. */
  runtimeState: DeliveryEvidenceState;
};

/** Typed `DeliveryBuild` record. */
export type DeliveryBuild = {
  __typename?: 'DeliveryBuild';
  /** Workflow attempt number. */
  attempt: Scalars['Int']['output'];
  /** Workflow head commit, not inferred artifact provenance. */
  commitSha: Scalars['String']['output'];
  /** Completed outcome. */
  conclusion?: Maybe<DeliveryConclusion>;
  /** Provider creation time. */
  createdAt: Scalars['DateTime']['output'];
  /** GitHub run ID. */
  id: Scalars['String']['output'];
  /** Jobs for this run attempt. */
  jobs: Array<DeliveryJob>;
  /** Run display name. */
  name: Scalars['String']['output'];
  /** Execution state. */
  status: DeliveryRunStatus;
  /** Provider update time. */
  updatedAt: Scalars['DateTime']['output'];
  /** Workflow run URL. */
  url: Scalars['String']['output'];
  /** Workflow path. */
  workflow: Scalars['String']['output'];
};

/** Values for `DeliveryCheckOutcome`. */
export enum DeliveryCheckOutcome {
  /** Failed. */
  Failed = 'FAILED',
  /** Passed. */
  Passed = 'PASSED',
  /** Unknown. */
  Unknown = 'UNKNOWN'
}

/** Typed `DeliveryCompatibilityCheck` record. */
export type DeliveryCompatibilityCheck = {
  __typename?: 'DeliveryCompatibilityCheck';
  /** Human-readable result detail. */
  message?: Maybe<Scalars['String']['output']>;
  /** Performed compatibility check. */
  name: Scalars['String']['output'];
  /** Check result. */
  outcome: DeliveryCheckOutcome;
};

/** Typed `DeliveryCompatibilityEvidence` record. */
export type DeliveryCompatibilityEvidence = {
  __typename?: 'DeliveryCompatibilityEvidence';
  /** Candidate configuration tested; omission only applies when current and candidate configurations match. */
  candidateConfigurationRevision?: Maybe<Scalars['String']['output']>;
  /** Exact candidate artifact tested. */
  candidateDigest: Scalars['String']['output'];
  /** Individual check outcomes. */
  checks: Array<DeliveryCompatibilityCheck>;
  /** Configuration identity against which the candidate was tested. */
  currentConfigurationRevision: Scalars['String']['output'];
  /** Schema identity against which the candidate was tested. */
  currentSchemaRevision: Scalars['String']['output'];
  /** Configured environment name. */
  environment: Scalars['String']['output'];
  /** Stable Scryr manifest identity. */
  manifestId: Scalars['String']['output'];
  /** Actual check completion time. */
  observedAt: Scalars['DateTime']['output'];
  /** Must be scryr.compatibility.v1. */
  schema: Scalars['String']['output'];
  /** Version of the checks that were executed. */
  suiteRevision: Scalars['String']['output'];
};

/** Values for `DeliveryConclusion`. */
export enum DeliveryConclusion {
  /** Action required. */
  ActionRequired = 'ACTION_REQUIRED',
  /** Cancelled. */
  Cancelled = 'CANCELLED',
  /** Failure. */
  Failure = 'FAILURE',
  /** Neutral. */
  Neutral = 'NEUTRAL',
  /** Skipped. */
  Skipped = 'SKIPPED',
  /** Stale. */
  Stale = 'STALE',
  /** Startup failure. */
  StartupFailure = 'STARTUP_FAILURE',
  /** Success. */
  Success = 'SUCCESS',
  /** Timed out. */
  TimedOut = 'TIMED_OUT',
  /** Unknown. */
  Unknown = 'UNKNOWN'
}

/** Typed `DeliveryDeployment` record. */
export type DeliveryDeployment = {
  __typename?: 'DeliveryDeployment';
  /** Requested deployment revision. */
  commitSha: Scalars['String']['output'];
  /** Deployment creation time. */
  createdAt: Scalars['DateTime']['output'];
  /** Provider environment name. */
  environment: Scalars['String']['output'];
  /** GitHub deployment ID. */
  id: Scalars['String']['output'];
  /** Latest reported status. */
  status: DeliveryDeploymentStatus;
  /** Append-only provider status history, newest first. */
  statuses: Array<DeliveryDeploymentEvent>;
  /** Latest status time. */
  updatedAt: Scalars['DateTime']['output'];
  /** Environment or deployment log URL. */
  url?: Maybe<Scalars['String']['output']>;
};

/** One GitHub deployment status event. */
export type DeliveryDeploymentEvent = {
  __typename?: 'DeliveryDeploymentEvent';
  /** Time reported by GitHub. */
  createdAt: Scalars['DateTime']['output'];
  /** Reported deployment state. */
  status: DeliveryDeploymentStatus;
  /** Environment or log URL. */
  url?: Maybe<Scalars['String']['output']>;
};

/** Values for `DeliveryDeploymentStatus`. */
export enum DeliveryDeploymentStatus {
  /** Error. */
  Error = 'ERROR',
  /** Failure. */
  Failure = 'FAILURE',
  /** Inactive. */
  Inactive = 'INACTIVE',
  /** In progress. */
  InProgress = 'IN_PROGRESS',
  /** Pending. */
  Pending = 'PENDING',
  /** Queued. */
  Queued = 'QUEUED',
  /** Success. */
  Success = 'SUCCESS',
  /** Unknown. */
  Unknown = 'UNKNOWN'
}

/** Values for `DeliveryEligibility`. */
export enum DeliveryEligibility {
  /** Blocked. */
  Blocked = 'BLOCKED',
  /** Eligible. */
  Eligible = 'ELIGIBLE',
  /** Unknown. */
  Unknown = 'UNKNOWN'
}

/** Typed `DeliveryEnvironment` record. */
export type DeliveryEnvironment = {
  __typename?: 'DeliveryEnvironment';
  /** Optional candidate compatibility evidence. */
  compatibility?: Maybe<ActionsReportSource>;
  /** Native deployment source. */
  deployments: GitHubDeploymentSource;
  /** Stable environment identity. */
  name: Scalars['String']['output'];
  /** Rollback evidence requirements. */
  rollback: DeliveryRollbackPolicy;
  /** Optional running-artifact evidence. */
  runtime?: Maybe<ActionsReportSource>;
};

/** Values for `DeliveryEvidenceState`. */
export enum DeliveryEvidenceState {
  /** Access denied. */
  AccessDenied = 'ACCESS_DENIED',
  /** Available. */
  Available = 'AVAILABLE',
  /** Error. */
  Error = 'ERROR',
  /** Missing. */
  Missing = 'MISSING',
  /** Stale. */
  Stale = 'STALE',
  /** Unsupported. */
  Unsupported = 'UNSUPPORTED'
}

/** Typed `DeliveryJob` record. */
export type DeliveryJob = {
  __typename?: 'DeliveryJob';
  /** Completed outcome. */
  conclusion?: Maybe<DeliveryConclusion>;
  /** GitHub job ID. */
  id: Scalars['String']['output'];
  /** Job display name. */
  name: Scalars['String']['output'];
  /** Execution state. */
  status: DeliveryRunStatus;
  /** Job log URL. */
  url: Scalars['String']['output'];
};

/** Typed `DeliveryLink` record. */
export type DeliveryLink = {
  __typename?: 'DeliveryLink';
  /** Deployed artifact content digest. */
  artifactDigest: Scalars['String']['output'];
  /** Producing run attempt. */
  buildAttempt: Scalars['Int']['output'];
  /** Explicit producing build run ID. */
  buildRunId: Scalars['String']['output'];
  /** Deployed configuration identity. */
  configurationRevision: Scalars['String']['output'];
  /** Explicit GitHub deployment ID. */
  deploymentId: Scalars['String']['output'];
  /** Configured environment name. */
  environment: Scalars['String']['output'];
  /** Stable Scryr manifest identity. */
  manifestId: Scalars['String']['output'];
  /** Explicit GitHub release ID. */
  releaseId?: Maybe<Scalars['String']['output']>;
  /** Must be scryr.delivery.v1. */
  schema: Scalars['String']['output'];
  /** Actual source revision reported by the build. */
  sourceRevision: Scalars['String']['output'];
};

/** Values for `DeliveryProvider`. */
export enum DeliveryProvider {
  /** Github. */
  Github = 'GITHUB'
}

/** Typed `DeliveryRelease` record. */
export type DeliveryRelease = {
  __typename?: 'DeliveryRelease';
  /** Explicit release asset identities. */
  assetIds: Array<Scalars['String']['output']>;
  /** Whether release is a draft. */
  draft: Scalars['Boolean']['output'];
  /** GitHub release ID. */
  id: Scalars['String']['output'];
  /** Display title. */
  name?: Maybe<Scalars['String']['output']>;
  /** Whether release is a prerelease. */
  prerelease: Scalars['Boolean']['output'];
  /** Publication time. */
  publishedAt?: Maybe<Scalars['DateTime']['output']>;
  /** Release tag. */
  tag: Scalars['String']['output'];
  /** Release URL. */
  url: Scalars['String']['output'];
};

/** Typed `DeliveryRollbackPolicy` record. */
export type DeliveryRollbackPolicy = {
  __typename?: 'DeliveryRollbackPolicy';
  /** Require candidate artifact availability. */
  requireArtifactAvailable: Scalars['Boolean']['output'];
  /** Require scoped compatibility evidence. */
  requireCompatibilityEvidence: Scalars['Boolean']['output'];
  /** Require candidate provenance verification. */
  requireVerifiedProvenance: Scalars['Boolean']['output'];
};

/** Values for `DeliveryRunStatus`. */
export enum DeliveryRunStatus {
  /** Completed. */
  Completed = 'COMPLETED',
  /** In progress. */
  InProgress = 'IN_PROGRESS',
  /** Pending. */
  Pending = 'PENDING',
  /** Queued. */
  Queued = 'QUEUED',
  /** Requested. */
  Requested = 'REQUESTED',
  /** Unknown. */
  Unknown = 'UNKNOWN',
  /** Waiting. */
  Waiting = 'WAITING'
}

/** Typed `DeliveryRuntimeEvidence` record. */
export type DeliveryRuntimeEvidence = {
  __typename?: 'DeliveryRuntimeEvidence';
  /** Observed configuration identity. */
  configurationRevision: Scalars['String']['output'];
  /** Deployment the probe identifies. */
  deploymentId: Scalars['String']['output'];
  /** Configured environment name. */
  environment: Scalars['String']['output'];
  /** Stable Scryr manifest identity. */
  manifestId: Scalars['String']['output'];
  /** Time of actual probe. */
  observedAt: Scalars['DateTime']['output'];
  /** Must be scryr.runtime.v1. */
  schema: Scalars['String']['output'];
  /** Observed database schema identity. */
  schemaRevision: Scalars['String']['output'];
  /** Observed workload digest distribution. */
  workloads: Array<DeliveryRuntimeWorkload>;
};

/** Typed `DeliveryRuntimeWorkload` record. */
export type DeliveryRuntimeWorkload = {
  __typename?: 'DeliveryRuntimeWorkload';
  /** Observed content digest. */
  artifactDigest: Scalars['String']['output'];
  /** Workload/container identity. */
  name: Scalars['String']['output'];
  /** Ready replicas for this digest. */
  readyReplicas: Scalars['Int']['output'];
  /** Observed replica count. */
  replicas: Scalars['Int']['output'];
};

/** Typed `DeliverySnapshot` record. */
export type DeliverySnapshot = {
  __typename?: 'DeliverySnapshot';
  /** Observed artifact inventory. */
  artifacts: Array<DeliveryArtifact>;
  /** Observed build runs. */
  builds: Array<DeliveryBuild>;
  /** Workflow-published compatibility observations. */
  compatibility: Array<DeliveryCompatibilityEvidence>;
  /** Exact declaration used by the collector; prevents reusing evidence after source changes. */
  configuration?: Maybe<GitHubDelivery>;
  /** Observed deployment history. */
  deployments: Array<DeliveryDeployment>;
  /** GitHub host. */
  host: Scalars['String']['output'];
  /** Explicit source/build/artifact/deployment links. */
  links: Array<DeliveryLink>;
  /** Collection snapshot time. */
  observedAt: Scalars['DateTime']['output'];
  /** Observed published releases. */
  releases: Array<DeliveryRelease>;
  /** Repository owner/name. */
  repository: Scalars['String']['output'];
  /** Workflow-published runtime observations. */
  runtime: Array<DeliveryRuntimeEvidence>;
  /** Collection status per source. */
  sources: Array<DeliverySourceState>;
};

/** Values for `DeliverySourceCategory`. */
export enum DeliverySourceCategory {
  /** Artifacts. */
  Artifacts = 'ARTIFACTS',
  /** Builds. */
  Builds = 'BUILDS',
  /** Compatibility. */
  Compatibility = 'COMPATIBILITY',
  /** Deployments. */
  Deployments = 'DEPLOYMENTS',
  /** Provenance. */
  Provenance = 'PROVENANCE',
  /** Releases. */
  Releases = 'RELEASES',
  /** Runtime. */
  Runtime = 'RUNTIME'
}

/** Typed `DeliverySourceState` record. */
export type DeliverySourceState = {
  __typename?: 'DeliverySourceState';
  /** Evidence category. */
  category: DeliverySourceCategory;
  /** Environment when the source is scoped. */
  environment?: Maybe<Scalars['String']['output']>;
  /** Non-sensitive diagnostic. */
  message?: Maybe<Scalars['String']['output']>;
  /** Collection result. */
  state: DeliveryEvidenceState;
};

/** Typed collection options declared inside a delivery definition. */
export type DeliveryTask = {
  __typename?: 'DeliveryTask';
  /** Maximum recent records requested from each source, between one and 100. */
  limit: Scalars['Int']['output'];
  /** Download release assets and verify their build-workflow attestations. */
  verifyAttestations: Scalars['Boolean']['output'];
};

/** Built-in delivery actions with declarative, typed options rather than shell commands. */
export type DeliveryTasks = {
  __typename?: 'DeliveryTasks';
  /** Collect native delivery records and configured workflow reports. */
  collect: DeliveryTask;
  /** Collect records and verify release-asset attestations by default. */
  verify: DeliveryTask;
};

/** Values for `DeliveryVerification`. */
export enum DeliveryVerification {
  /** Failed. */
  Failed = 'FAILED',
  /** Unknown. */
  Unknown = 'UNKNOWN',
  /** Verified. */
  Verified = 'VERIFIED'
}

export type GeneratedManifestMutationRoot = {
  __typename?: 'GeneratedManifestMutationRoot';
  /** Record one GitHub Actions observation without rewriting a generated Manifest. */
  recordActionRun: Scalars['Boolean']['output'];
  /** Append a typed operational observation for the active organization. */
  recordReport: Scalars['Boolean']['output'];
  /** Save source and all its diagram artifacts with optimistic concurrency. */
  saveManifestDocument: Scalars['JSON']['output'];
  /** Upsert a generated manifest artifact into storage. */
  upsertGeneratedManifest: UpsertGeneratedManifestPayload;
};


export type GeneratedManifestMutationRootRecordActionRunArgs = {
  eventId?: InputMaybe<Scalars['String']['input']>;
  manifestId: Scalars['String']['input'];
  run: Scalars['JSON']['input'];
  source?: Scalars['String']['input'];
};


export type GeneratedManifestMutationRootRecordReportArgs = {
  manifestId: Scalars['String']['input'];
  report: Scalars['JSON']['input'];
};


export type GeneratedManifestMutationRootSaveManifestDocumentArgs = {
  envelope: Scalars['JSON']['input'];
  identifier: Scalars['String']['input'];
  revision: Scalars['String']['input'];
};


export type GeneratedManifestMutationRootUpsertGeneratedManifestArgs = {
  input: UpsertGeneratedManifestInput;
};

/** Typed `GitHubDelivery` record. */
export type GitHubDelivery = {
  __typename?: 'GitHubDelivery';
  /** Build source selection. */
  builds: ActionsBuildSource;
  /** Configured deployment environments. */
  environments: Array<DeliveryEnvironment>;
  /** Provider discriminator. */
  kind: DeliveryProvider;
  /** Release source selection. */
  releases: GitHubReleaseSource;
  /** Repository identity. */
  repository: GitHubRepository;
  /** Named, typed collection actions; defaults preserve legacy definitions. */
  tasks: DeliveryTasks;
};

/** Typed `GitHubDeploymentSource` record. */
export type GitHubDeploymentSource = {
  __typename?: 'GitHubDeploymentSource';
  /** Exact GitHub environment name. */
  environment: Scalars['String']['output'];
  /** Deployment task filter. */
  task: Scalars['String']['output'];
};

/** Typed `GitHubReleaseSource` record. */
export type GitHubReleaseSource = {
  __typename?: 'GitHubReleaseSource';
  /** Include unpublished drafts. */
  includeDrafts: Scalars['Boolean']['output'];
  /** Include prereleases. */
  includePrereleases: Scalars['Boolean']['output'];
};

/** Typed `GitHubRepository` record. */
export type GitHubRepository = {
  __typename?: 'GitHubRepository';
  /** GitHub host, without scheme. */
  host: Scalars['String']['output'];
  /** Repository name. */
  name: Scalars['String']['output'];
  /** Repository owner. */
  owner: Scalars['String']['output'];
};

/** Health check result exposed over GraphQL and HTTP. */
export type HealthStatus = {
  __typename?: 'HealthStatus';
  /** Details about the database health check result. */
  databaseMessage: Scalars['String']['output'];
  /** Whether the database query succeeded. */
  databaseOk: Scalars['Boolean']['output'];
  /** Overall service status. */
  status: Scalars['String']['output'];
};

/** External reference associated with a block (e.g., repo, docs, dashboard). */
export type Link = {
  __typename?: 'Link';
  /** URL of the target resource. */
  httpUrl?: Maybe<Scalars['String']['output']>;
  /** Human-readable name for the link target. */
  siteName?: Maybe<Scalars['String']['output']>;
};

export type QueryRoot = {
  __typename?: 'QueryRoot';
  /** Read complete workflow attempts, newest first, scoped to the active organization. */
  actionHistory: Scalars['JSON']['output'];
  /**
   * Loads and returns blocks from generated manifest artifacts in the configured database.
   * Pass `scry_identifier` to load an exact Scryr map, or `sample` for legacy lookup.
   */
  blocks: Array<Block>;
  /** Fetch configured runtime metrics once when opening a diagram. Block polling never calls this. */
  diagramMetrics: Scalars['JSON']['output'];
  health: HealthStatus;
  /** Read the selected diagram's complete source document. */
  manifestDocument: Scalars['JSON']['output'];
  /** Execute a named local declaration without persisting a diagram. */
  manifestQuery: Scalars['JSON']['output'];
  /** Read operational observations for the active organization. */
  reportHistory: Scalars['JSON']['output'];
  /** Lists persisted Scryr maps available in the configured database. */
  scryrMaps: Array<ScryrMap>;
};


export type QueryRootActionHistoryArgs = {
  limit?: Scalars['Int']['input'];
  manifestId: Scalars['String']['input'];
  offset?: Scalars['Int']['input'];
};


export type QueryRootBlocksArgs = {
  sample?: InputMaybe<Scalars['String']['input']>;
  scryIdentifier?: InputMaybe<Scalars['String']['input']>;
};


export type QueryRootDiagramMetricsArgs = {
  sample?: InputMaybe<Scalars['String']['input']>;
  scryIdentifier?: InputMaybe<Scalars['String']['input']>;
};


export type QueryRootManifestDocumentArgs = {
  identifier: Scalars['String']['input'];
};


export type QueryRootManifestQueryArgs = {
  name: Scalars['String']['input'];
  source: Scalars['JSON']['input'];
};


export type QueryRootReportHistoryArgs = {
  limit?: Scalars['Int']['input'];
  manifestId: Scalars['String']['input'];
  offset?: Scalars['Int']['input'];
};

/** A persisted Scryr map artifact available in storage. */
export type ScryrMap = {
  __typename?: 'ScryrMap';
  /** Clerk organization id that owns the uploaded manifest. */
  clerkOrgId: Scalars['String']['output'];
  /** Uploaded manifest file name. */
  fileName: Scalars['String']['output'];
  /** Folder path containing the uploaded manifest file. */
  folderPath: Scalars['String']['output'];
  /** Git commit SHA associated with the upload, when supplied. */
  gitCommitSha?: Maybe<Scalars['String']['output']>;
  /** Public map identifier used by the UI. */
  id: Scalars['String']['output'];
  /** Artifact key used by legacy sample-based lookups. */
  key: Scalars['String']['output'];
  /** Human-readable map name from the declared Diagram. */
  name: Scalars['String']['output'];
  /** Clerk organization slug cached from the uploader's active session. */
  orgSlug?: Maybe<Scalars['String']['output']>;
  /** Stable Scryr identifier, normally the top-level Diagram variable name. */
  scryIdentifier: Scalars['String']['output'];
  /** Last update timestamp rendered by the database. */
  updatedAt: Scalars['String']['output'];
};

/** Input payload for upserting a generated manifest artifact. */
export type UpsertGeneratedManifestInput = {
  /** Persisted manifest artifact key. */
  artifactKey: Scalars['String']['input'];
  /** Logical artifact kind. */
  artifactKind: ArtifactKind;
  /** Full generated artifact content. */
  content: Scalars['String']['input'];
  /** Uploaded manifest file name. */
  fileName?: InputMaybe<Scalars['String']['input']>;
  /** Folder path containing the uploaded manifest file. */
  folderPath?: InputMaybe<Scalars['String']['input']>;
  /** Git commit SHA associated with the upload, when supplied by the caller. */
  gitCommitSha?: InputMaybe<Scalars['String']['input']>;
  /** Human-readable map name from the declared Diagram. */
  name?: InputMaybe<Scalars['String']['input']>;
  /** Stable Scryr identifier, normally the top-level Diagram variable name. */
  scryIdentifier?: InputMaybe<Scalars['String']['input']>;
};

/** Payload returned after upserting a generated manifest artifact. */
export type UpsertGeneratedManifestPayload = {
  __typename?: 'UpsertGeneratedManifestPayload';
  /** Persisted row identifier. */
  id: Scalars['UUID']['output'];
};
