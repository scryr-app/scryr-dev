/** Internal type. DO NOT USE DIRECTLY. */
type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
/** Internal type. DO NOT USE DIRECTLY. */
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
import { graphqlFetcher } from './client';
export * from './schema.generated';

import type * as Schema from '../../schema';

import type { DocumentTypeDecoration } from '@graphql-typed-document-node/core';
import { useQuery, type UseQueryOptions } from '@tanstack/react-query';
export type HealthCheckQueryVariables = Exact<{ [key: string]: never; }>;


export type HealthCheckQuery = { health: { status: string, databaseOk: boolean, databaseMessage: string } };

export type GetScryrMapsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetScryrMapsQuery = { scryrMaps: Array<{ id: string, key: string, scryIdentifier: string, name: string, clerkOrgId: string, orgSlug: string | null, folderPath: string, fileName: string, gitCommitSha: string | null, updatedAt: string }> };

export type GetBlocksQueryVariables = Exact<{
  scryIdentifier?: string | null | undefined;
  sample?: string | null | undefined;
}>;


export type GetBlocksQuery = { blocks: Array<{ manifestId: string | null, name: string | null, description: string | null, version: string | null, lineNumber: number | null, consumerType: string | null, icon: string | null, languages: Array<string>, frameworks: Array<string>, deployments: Array<string>, sourceCodeUrl: string | null, connections: Array<string>, docs: Array<string>, owners: string | null, tags: Array<string>, metricTools: Array<string>, cicdTool: string | null, rawJsonString: string, delivery: { kind: Schema.DeliveryProvider, tasks: { collect: { limit: number, verifyAttestations: boolean }, verify: { limit: number, verifyAttestations: boolean } }, repository: { host: string, owner: string, name: string }, builds: { workflow: string, artifactNames: Array<string> }, releases: { includeDrafts: boolean, includePrereleases: boolean }, environments: Array<{ name: string, deployments: { environment: string, task: string }, runtime: { workflow: string, artifactName: string, schema: string, maxAgeSeconds: number } | null, compatibility: { workflow: string, artifactName: string, schema: string, maxAgeSeconds: number } | null, rollback: { requireArtifactAvailable: boolean, requireVerifiedProvenance: boolean, requireCompatibilityEvidence: boolean } }> } | null, deliverySnapshot: { host: string, repository: string, observedAt: string, sources: Array<{ category: Schema.DeliverySourceCategory, environment: string | null, state: Schema.DeliveryEvidenceState, message: string | null }>, builds: Array<{ id: string, attempt: number, workflow: string, name: string, commitSha: string, status: Schema.DeliveryRunStatus, conclusion: Schema.DeliveryConclusion | null, createdAt: string, updatedAt: string, url: string, jobs: Array<{ id: string, name: string, status: Schema.DeliveryRunStatus, conclusion: Schema.DeliveryConclusion | null, url: string }> }>, artifacts: Array<{ id: string, kind: Schema.DeliveryArtifactKind, name: string, digest: string | null, runId: string | null, releaseId: string | null, expired: boolean, url: string | null, provenance: Schema.DeliveryVerification }>, releases: Array<{ id: string, tag: string, name: string | null, publishedAt: string | null, draft: boolean, prerelease: boolean, url: string, assetIds: Array<string> }>, deployments: Array<{ id: string, environment: string, commitSha: string, status: Schema.DeliveryDeploymentStatus, createdAt: string, updatedAt: string, url: string | null, statuses: Array<{ status: Schema.DeliveryDeploymentStatus, createdAt: string, url: string | null }> }>, links: Array<{ schema: string, manifestId: string, environment: string, deploymentId: string, buildRunId: string, buildAttempt: number, sourceRevision: string, artifactDigest: string, releaseId: string | null, configurationRevision: string }>, runtime: Array<{ schema: string, manifestId: string, environment: string, deploymentId: string, configurationRevision: string, schemaRevision: string, observedAt: string, workloads: Array<{ name: string, artifactDigest: string, replicas: number, readyReplicas: number }> }>, compatibility: Array<{ schema: string, manifestId: string, environment: string, candidateDigest: string, currentSchemaRevision: string, currentConfigurationRevision: string, candidateConfigurationRevision: string | null, suiteRevision: string, observedAt: string, checks: Array<{ name: string, outcome: Schema.DeliveryCheckOutcome, message: string | null }> }> } | null, deliveryAssessments: Array<{ environment: string, runtimeState: Schema.DeliveryEvidenceState, runningMatchesDeployment: boolean | null, candidateDeploymentId: string | null, candidateDigest: string | null, eligibility: Schema.DeliveryEligibility, checks: Array<{ name: string, outcome: Schema.DeliveryCheckOutcome, message: string | null }> }>, links: Array<{ siteName: string | null, httpUrl: string | null }> }> };


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
    query GetBlocks($scryIdentifier: String, $sample: String) {
  blocks(scryIdentifier: $scryIdentifier, sample: $sample) {
    manifestId
    delivery {
      kind
      tasks {
        collect {
          limit
          verifyAttestations
        }
        verify {
          limit
          verifyAttestations
        }
      }
      repository {
        host
        owner
        name
      }
      builds {
        workflow
        artifactNames
      }
      releases {
        includeDrafts
        includePrereleases
      }
      environments {
        name
        deployments {
          environment
          task
        }
        runtime {
          workflow
          artifactName
          schema
          maxAgeSeconds
        }
        compatibility {
          workflow
          artifactName
          schema
          maxAgeSeconds
        }
        rollback {
          requireArtifactAvailable
          requireVerifiedProvenance
          requireCompatibilityEvidence
        }
      }
    }
    deliverySnapshot {
      host
      repository
      observedAt
      sources {
        category
        environment
        state
        message
      }
      builds {
        id
        attempt
        workflow
        name
        commitSha
        status
        conclusion
        createdAt
        updatedAt
        url
        jobs {
          id
          name
          status
          conclusion
          url
        }
      }
      artifacts {
        id
        kind
        name
        digest
        runId
        releaseId
        expired
        url
        provenance
      }
      releases {
        id
        tag
        name
        publishedAt
        draft
        prerelease
        url
        assetIds
      }
      deployments {
        statuses {
          status
          createdAt
          url
        }
        id
        environment
        commitSha
        status
        createdAt
        updatedAt
        url
      }
      links {
        schema
        manifestId
        environment
        deploymentId
        buildRunId
        buildAttempt
        sourceRevision
        artifactDigest
        releaseId
        configurationRevision
      }
      runtime {
        schema
        manifestId
        environment
        deploymentId
        configurationRevision
        schemaRevision
        observedAt
        workloads {
          name
          artifactDigest
          replicas
          readyReplicas
        }
      }
      compatibility {
        schema
        manifestId
        environment
        candidateDigest
        currentSchemaRevision
        currentConfigurationRevision
        candidateConfigurationRevision
        suiteRevision
        observedAt
        checks {
          name
          outcome
          message
        }
      }
    }
    deliveryAssessments {
      environment
      runtimeState
      runningMatchesDeployment
      candidateDeploymentId
      candidateDigest
      eligibility
      checks {
        name
        outcome
        message
      }
    }
    name
    description
    version
    lineNumber
    consumerType
    icon
    languages
    frameworks
    deployments
    sourceCodeUrl
    connections
    docs
    owners
    tags
    metricTools
    cicdTool
    links {
      siteName
      httpUrl
    }
    rawJsonString
  }
}
    `);

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
