/** Internal type. DO NOT USE DIRECTLY. */
type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
/** Internal type. DO NOT USE DIRECTLY. */
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
import { graphqlFetcher } from './client';
import type { DocumentTypeDecoration } from '@graphql-typed-document-node/core';
import { useQuery, type UseQueryOptions } from '@tanstack/react-query';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
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

/** Logical kind for a generated artifact persisted to storage. */
export enum ArtifactKind {
  /** Generated schema or type-definition output. */
  Schema = 'schema',
  /** Generated instance/value output. */
  Value = 'value'
}

export type Block = {
  __typename?: 'Block';
  /** Named connections to other components in the graph. */
  connections: Array<Scalars['String']['output']>;
  /** Service classification (e.g. `public_api`, `worker`, `database`). */
  consumerType?: Maybe<Scalars['String']['output']>;
  /** Emoji or small icon string for the component. */
  icon?: Maybe<Scalars['String']['output']>;
  /** 1-based line number of the manifest assignment in the source file. */
  lineNumber?: Maybe<Scalars['Int']['output']>;
  /** Stable identity for the architecture component. */
  manifestId?: Maybe<Scalars['String']['output']>;
  /** Display name for the component (used as the block label). */
  name?: Maybe<Scalars['String']['output']>;
  /** Returns the raw JSON as a string for debugging. */
  rawJsonString: Scalars['String']['output'];
  /** Arbitrary tags for categorization and filtering. */
  tags: Array<Scalars['String']['output']>;
};

export type GeneratedManifestMutationRoot = {
  __typename?: 'GeneratedManifestMutationRoot';
  /** Save source and all its diagram artifacts with optimistic concurrency. */
  saveManifestDocument: Scalars['JSON']['output'];
  /** Upsert a generated manifest artifact into storage. */
  upsertGeneratedManifest: UpsertGeneratedManifestPayload;
};


export type GeneratedManifestMutationRootSaveManifestDocumentArgs = {
  envelope: Scalars['JSON']['input'];
  identifier: Scalars['String']['input'];
  revision: Scalars['String']['input'];
};


export type GeneratedManifestMutationRootUpsertGeneratedManifestArgs = {
  input: UpsertGeneratedManifestInput;
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

export type QueryRoot = {
  __typename?: 'QueryRoot';
  /**
   * Loads and returns blocks from generated manifest artifacts in the configured database.
   * Pass `scry_identifier` to load an exact Scryr map, or `sample` for legacy lookup.
   */
  blocks: Array<Block>;
  health: HealthStatus;
  /** Read the selected diagram's complete source document. */
  manifestDocument: Scalars['JSON']['output'];
  /** Lists persisted Scryr maps available in the configured database. */
  scryrMaps: Array<ScryrMap>;
};


export type QueryRootBlocksArgs = {
  sample?: InputMaybe<Scalars['String']['input']>;
  scryIdentifier?: InputMaybe<Scalars['String']['input']>;
};


export type QueryRootManifestDocumentArgs = {
  identifier: Scalars['String']['input'];
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

export type HealthCheckQueryVariables = Exact<{ [key: string]: never; }>;


export type HealthCheckQuery = { health: { status: string, databaseOk: boolean, databaseMessage: string } };

export type GetScryrMapsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetScryrMapsQuery = { scryrMaps: Array<{ id: string, key: string, scryIdentifier: string, name: string, clerkOrgId: string, orgSlug: string | null, folderPath: string, fileName: string, gitCommitSha: string | null, updatedAt: string }> };

export type GetBlocksQueryVariables = Exact<{
  scryIdentifier?: string | null | undefined;
  sample?: string | null | undefined;
}>;


export type GetBlocksQuery = { blocks: Array<{ manifestId: string | null, name: string | null, lineNumber: number | null, consumerType: string | null, icon: string | null, connections: Array<string>, tags: Array<string>, rawJsonString: string }> };


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
    name
    lineNumber
    consumerType
    icon
    connections
    tags
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
