/** Internal type. DO NOT USE DIRECTLY. */
type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
/** Internal type. DO NOT USE DIRECTLY. */
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
import { graphqlFetcher } from './client';
import type { DocumentTypeDecoration } from '@graphql-typed-document-node/core';
import { useQuery, type UseQueryOptions } from '@tanstack/react-query';
export type CollectorState =
  | 'CANCELLED'
  | 'DISABLED'
  | 'ERROR'
  | 'INCOMPATIBLE_TOOL'
  | 'MISSING_TOOL'
  | 'NEEDS_LOGIN'
  | 'READY'
  | 'RUNNING'
  | 'WAITING';

export type EvidenceSection =
  | 'CHECKS'
  | 'DEPENDENCIES'
  | 'METRICS'
  | 'PERFORMANCE'
  | 'REPOSITORY'
  | 'TESTS';

export type HealthCheckQueryVariables = Exact<{ [key: string]: never; }>;


export type HealthCheckQuery = { health: { status: string, databaseOk: boolean, databaseMessage: string } };

export type GetScryrMapsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetScryrMapsQuery = { scryrMaps: Array<{ id: string, key: string, scryIdentifier: string, name: string, clerkOrgId: string, orgSlug: string | null, folderPath: string, fileName: string, gitCommitSha: string | null, updatedAt: string }> };

export type GetBlocksQueryVariables = Exact<{
  scryIdentifier?: string | null | undefined;
  sample?: string | null | undefined;
  workspaceId?: string | null | undefined;
}>;


export type GetBlocksQuery = { blocks: Array<{ name: string | null, description: string | null, version: string | null, lineNumber: number | null, consumerType: string | null, icon: string | null, language: string | null, frameworks: Array<string>, deployment: string | null, deploymentProvider: string | null, connections: Array<string>, docs: Array<string>, ownerTeam: string | null, tags: Array<string>, authType: string | null, monitoring: string | null, logAggregation: string | null, tracing: string | null, iacTool: string | null, maxReplicas: number | null, minReplicas: number | null, rawJsonString: string, links: Array<{ siteName: string | null, httpUrl: string | null }>, evidence: Array<{ manifestId: string, section: EvidenceSection, collectorId: string, integration: string, workspaceId: string, state: CollectorState, message: string | null, updatedAt: string | null, freshnessSeconds: number, stale: boolean, outdated: boolean, latest: { schemaVersion: number, observationId: string, manifestId: string, section: EvidenceSection, collectorId: string, integration: string, workspaceId: string, environment: string, scope: string, planRevision: string, collectorRevision: string, runId: string, attempt: number, observedAt: string, sourceUpdatedAt: string | null, startedAt: string, recordedAt: string | null, inputFingerprint: string, commitSha: string | null, branch: string | null, dirty: boolean | null, toolVersion: string | null, upstreamFingerprint: string | null, policyRevision: string | null, result:
          | { __typename: 'BenchmarkResult', name: string, meanSeconds: number, stddevSeconds: number, medianSeconds: number, runs: number, command: string, baselineMeanSeconds: number | null, machine: string }
          | { __typename: 'CheckResult', name: string, passed: boolean, durationSeconds: number, diagnostics: Array<{ message: string, path: string | null, line: number | null, severity: string }> }
          | { __typename: 'CoverageResult', suite: string, covered: number, total: number }
          | { __typename: 'GitResult', branch: string | null, commit: string | null, dirty: boolean, changedFiles: number, ahead: number, behind: number, remoteUrl: string | null }
          | { __typename: 'InventoryResult', complete: boolean, artifactHash: string, totalPackages: number, totalRelationships: number, licensedPackages: number, unknownLicensePackages: number, packages: Array<{ id: string, name: string, version: string, ecosystem: string, purl: string | null, paths: Array<string>, licenses: Array<string> }>, relationships: Array<{ from: string, to: string }> }
          | { __typename: 'LicenseResult', policyRevision: string, inventoryHash: string, complete: boolean, totalItems: number, allowedCount: number, deniedCount: number, reviewCount: number, items: Array<{ packageId: string, expression: string | null, decision: string, reason: string }> }
          | { __typename: 'MetricsResult', scrapedAt: string, complete: boolean, samples: Array<{ name: string, title: string | null, value: number, unit: string | null, metricType: string, labels: Array<{ name: string, value: string }> }> }
          | { __typename: 'PullRequestsResult', repository: string, complete: boolean, items: Array<{ number: number, title: string, url: string, state: string, reviewDecision: string | null, headRefName: string | null, headRefOid: string | null }> }
          | { __typename: 'TestResult', suite: string, passing: number, failing: number, errors: number, skipped: number, durationSeconds: number, cases: Array<{ name: string, suite: string, status: string, message: string | null, durationSeconds: number }> }
          | { __typename: 'VulnerabilityResult', inventoryHash: string, databaseAgeSeconds: number | null, databaseVersion: string | null, complete: boolean, totalFindings: number, affectedPackages: number, criticalCount: number, highCount: number, mediumCount: number, lowCount: number, unknownSeverityCount: number, items: Array<{ packageId: string, advisoryId: string, aliases: Array<string>, severity: string, fixVersions: Array<string>, advisoryUrl: string | null }> }
          | { __typename: 'WorkflowsResult', repository: string, complete: boolean, items: Array<{ runId: string, attempt: number, name: string, status: string, conclusion: string | null, branch: string, commit: string, url: string, updatedAt: string }> }
         } | null }> }> };

export type EvidenceObservationIdentityFragment = { schemaVersion: number, observationId: string, manifestId: string, section: EvidenceSection, collectorId: string, integration: string, workspaceId: string, environment: string, scope: string, planRevision: string, collectorRevision: string, runId: string, attempt: number, observedAt: string, sourceUpdatedAt: string | null, startedAt: string, recordedAt: string | null, inputFingerprint: string, commitSha: string | null, branch: string | null, dirty: boolean | null, toolVersion: string | null, upstreamFingerprint: string | null, policyRevision: string | null };

type NonDependencyEvidenceFields_BenchmarkResult_Fragment = { __typename: 'BenchmarkResult', name: string, meanSeconds: number, stddevSeconds: number, medianSeconds: number, runs: number, command: string, baselineMeanSeconds: number | null, machine: string };

type NonDependencyEvidenceFields_CheckResult_Fragment = { __typename: 'CheckResult', name: string, passed: boolean, durationSeconds: number, diagnostics: Array<{ message: string, path: string | null, line: number | null, severity: string }> };

type NonDependencyEvidenceFields_CoverageResult_Fragment = { __typename: 'CoverageResult', suite: string, covered: number, total: number };

type NonDependencyEvidenceFields_GitResult_Fragment = { __typename: 'GitResult', branch: string | null, commit: string | null, dirty: boolean, changedFiles: number, ahead: number, behind: number, remoteUrl: string | null };

type NonDependencyEvidenceFields_InventoryResult_Fragment = { __typename: 'InventoryResult' };

type NonDependencyEvidenceFields_LicenseResult_Fragment = { __typename: 'LicenseResult' };

type NonDependencyEvidenceFields_MetricsResult_Fragment = { __typename: 'MetricsResult', scrapedAt: string, complete: boolean, samples: Array<{ name: string, title: string | null, value: number, unit: string | null, metricType: string, labels: Array<{ name: string, value: string }> }> };

type NonDependencyEvidenceFields_PullRequestsResult_Fragment = { __typename: 'PullRequestsResult', repository: string, complete: boolean, items: Array<{ number: number, title: string, url: string, state: string, reviewDecision: string | null, headRefName: string | null, headRefOid: string | null }> };

type NonDependencyEvidenceFields_TestResult_Fragment = { __typename: 'TestResult', suite: string, passing: number, failing: number, errors: number, skipped: number, durationSeconds: number, cases: Array<{ name: string, suite: string, status: string, message: string | null, durationSeconds: number }> };

type NonDependencyEvidenceFields_VulnerabilityResult_Fragment = { __typename: 'VulnerabilityResult' };

type NonDependencyEvidenceFields_WorkflowsResult_Fragment = { __typename: 'WorkflowsResult', repository: string, complete: boolean, items: Array<{ runId: string, attempt: number, name: string, status: string, conclusion: string | null, branch: string, commit: string, url: string, updatedAt: string }> };

export type NonDependencyEvidenceFieldsFragment =
  | NonDependencyEvidenceFields_BenchmarkResult_Fragment
  | NonDependencyEvidenceFields_CheckResult_Fragment
  | NonDependencyEvidenceFields_CoverageResult_Fragment
  | NonDependencyEvidenceFields_GitResult_Fragment
  | NonDependencyEvidenceFields_InventoryResult_Fragment
  | NonDependencyEvidenceFields_LicenseResult_Fragment
  | NonDependencyEvidenceFields_MetricsResult_Fragment
  | NonDependencyEvidenceFields_PullRequestsResult_Fragment
  | NonDependencyEvidenceFields_TestResult_Fragment
  | NonDependencyEvidenceFields_VulnerabilityResult_Fragment
  | NonDependencyEvidenceFields_WorkflowsResult_Fragment
;

export type PackageFieldsFragment = { id: string, name: string, version: string, ecosystem: string, purl: string | null, paths: Array<string>, licenses: Array<string> };

export type LicenseFindingFieldsFragment = { packageId: string, expression: string | null, decision: string, reason: string };

export type VulnerabilityFindingFieldsFragment = { packageId: string, advisoryId: string, aliases: Array<string>, severity: string, fixVersions: Array<string>, advisoryUrl: string | null };

export type InventorySummaryFragment = { complete: boolean, artifactHash: string, totalPackages: number, totalRelationships: number, licensedPackages: number, unknownLicensePackages: number };

export type LicenseSummaryFragment = { policyRevision: string, inventoryHash: string, complete: boolean, totalItems: number, allowedCount: number, deniedCount: number, reviewCount: number };

export type VulnerabilitySummaryFragment = { inventoryHash: string, databaseAgeSeconds: number | null, databaseVersion: string | null, complete: boolean, totalFindings: number, affectedPackages: number, criticalCount: number, highCount: number, mediumCount: number, lowCount: number, unknownSeverityCount: number };

export type EvidenceObservationFieldsFragment = { schemaVersion: number, observationId: string, manifestId: string, section: EvidenceSection, collectorId: string, integration: string, workspaceId: string, environment: string, scope: string, planRevision: string, collectorRevision: string, runId: string, attempt: number, observedAt: string, sourceUpdatedAt: string | null, startedAt: string, recordedAt: string | null, inputFingerprint: string, commitSha: string | null, branch: string | null, dirty: boolean | null, toolVersion: string | null, upstreamFingerprint: string | null, policyRevision: string | null, result:
    | { __typename: 'BenchmarkResult', name: string, meanSeconds: number, stddevSeconds: number, medianSeconds: number, runs: number, command: string, baselineMeanSeconds: number | null, machine: string }
    | { __typename: 'CheckResult', name: string, passed: boolean, durationSeconds: number, diagnostics: Array<{ message: string, path: string | null, line: number | null, severity: string }> }
    | { __typename: 'CoverageResult', suite: string, covered: number, total: number }
    | { __typename: 'GitResult', branch: string | null, commit: string | null, dirty: boolean, changedFiles: number, ahead: number, behind: number, remoteUrl: string | null }
    | { __typename: 'InventoryResult', complete: boolean, artifactHash: string, totalPackages: number, totalRelationships: number, licensedPackages: number, unknownLicensePackages: number, packages: Array<{ id: string, name: string, version: string, ecosystem: string, purl: string | null, paths: Array<string>, licenses: Array<string> }>, relationships: Array<{ from: string, to: string }> }
    | { __typename: 'LicenseResult', policyRevision: string, inventoryHash: string, complete: boolean, totalItems: number, allowedCount: number, deniedCount: number, reviewCount: number, items: Array<{ packageId: string, expression: string | null, decision: string, reason: string }> }
    | { __typename: 'MetricsResult', scrapedAt: string, complete: boolean, samples: Array<{ name: string, title: string | null, value: number, unit: string | null, metricType: string, labels: Array<{ name: string, value: string }> }> }
    | { __typename: 'PullRequestsResult', repository: string, complete: boolean, items: Array<{ number: number, title: string, url: string, state: string, reviewDecision: string | null, headRefName: string | null, headRefOid: string | null }> }
    | { __typename: 'TestResult', suite: string, passing: number, failing: number, errors: number, skipped: number, durationSeconds: number, cases: Array<{ name: string, suite: string, status: string, message: string | null, durationSeconds: number }> }
    | { __typename: 'VulnerabilityResult', inventoryHash: string, databaseAgeSeconds: number | null, databaseVersion: string | null, complete: boolean, totalFindings: number, affectedPackages: number, criticalCount: number, highCount: number, mediumCount: number, lowCount: number, unknownSeverityCount: number, items: Array<{ packageId: string, advisoryId: string, aliases: Array<string>, severity: string, fixVersions: Array<string>, advisoryUrl: string | null }> }
    | { __typename: 'WorkflowsResult', repository: string, complete: boolean, items: Array<{ runId: string, attempt: number, name: string, status: string, conclusion: string | null, branch: string, commit: string, url: string, updatedAt: string }> }
   };

export type CollectorEvidenceFieldsFragment = { manifestId: string, section: EvidenceSection, collectorId: string, integration: string, workspaceId: string, state: CollectorState, message: string | null, updatedAt: string | null, freshnessSeconds: number, stale: boolean, outdated: boolean, latest: { schemaVersion: number, observationId: string, manifestId: string, section: EvidenceSection, collectorId: string, integration: string, workspaceId: string, environment: string, scope: string, planRevision: string, collectorRevision: string, runId: string, attempt: number, observedAt: string, sourceUpdatedAt: string | null, startedAt: string, recordedAt: string | null, inputFingerprint: string, commitSha: string | null, branch: string | null, dirty: boolean | null, toolVersion: string | null, upstreamFingerprint: string | null, policyRevision: string | null, result:
      | { __typename: 'BenchmarkResult', name: string, meanSeconds: number, stddevSeconds: number, medianSeconds: number, runs: number, command: string, baselineMeanSeconds: number | null, machine: string }
      | { __typename: 'CheckResult', name: string, passed: boolean, durationSeconds: number, diagnostics: Array<{ message: string, path: string | null, line: number | null, severity: string }> }
      | { __typename: 'CoverageResult', suite: string, covered: number, total: number }
      | { __typename: 'GitResult', branch: string | null, commit: string | null, dirty: boolean, changedFiles: number, ahead: number, behind: number, remoteUrl: string | null }
      | { __typename: 'InventoryResult', complete: boolean, artifactHash: string, totalPackages: number, totalRelationships: number, licensedPackages: number, unknownLicensePackages: number, packages: Array<{ id: string, name: string, version: string, ecosystem: string, purl: string | null, paths: Array<string>, licenses: Array<string> }>, relationships: Array<{ from: string, to: string }> }
      | { __typename: 'LicenseResult', policyRevision: string, inventoryHash: string, complete: boolean, totalItems: number, allowedCount: number, deniedCount: number, reviewCount: number, items: Array<{ packageId: string, expression: string | null, decision: string, reason: string }> }
      | { __typename: 'MetricsResult', scrapedAt: string, complete: boolean, samples: Array<{ name: string, title: string | null, value: number, unit: string | null, metricType: string, labels: Array<{ name: string, value: string }> }> }
      | { __typename: 'PullRequestsResult', repository: string, complete: boolean, items: Array<{ number: number, title: string, url: string, state: string, reviewDecision: string | null, headRefName: string | null, headRefOid: string | null }> }
      | { __typename: 'TestResult', suite: string, passing: number, failing: number, errors: number, skipped: number, durationSeconds: number, cases: Array<{ name: string, suite: string, status: string, message: string | null, durationSeconds: number }> }
      | { __typename: 'VulnerabilityResult', inventoryHash: string, databaseAgeSeconds: number | null, databaseVersion: string | null, complete: boolean, totalFindings: number, affectedPackages: number, criticalCount: number, highCount: number, mediumCount: number, lowCount: number, unknownSeverityCount: number, items: Array<{ packageId: string, advisoryId: string, aliases: Array<string>, severity: string, fixVersions: Array<string>, advisoryUrl: string | null }> }
      | { __typename: 'WorkflowsResult', repository: string, complete: boolean, items: Array<{ runId: string, attempt: number, name: string, status: string, conclusion: string | null, branch: string, commit: string, url: string, updatedAt: string }> }
     } | null };

export type GetEvidenceHistoryQueryVariables = Exact<{
  manifestId: string;
  section: EvidenceSection;
  collectorId: string;
  workspaceId: string;
  limit?: number;
  offset?: number;
}>;


export type GetEvidenceHistoryQuery = { evidenceHistory: Array<{ schemaVersion: number, observationId: string, manifestId: string, section: EvidenceSection, collectorId: string, integration: string, workspaceId: string, environment: string, scope: string, planRevision: string, collectorRevision: string, runId: string, attempt: number, observedAt: string, sourceUpdatedAt: string | null, startedAt: string, recordedAt: string | null, inputFingerprint: string, commitSha: string | null, branch: string | null, dirty: boolean | null, toolVersion: string | null, upstreamFingerprint: string | null, policyRevision: string | null, result:
      | { __typename: 'BenchmarkResult', name: string, meanSeconds: number, stddevSeconds: number, medianSeconds: number, runs: number, command: string, baselineMeanSeconds: number | null, machine: string }
      | { __typename: 'CheckResult', name: string, passed: boolean, durationSeconds: number, diagnostics: Array<{ message: string, path: string | null, line: number | null, severity: string }> }
      | { __typename: 'CoverageResult', suite: string, covered: number, total: number }
      | { __typename: 'GitResult', branch: string | null, commit: string | null, dirty: boolean, changedFiles: number, ahead: number, behind: number, remoteUrl: string | null }
      | { __typename: 'InventoryResult', complete: boolean, artifactHash: string, totalPackages: number, totalRelationships: number, licensedPackages: number, unknownLicensePackages: number, packages: Array<{ id: string, name: string, version: string, ecosystem: string, purl: string | null, paths: Array<string>, licenses: Array<string> }>, relationships: Array<{ from: string, to: string }> }
      | { __typename: 'LicenseResult', policyRevision: string, inventoryHash: string, complete: boolean, totalItems: number, allowedCount: number, deniedCount: number, reviewCount: number, items: Array<{ packageId: string, expression: string | null, decision: string, reason: string }> }
      | { __typename: 'MetricsResult', scrapedAt: string, complete: boolean, samples: Array<{ name: string, title: string | null, value: number, unit: string | null, metricType: string, labels: Array<{ name: string, value: string }> }> }
      | { __typename: 'PullRequestsResult', repository: string, complete: boolean, items: Array<{ number: number, title: string, url: string, state: string, reviewDecision: string | null, headRefName: string | null, headRefOid: string | null }> }
      | { __typename: 'TestResult', suite: string, passing: number, failing: number, errors: number, skipped: number, durationSeconds: number, cases: Array<{ name: string, suite: string, status: string, message: string | null, durationSeconds: number }> }
      | { __typename: 'VulnerabilityResult', inventoryHash: string, databaseAgeSeconds: number | null, databaseVersion: string | null, complete: boolean, totalFindings: number, affectedPackages: number, criticalCount: number, highCount: number, mediumCount: number, lowCount: number, unknownSeverityCount: number, items: Array<{ packageId: string, advisoryId: string, aliases: Array<string>, severity: string, fixVersions: Array<string>, advisoryUrl: string | null }> }
      | { __typename: 'WorkflowsResult', repository: string, complete: boolean, items: Array<{ runId: string, attempt: number, name: string, status: string, conclusion: string | null, branch: string, commit: string, url: string, updatedAt: string }> }
     }> };

export type GetEvidenceObservationQueryVariables = Exact<{
  observationId: string;
  workspaceId: string;
  limit?: number;
  offset?: number;
  search?: string | null | undefined;
  filter?: string | null | undefined;
}>;


export type GetEvidenceObservationQuery = { evidenceObservation: { schemaVersion: number, observationId: string, manifestId: string, section: EvidenceSection, collectorId: string, integration: string, workspaceId: string, environment: string, scope: string, planRevision: string, collectorRevision: string, runId: string, attempt: number, observedAt: string, sourceUpdatedAt: string | null, startedAt: string, recordedAt: string | null, inputFingerprint: string, commitSha: string | null, branch: string | null, dirty: boolean | null, toolVersion: string | null, upstreamFingerprint: string | null, policyRevision: string | null, result:
      | { __typename: 'BenchmarkResult', name: string, meanSeconds: number, stddevSeconds: number, medianSeconds: number, runs: number, command: string, baselineMeanSeconds: number | null, machine: string }
      | { __typename: 'CheckResult', name: string, passed: boolean, durationSeconds: number, diagnostics: Array<{ message: string, path: string | null, line: number | null, severity: string }> }
      | { __typename: 'CoverageResult', suite: string, covered: number, total: number }
      | { __typename: 'GitResult', branch: string | null, commit: string | null, dirty: boolean, changedFiles: number, ahead: number, behind: number, remoteUrl: string | null }
      | { __typename: 'InventoryResult', matchingPackages: number, complete: boolean, artifactHash: string, totalPackages: number, totalRelationships: number, licensedPackages: number, unknownLicensePackages: number, packages: Array<{ id: string, name: string, version: string, ecosystem: string, purl: string | null, paths: Array<string>, licenses: Array<string> }>, relationships: Array<{ from: string, to: string }> }
      | { __typename: 'LicenseResult', matchingItems: number, policyRevision: string, inventoryHash: string, complete: boolean, totalItems: number, allowedCount: number, deniedCount: number, reviewCount: number, items: Array<{ packageId: string, expression: string | null, decision: string, reason: string }> }
      | { __typename: 'MetricsResult', scrapedAt: string, complete: boolean, samples: Array<{ name: string, title: string | null, value: number, unit: string | null, metricType: string, labels: Array<{ name: string, value: string }> }> }
      | { __typename: 'PullRequestsResult', repository: string, complete: boolean, items: Array<{ number: number, title: string, url: string, state: string, reviewDecision: string | null, headRefName: string | null, headRefOid: string | null }> }
      | { __typename: 'TestResult', suite: string, passing: number, failing: number, errors: number, skipped: number, durationSeconds: number, cases: Array<{ name: string, suite: string, status: string, message: string | null, durationSeconds: number }> }
      | { __typename: 'VulnerabilityResult', matchingFindings: number, inventoryHash: string, databaseAgeSeconds: number | null, databaseVersion: string | null, complete: boolean, totalFindings: number, affectedPackages: number, criticalCount: number, highCount: number, mediumCount: number, lowCount: number, unknownSeverityCount: number, items: Array<{ packageId: string, advisoryId: string, aliases: Array<string>, severity: string, fixVersions: Array<string>, advisoryUrl: string | null }> }
      | { __typename: 'WorkflowsResult', repository: string, complete: boolean, items: Array<{ runId: string, attempt: number, name: string, status: string, conclusion: string | null, branch: string, commit: string, url: string, updatedAt: string }> }
     } | null };

export type GetInventoryPackagesQueryVariables = Exact<{
  observationId: string;
  workspaceId: string;
  ids: Array<string> | string;
}>;


export type GetInventoryPackagesQuery = { evidenceObservation: { observationId: string, workspaceId: string, environment: string, result:
      | { __typename: 'BenchmarkResult' }
      | { __typename: 'CheckResult' }
      | { __typename: 'CoverageResult' }
      | { __typename: 'GitResult' }
      | { __typename: 'InventoryResult', artifactHash: string, packages: Array<{ id: string, name: string, version: string, ecosystem: string, purl: string | null, paths: Array<string>, licenses: Array<string> }> }
      | { __typename: 'LicenseResult' }
      | { __typename: 'MetricsResult' }
      | { __typename: 'PullRequestsResult' }
      | { __typename: 'TestResult' }
      | { __typename: 'VulnerabilityResult' }
      | { __typename: 'WorkflowsResult' }
     } | null };


export class TypedDocumentString<TResult, TVariables>
  extends String
  implements DocumentTypeDecoration<TResult, TVariables>
{
  __apiType?: NonNullable<DocumentTypeDecoration<TResult, TVariables>['__apiType']>;
  private value: string;
  public __meta__?: Record<string, any> | undefined;

  constructor(value: string, __meta__?: Record<string, any> | undefined) {
    super(value);
    this.value = value;
    this.__meta__ = __meta__;
  }

  override toString(): string & DocumentTypeDecoration<TResult, TVariables> {
    return this.value;
  }
}
export const EvidenceObservationIdentityFragmentDoc = new TypedDocumentString(`
    fragment EvidenceObservationIdentity on EvidenceObservation {
  schemaVersion
  observationId
  manifestId
  section
  collectorId
  integration
  workspaceId
  environment
  scope
  planRevision
  collectorRevision
  runId
  attempt
  observedAt
  sourceUpdatedAt
  startedAt
  recordedAt
  inputFingerprint
  commitSha
  branch
  dirty
  toolVersion
  upstreamFingerprint
  policyRevision
}
    `, {"fragmentName":"EvidenceObservationIdentity"});
export const NonDependencyEvidenceFieldsFragmentDoc = new TypedDocumentString(`
    fragment NonDependencyEvidenceFields on EvidenceResult {
  __typename
  ... on GitResult {
    branch
    commit
    dirty
    changedFiles
    ahead
    behind
    remoteUrl
  }
  ... on PullRequestsResult {
    repository
    complete
    items {
      number
      title
      url
      state
      reviewDecision
      headRefName
      headRefOid
    }
  }
  ... on WorkflowsResult {
    repository
    complete
    items {
      runId
      attempt
      name
      status
      conclusion
      branch
      commit
      url
      updatedAt
    }
  }
  ... on CheckResult {
    name
    passed
    durationSeconds
    diagnostics {
      message
      path
      line
      severity
    }
  }
  ... on TestResult {
    suite
    passing
    failing
    errors
    skipped
    durationSeconds
    cases {
      name
      suite
      status
      message
      durationSeconds
    }
  }
  ... on CoverageResult {
    suite
    covered
    total
  }
  ... on MetricsResult {
    scrapedAt
    complete
    samples {
      name
      title
      labels {
        name
        value
      }
      value
      unit
      metricType
    }
  }
  ... on BenchmarkResult {
    name
    meanSeconds
    stddevSeconds
    medianSeconds
    runs
    command
    baselineMeanSeconds
    machine
  }
}
    `, {"fragmentName":"NonDependencyEvidenceFields"});
export const InventorySummaryFragmentDoc = new TypedDocumentString(`
    fragment InventorySummary on InventoryResult {
  complete
  artifactHash
  totalPackages
  totalRelationships
  licensedPackages
  unknownLicensePackages
}
    `, {"fragmentName":"InventorySummary"});
export const PackageFieldsFragmentDoc = new TypedDocumentString(`
    fragment PackageFields on Package {
  id
  name
  version
  ecosystem
  purl
  paths
  licenses
}
    `, {"fragmentName":"PackageFields"});
export const LicenseSummaryFragmentDoc = new TypedDocumentString(`
    fragment LicenseSummary on LicenseResult {
  policyRevision
  inventoryHash
  complete
  totalItems
  allowedCount
  deniedCount
  reviewCount
}
    `, {"fragmentName":"LicenseSummary"});
export const LicenseFindingFieldsFragmentDoc = new TypedDocumentString(`
    fragment LicenseFindingFields on LicenseFinding {
  packageId
  expression
  decision
  reason
}
    `, {"fragmentName":"LicenseFindingFields"});
export const VulnerabilitySummaryFragmentDoc = new TypedDocumentString(`
    fragment VulnerabilitySummary on VulnerabilityResult {
  inventoryHash
  databaseAgeSeconds
  databaseVersion
  complete
  totalFindings
  affectedPackages
  criticalCount
  highCount
  mediumCount
  lowCount
  unknownSeverityCount
}
    `, {"fragmentName":"VulnerabilitySummary"});
export const VulnerabilityFindingFieldsFragmentDoc = new TypedDocumentString(`
    fragment VulnerabilityFindingFields on VulnerabilityFinding {
  packageId
  advisoryId
  aliases
  severity
  fixVersions
  advisoryUrl: url
}
    `, {"fragmentName":"VulnerabilityFindingFields"});
export const EvidenceObservationFieldsFragmentDoc = new TypedDocumentString(`
    fragment EvidenceObservationFields on EvidenceObservation {
  ...EvidenceObservationIdentity
  result {
    ...NonDependencyEvidenceFields
    ... on InventoryResult {
      ...InventorySummary
      packages(limit: 0) {
        ...PackageFields
      }
      relationships(limit: 0) {
        from
        to
      }
    }
    ... on LicenseResult {
      ...LicenseSummary
      items(limit: 0) {
        ...LicenseFindingFields
      }
    }
    ... on VulnerabilityResult {
      ...VulnerabilitySummary
      items(limit: 0) {
        ...VulnerabilityFindingFields
      }
    }
  }
}
    fragment EvidenceObservationIdentity on EvidenceObservation {
  schemaVersion
  observationId
  manifestId
  section
  collectorId
  integration
  workspaceId
  environment
  scope
  planRevision
  collectorRevision
  runId
  attempt
  observedAt
  sourceUpdatedAt
  startedAt
  recordedAt
  inputFingerprint
  commitSha
  branch
  dirty
  toolVersion
  upstreamFingerprint
  policyRevision
}
fragment NonDependencyEvidenceFields on EvidenceResult {
  __typename
  ... on GitResult {
    branch
    commit
    dirty
    changedFiles
    ahead
    behind
    remoteUrl
  }
  ... on PullRequestsResult {
    repository
    complete
    items {
      number
      title
      url
      state
      reviewDecision
      headRefName
      headRefOid
    }
  }
  ... on WorkflowsResult {
    repository
    complete
    items {
      runId
      attempt
      name
      status
      conclusion
      branch
      commit
      url
      updatedAt
    }
  }
  ... on CheckResult {
    name
    passed
    durationSeconds
    diagnostics {
      message
      path
      line
      severity
    }
  }
  ... on TestResult {
    suite
    passing
    failing
    errors
    skipped
    durationSeconds
    cases {
      name
      suite
      status
      message
      durationSeconds
    }
  }
  ... on CoverageResult {
    suite
    covered
    total
  }
  ... on MetricsResult {
    scrapedAt
    complete
    samples {
      name
      title
      labels {
        name
        value
      }
      value
      unit
      metricType
    }
  }
  ... on BenchmarkResult {
    name
    meanSeconds
    stddevSeconds
    medianSeconds
    runs
    command
    baselineMeanSeconds
    machine
  }
}
fragment PackageFields on Package {
  id
  name
  version
  ecosystem
  purl
  paths
  licenses
}
fragment LicenseFindingFields on LicenseFinding {
  packageId
  expression
  decision
  reason
}
fragment VulnerabilityFindingFields on VulnerabilityFinding {
  packageId
  advisoryId
  aliases
  severity
  fixVersions
  advisoryUrl: url
}
fragment InventorySummary on InventoryResult {
  complete
  artifactHash
  totalPackages
  totalRelationships
  licensedPackages
  unknownLicensePackages
}
fragment LicenseSummary on LicenseResult {
  policyRevision
  inventoryHash
  complete
  totalItems
  allowedCount
  deniedCount
  reviewCount
}
fragment VulnerabilitySummary on VulnerabilityResult {
  inventoryHash
  databaseAgeSeconds
  databaseVersion
  complete
  totalFindings
  affectedPackages
  criticalCount
  highCount
  mediumCount
  lowCount
  unknownSeverityCount
}`, {"fragmentName":"EvidenceObservationFields"});
export const CollectorEvidenceFieldsFragmentDoc = new TypedDocumentString(`
    fragment CollectorEvidenceFields on CollectorEvidence {
  manifestId
  section
  collectorId
  integration
  workspaceId
  state
  message
  updatedAt
  freshnessSeconds
  stale
  outdated
  latest {
    ...EvidenceObservationFields
  }
}
    fragment EvidenceObservationIdentity on EvidenceObservation {
  schemaVersion
  observationId
  manifestId
  section
  collectorId
  integration
  workspaceId
  environment
  scope
  planRevision
  collectorRevision
  runId
  attempt
  observedAt
  sourceUpdatedAt
  startedAt
  recordedAt
  inputFingerprint
  commitSha
  branch
  dirty
  toolVersion
  upstreamFingerprint
  policyRevision
}
fragment NonDependencyEvidenceFields on EvidenceResult {
  __typename
  ... on GitResult {
    branch
    commit
    dirty
    changedFiles
    ahead
    behind
    remoteUrl
  }
  ... on PullRequestsResult {
    repository
    complete
    items {
      number
      title
      url
      state
      reviewDecision
      headRefName
      headRefOid
    }
  }
  ... on WorkflowsResult {
    repository
    complete
    items {
      runId
      attempt
      name
      status
      conclusion
      branch
      commit
      url
      updatedAt
    }
  }
  ... on CheckResult {
    name
    passed
    durationSeconds
    diagnostics {
      message
      path
      line
      severity
    }
  }
  ... on TestResult {
    suite
    passing
    failing
    errors
    skipped
    durationSeconds
    cases {
      name
      suite
      status
      message
      durationSeconds
    }
  }
  ... on CoverageResult {
    suite
    covered
    total
  }
  ... on MetricsResult {
    scrapedAt
    complete
    samples {
      name
      title
      labels {
        name
        value
      }
      value
      unit
      metricType
    }
  }
  ... on BenchmarkResult {
    name
    meanSeconds
    stddevSeconds
    medianSeconds
    runs
    command
    baselineMeanSeconds
    machine
  }
}
fragment PackageFields on Package {
  id
  name
  version
  ecosystem
  purl
  paths
  licenses
}
fragment LicenseFindingFields on LicenseFinding {
  packageId
  expression
  decision
  reason
}
fragment VulnerabilityFindingFields on VulnerabilityFinding {
  packageId
  advisoryId
  aliases
  severity
  fixVersions
  advisoryUrl: url
}
fragment InventorySummary on InventoryResult {
  complete
  artifactHash
  totalPackages
  totalRelationships
  licensedPackages
  unknownLicensePackages
}
fragment LicenseSummary on LicenseResult {
  policyRevision
  inventoryHash
  complete
  totalItems
  allowedCount
  deniedCount
  reviewCount
}
fragment VulnerabilitySummary on VulnerabilityResult {
  inventoryHash
  databaseAgeSeconds
  databaseVersion
  complete
  totalFindings
  affectedPackages
  criticalCount
  highCount
  mediumCount
  lowCount
  unknownSeverityCount
}
fragment EvidenceObservationFields on EvidenceObservation {
  ...EvidenceObservationIdentity
  result {
    ...NonDependencyEvidenceFields
    ... on InventoryResult {
      ...InventorySummary
      packages(limit: 0) {
        ...PackageFields
      }
      relationships(limit: 0) {
        from
        to
      }
    }
    ... on LicenseResult {
      ...LicenseSummary
      items(limit: 0) {
        ...LicenseFindingFields
      }
    }
    ... on VulnerabilityResult {
      ...VulnerabilitySummary
      items(limit: 0) {
        ...VulnerabilityFindingFields
      }
    }
  }
}`, {"fragmentName":"CollectorEvidenceFields"});
export const HealthCheckDocument = new TypedDocumentString(`
    query HealthCheck {
  health {
    status
    databaseOk
    databaseMessage
  }
}
    `);

export const useHealthCheckQuery = <
      TData = HealthCheckQuery,
      TError = unknown
    >(
      variables?: HealthCheckQueryVariables,
      options?: Omit<UseQueryOptions<HealthCheckQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<HealthCheckQuery, TError, TData>['queryKey'] }
    ) => {

    return useQuery<HealthCheckQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['HealthCheck'] : ['HealthCheck', variables],
    queryFn: graphqlFetcher<HealthCheckQuery, HealthCheckQueryVariables>(HealthCheckDocument, variables),
    ...options
  }
    )};

useHealthCheckQuery.document = HealthCheckDocument;

useHealthCheckQuery.getKey = (variables?: HealthCheckQueryVariables) => variables === undefined ? ['HealthCheck'] : ['HealthCheck', variables];

export const GetScryrMapsDocument = new TypedDocumentString(`
    query GetScryrMaps {
  scryrMaps {
    id
    key
    scryIdentifier
    name
    clerkOrgId
    orgSlug
    folderPath
    fileName
    gitCommitSha
    updatedAt
  }
}
    `);

export const useGetScryrMapsQuery = <
      TData = GetScryrMapsQuery,
      TError = unknown
    >(
      variables?: GetScryrMapsQueryVariables,
      options?: Omit<UseQueryOptions<GetScryrMapsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetScryrMapsQuery, TError, TData>['queryKey'] }
    ) => {

    return useQuery<GetScryrMapsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetScryrMaps'] : ['GetScryrMaps', variables],
    queryFn: graphqlFetcher<GetScryrMapsQuery, GetScryrMapsQueryVariables>(GetScryrMapsDocument, variables),
    ...options
  }
    )};

useGetScryrMapsQuery.document = GetScryrMapsDocument;

useGetScryrMapsQuery.getKey = (variables?: GetScryrMapsQueryVariables) => variables === undefined ? ['GetScryrMaps'] : ['GetScryrMaps', variables];

export const GetBlocksDocument = new TypedDocumentString(`
    query GetBlocks($scryIdentifier: String, $sample: String, $workspaceId: String) {
  blocks(
    scryIdentifier: $scryIdentifier
    sample: $sample
    workspaceId: $workspaceId
  ) {
    name
    description
    version
    lineNumber
    consumerType
    icon
    language
    frameworks
    deployment
    deploymentProvider
    connections
    docs
    ownerTeam
    tags
    authType
    monitoring
    logAggregation
    tracing
    iacTool
    maxReplicas
    minReplicas
    links {
      siteName
      httpUrl
    }
    evidence {
      ...CollectorEvidenceFields
    }
    rawJsonString
  }
}
    fragment EvidenceObservationIdentity on EvidenceObservation {
  schemaVersion
  observationId
  manifestId
  section
  collectorId
  integration
  workspaceId
  environment
  scope
  planRevision
  collectorRevision
  runId
  attempt
  observedAt
  sourceUpdatedAt
  startedAt
  recordedAt
  inputFingerprint
  commitSha
  branch
  dirty
  toolVersion
  upstreamFingerprint
  policyRevision
}
fragment NonDependencyEvidenceFields on EvidenceResult {
  __typename
  ... on GitResult {
    branch
    commit
    dirty
    changedFiles
    ahead
    behind
    remoteUrl
  }
  ... on PullRequestsResult {
    repository
    complete
    items {
      number
      title
      url
      state
      reviewDecision
      headRefName
      headRefOid
    }
  }
  ... on WorkflowsResult {
    repository
    complete
    items {
      runId
      attempt
      name
      status
      conclusion
      branch
      commit
      url
      updatedAt
    }
  }
  ... on CheckResult {
    name
    passed
    durationSeconds
    diagnostics {
      message
      path
      line
      severity
    }
  }
  ... on TestResult {
    suite
    passing
    failing
    errors
    skipped
    durationSeconds
    cases {
      name
      suite
      status
      message
      durationSeconds
    }
  }
  ... on CoverageResult {
    suite
    covered
    total
  }
  ... on MetricsResult {
    scrapedAt
    complete
    samples {
      name
      title
      labels {
        name
        value
      }
      value
      unit
      metricType
    }
  }
  ... on BenchmarkResult {
    name
    meanSeconds
    stddevSeconds
    medianSeconds
    runs
    command
    baselineMeanSeconds
    machine
  }
}
fragment PackageFields on Package {
  id
  name
  version
  ecosystem
  purl
  paths
  licenses
}
fragment LicenseFindingFields on LicenseFinding {
  packageId
  expression
  decision
  reason
}
fragment VulnerabilityFindingFields on VulnerabilityFinding {
  packageId
  advisoryId
  aliases
  severity
  fixVersions
  advisoryUrl: url
}
fragment InventorySummary on InventoryResult {
  complete
  artifactHash
  totalPackages
  totalRelationships
  licensedPackages
  unknownLicensePackages
}
fragment LicenseSummary on LicenseResult {
  policyRevision
  inventoryHash
  complete
  totalItems
  allowedCount
  deniedCount
  reviewCount
}
fragment VulnerabilitySummary on VulnerabilityResult {
  inventoryHash
  databaseAgeSeconds
  databaseVersion
  complete
  totalFindings
  affectedPackages
  criticalCount
  highCount
  mediumCount
  lowCount
  unknownSeverityCount
}
fragment EvidenceObservationFields on EvidenceObservation {
  ...EvidenceObservationIdentity
  result {
    ...NonDependencyEvidenceFields
    ... on InventoryResult {
      ...InventorySummary
      packages(limit: 0) {
        ...PackageFields
      }
      relationships(limit: 0) {
        from
        to
      }
    }
    ... on LicenseResult {
      ...LicenseSummary
      items(limit: 0) {
        ...LicenseFindingFields
      }
    }
    ... on VulnerabilityResult {
      ...VulnerabilitySummary
      items(limit: 0) {
        ...VulnerabilityFindingFields
      }
    }
  }
}
fragment CollectorEvidenceFields on CollectorEvidence {
  manifestId
  section
  collectorId
  integration
  workspaceId
  state
  message
  updatedAt
  freshnessSeconds
  stale
  outdated
  latest {
    ...EvidenceObservationFields
  }
}`);

export const useGetBlocksQuery = <
      TData = GetBlocksQuery,
      TError = unknown
    >(
      variables?: GetBlocksQueryVariables,
      options?: Omit<UseQueryOptions<GetBlocksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetBlocksQuery, TError, TData>['queryKey'] }
    ) => {

    return useQuery<GetBlocksQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetBlocks'] : ['GetBlocks', variables],
    queryFn: graphqlFetcher<GetBlocksQuery, GetBlocksQueryVariables>(GetBlocksDocument, variables),
    ...options
  }
    )};

useGetBlocksQuery.document = GetBlocksDocument;

useGetBlocksQuery.getKey = (variables?: GetBlocksQueryVariables) => variables === undefined ? ['GetBlocks'] : ['GetBlocks', variables];

export const GetEvidenceHistoryDocument = new TypedDocumentString(`
    query GetEvidenceHistory($manifestId: String!, $section: EvidenceSection!, $collectorId: String!, $workspaceId: String!, $limit: Int! = 20, $offset: Int! = 0) {
  evidenceHistory(
    manifestId: $manifestId
    section: $section
    collectorId: $collectorId
    workspaceId: $workspaceId
    limit: $limit
    offset: $offset
  ) {
    ...EvidenceObservationFields
  }
}
    fragment EvidenceObservationIdentity on EvidenceObservation {
  schemaVersion
  observationId
  manifestId
  section
  collectorId
  integration
  workspaceId
  environment
  scope
  planRevision
  collectorRevision
  runId
  attempt
  observedAt
  sourceUpdatedAt
  startedAt
  recordedAt
  inputFingerprint
  commitSha
  branch
  dirty
  toolVersion
  upstreamFingerprint
  policyRevision
}
fragment NonDependencyEvidenceFields on EvidenceResult {
  __typename
  ... on GitResult {
    branch
    commit
    dirty
    changedFiles
    ahead
    behind
    remoteUrl
  }
  ... on PullRequestsResult {
    repository
    complete
    items {
      number
      title
      url
      state
      reviewDecision
      headRefName
      headRefOid
    }
  }
  ... on WorkflowsResult {
    repository
    complete
    items {
      runId
      attempt
      name
      status
      conclusion
      branch
      commit
      url
      updatedAt
    }
  }
  ... on CheckResult {
    name
    passed
    durationSeconds
    diagnostics {
      message
      path
      line
      severity
    }
  }
  ... on TestResult {
    suite
    passing
    failing
    errors
    skipped
    durationSeconds
    cases {
      name
      suite
      status
      message
      durationSeconds
    }
  }
  ... on CoverageResult {
    suite
    covered
    total
  }
  ... on MetricsResult {
    scrapedAt
    complete
    samples {
      name
      title
      labels {
        name
        value
      }
      value
      unit
      metricType
    }
  }
  ... on BenchmarkResult {
    name
    meanSeconds
    stddevSeconds
    medianSeconds
    runs
    command
    baselineMeanSeconds
    machine
  }
}
fragment PackageFields on Package {
  id
  name
  version
  ecosystem
  purl
  paths
  licenses
}
fragment LicenseFindingFields on LicenseFinding {
  packageId
  expression
  decision
  reason
}
fragment VulnerabilityFindingFields on VulnerabilityFinding {
  packageId
  advisoryId
  aliases
  severity
  fixVersions
  advisoryUrl: url
}
fragment InventorySummary on InventoryResult {
  complete
  artifactHash
  totalPackages
  totalRelationships
  licensedPackages
  unknownLicensePackages
}
fragment LicenseSummary on LicenseResult {
  policyRevision
  inventoryHash
  complete
  totalItems
  allowedCount
  deniedCount
  reviewCount
}
fragment VulnerabilitySummary on VulnerabilityResult {
  inventoryHash
  databaseAgeSeconds
  databaseVersion
  complete
  totalFindings
  affectedPackages
  criticalCount
  highCount
  mediumCount
  lowCount
  unknownSeverityCount
}
fragment EvidenceObservationFields on EvidenceObservation {
  ...EvidenceObservationIdentity
  result {
    ...NonDependencyEvidenceFields
    ... on InventoryResult {
      ...InventorySummary
      packages(limit: 0) {
        ...PackageFields
      }
      relationships(limit: 0) {
        from
        to
      }
    }
    ... on LicenseResult {
      ...LicenseSummary
      items(limit: 0) {
        ...LicenseFindingFields
      }
    }
    ... on VulnerabilityResult {
      ...VulnerabilitySummary
      items(limit: 0) {
        ...VulnerabilityFindingFields
      }
    }
  }
}`);

export const useGetEvidenceHistoryQuery = <
      TData = GetEvidenceHistoryQuery,
      TError = unknown
    >(
      variables: GetEvidenceHistoryQueryVariables,
      options?: Omit<UseQueryOptions<GetEvidenceHistoryQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetEvidenceHistoryQuery, TError, TData>['queryKey'] }
    ) => {

    return useQuery<GetEvidenceHistoryQuery, TError, TData>(
      {
    queryKey: ['GetEvidenceHistory', variables],
    queryFn: graphqlFetcher<GetEvidenceHistoryQuery, GetEvidenceHistoryQueryVariables>(GetEvidenceHistoryDocument, variables),
    ...options
  }
    )};

useGetEvidenceHistoryQuery.document = GetEvidenceHistoryDocument;

useGetEvidenceHistoryQuery.getKey = (variables: GetEvidenceHistoryQueryVariables) => ['GetEvidenceHistory', variables];

export const GetEvidenceObservationDocument = new TypedDocumentString(`
    query GetEvidenceObservation($observationId: String!, $workspaceId: String!, $limit: Int! = 25, $offset: Int! = 0, $search: String, $filter: String) {
  evidenceObservation(observationId: $observationId, workspaceId: $workspaceId) {
    ...EvidenceObservationIdentity
    result {
      ...NonDependencyEvidenceFields
      ... on InventoryResult {
        ...InventorySummary
        matchingPackages(search: $search, filter: $filter)
        packages(limit: $limit, offset: $offset, search: $search, filter: $filter) {
          ...PackageFields
        }
        relationships(limit: 0) {
          from
          to
        }
      }
      ... on LicenseResult {
        ...LicenseSummary
        matchingItems(search: $search, filter: $filter)
        items(limit: $limit, offset: $offset, search: $search, filter: $filter) {
          ...LicenseFindingFields
        }
      }
      ... on VulnerabilityResult {
        ...VulnerabilitySummary
        matchingFindings(search: $search, filter: $filter)
        items(limit: $limit, offset: $offset, search: $search, filter: $filter) {
          ...VulnerabilityFindingFields
        }
      }
    }
  }
}
    fragment EvidenceObservationIdentity on EvidenceObservation {
  schemaVersion
  observationId
  manifestId
  section
  collectorId
  integration
  workspaceId
  environment
  scope
  planRevision
  collectorRevision
  runId
  attempt
  observedAt
  sourceUpdatedAt
  startedAt
  recordedAt
  inputFingerprint
  commitSha
  branch
  dirty
  toolVersion
  upstreamFingerprint
  policyRevision
}
fragment NonDependencyEvidenceFields on EvidenceResult {
  __typename
  ... on GitResult {
    branch
    commit
    dirty
    changedFiles
    ahead
    behind
    remoteUrl
  }
  ... on PullRequestsResult {
    repository
    complete
    items {
      number
      title
      url
      state
      reviewDecision
      headRefName
      headRefOid
    }
  }
  ... on WorkflowsResult {
    repository
    complete
    items {
      runId
      attempt
      name
      status
      conclusion
      branch
      commit
      url
      updatedAt
    }
  }
  ... on CheckResult {
    name
    passed
    durationSeconds
    diagnostics {
      message
      path
      line
      severity
    }
  }
  ... on TestResult {
    suite
    passing
    failing
    errors
    skipped
    durationSeconds
    cases {
      name
      suite
      status
      message
      durationSeconds
    }
  }
  ... on CoverageResult {
    suite
    covered
    total
  }
  ... on MetricsResult {
    scrapedAt
    complete
    samples {
      name
      title
      labels {
        name
        value
      }
      value
      unit
      metricType
    }
  }
  ... on BenchmarkResult {
    name
    meanSeconds
    stddevSeconds
    medianSeconds
    runs
    command
    baselineMeanSeconds
    machine
  }
}
fragment PackageFields on Package {
  id
  name
  version
  ecosystem
  purl
  paths
  licenses
}
fragment LicenseFindingFields on LicenseFinding {
  packageId
  expression
  decision
  reason
}
fragment VulnerabilityFindingFields on VulnerabilityFinding {
  packageId
  advisoryId
  aliases
  severity
  fixVersions
  advisoryUrl: url
}
fragment InventorySummary on InventoryResult {
  complete
  artifactHash
  totalPackages
  totalRelationships
  licensedPackages
  unknownLicensePackages
}
fragment LicenseSummary on LicenseResult {
  policyRevision
  inventoryHash
  complete
  totalItems
  allowedCount
  deniedCount
  reviewCount
}
fragment VulnerabilitySummary on VulnerabilityResult {
  inventoryHash
  databaseAgeSeconds
  databaseVersion
  complete
  totalFindings
  affectedPackages
  criticalCount
  highCount
  mediumCount
  lowCount
  unknownSeverityCount
}`);

export const useGetEvidenceObservationQuery = <
      TData = GetEvidenceObservationQuery,
      TError = unknown
    >(
      variables: GetEvidenceObservationQueryVariables,
      options?: Omit<UseQueryOptions<GetEvidenceObservationQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetEvidenceObservationQuery, TError, TData>['queryKey'] }
    ) => {

    return useQuery<GetEvidenceObservationQuery, TError, TData>(
      {
    queryKey: ['GetEvidenceObservation', variables],
    queryFn: graphqlFetcher<GetEvidenceObservationQuery, GetEvidenceObservationQueryVariables>(GetEvidenceObservationDocument, variables),
    ...options
  }
    )};

useGetEvidenceObservationQuery.document = GetEvidenceObservationDocument;

useGetEvidenceObservationQuery.getKey = (variables: GetEvidenceObservationQueryVariables) => ['GetEvidenceObservation', variables];

export const GetInventoryPackagesDocument = new TypedDocumentString(`
    query GetInventoryPackages($observationId: String!, $workspaceId: String!, $ids: [String!]!) {
  evidenceObservation(observationId: $observationId, workspaceId: $workspaceId) {
    observationId
    workspaceId
    environment
    result {
      __typename
      ... on InventoryResult {
        artifactHash
        packages(ids: $ids, limit: 100) {
          ...PackageFields
        }
      }
    }
  }
}
    fragment PackageFields on Package {
  id
  name
  version
  ecosystem
  purl
  paths
  licenses
}`);

export const useGetInventoryPackagesQuery = <
      TData = GetInventoryPackagesQuery,
      TError = unknown
    >(
      variables: GetInventoryPackagesQueryVariables,
      options?: Omit<UseQueryOptions<GetInventoryPackagesQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetInventoryPackagesQuery, TError, TData>['queryKey'] }
    ) => {

    return useQuery<GetInventoryPackagesQuery, TError, TData>(
      {
    queryKey: ['GetInventoryPackages', variables],
    queryFn: graphqlFetcher<GetInventoryPackagesQuery, GetInventoryPackagesQueryVariables>(GetInventoryPackagesDocument, variables),
    ...options
  }
    )};

useGetInventoryPackagesQuery.document = GetInventoryPackagesDocument;

useGetInventoryPackagesQuery.getKey = (variables: GetInventoryPackagesQueryVariables) => ['GetInventoryPackages', variables];
