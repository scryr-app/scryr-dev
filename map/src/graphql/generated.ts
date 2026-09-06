/** Internal type. DO NOT USE DIRECTLY. */
type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
/** Internal type. DO NOT USE DIRECTLY. */
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
import { useQuery, type UseQueryOptions } from '@tanstack/react-query';
import type { DocumentTypeDecoration } from '@graphql-typed-document-node/core';
import { graphqlFetcher } from './client';

export class TypedDocumentString<TResult, TVariables>
  extends String
  implements DocumentTypeDecoration<TResult, TVariables>
{
  __apiType?: NonNullable<DocumentTypeDecoration<TResult, TVariables>['__apiType']>;
  private value: string;
  public __meta__?: Record<string, unknown> | undefined;

  constructor(value: string, __meta__?: Record<string, unknown> | undefined) {
    super(value);
    this.value = value;
    this.__meta__ = __meta__;
  }

  override toString(): string & DocumentTypeDecoration<TResult, TVariables> {
    return this.value as string & DocumentTypeDecoration<TResult, TVariables>;
  }
}

export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  DateTime: { input: unknown; output: unknown; }
  UUID: { input: unknown; output: unknown; }
};

/** Logical kind for a generated artifact persisted to Postgres. */
export enum ArtifactKind {
  /** Generated schema or type-definition output. */
  Schema = 'schema',
  /** Generated instance/value output. */
  Value = 'value'
}

export type Block = {
  __typename?: 'Block';
  /** Primary authentication mechanism. */
  authType?: Maybe<Scalars['String']['output']>;
  /** CI/CD pipeline platform (`github_actions`, `jenkins`, `circleci`, etc.). */
  cicdTool?: Maybe<Scalars['String']['output']>;
  /** Named connections to other components in the graph. */
  connections: Array<Scalars['String']['output']>;
  /** Service classification (e.g. `public_api`, `worker`, `database`). */
  consumerType?: Maybe<Scalars['String']['output']>;
  /** Primary deployment target identifier. */
  deployment?: Maybe<Scalars['String']['output']>;
  /** Cloud provider extracted from the deployment tuple. */
  deploymentProvider?: Maybe<Scalars['String']['output']>;
  /** Short description of the component. */
  description?: Maybe<Scalars['String']['output']>;
  /** Documentation URLs. */
  docs: Array<Scalars['String']['output']>;
  /** Web frameworks in use. */
  frameworks: Array<Scalars['String']['output']>;
  /** Infrastructure as Code tooling (terraform, pulumi, cdk, etc.). */
  iacTool?: Maybe<Scalars['String']['output']>;
  /** Emoji or small icon string for the component. */
  icon?: Maybe<Scalars['String']['output']>;
  /** Primary programming language. */
  language?: Maybe<Scalars['String']['output']>;
  /** 1-based line number of the manifest assignment in the source file. */
  lineNumber?: Maybe<Scalars['Int']['output']>;
  /** Helpful external links with display name and URL. */
  links: Array<Link>;
  /** Log aggregation and analysis platform. */
  logAggregation?: Maybe<Scalars['String']['output']>;
  /** Maximum number of service replicas. */
  maxReplicas?: Maybe<Scalars['Int']['output']>;
  /** Minimum number of service replicas. */
  minReplicas?: Maybe<Scalars['Int']['output']>;
  /** Monitoring and metrics platform. */
  monitoring?: Maybe<Scalars['String']['output']>;
  /** Display name for the component (used as the block label). */
  name?: Maybe<Scalars['String']['output']>;
  /** Team responsible for this service. */
  ownerTeam?: Maybe<Scalars['String']['output']>;
  /** Returns the raw JSON as a string for debugging. */
  rawJsonString: Scalars['String']['output'];
  /** Repository or source URL (also used by `GithubCard`). */
  sourceCodeUrl?: Maybe<Scalars['String']['output']>;
  /** Arbitrary tags for categorization and filtering. */
  tags: Array<Scalars['String']['output']>;
  /** Distributed tracing and APM platform. */
  tracing?: Maybe<Scalars['String']['output']>;
  /** Semver-like version string (e.g. "1.2.3"). */
  version?: Maybe<Scalars['String']['output']>;
};

export type CreatePostInput = {
  authorId: Scalars['UUID']['input'];
  body: Scalars['String']['input'];
  title: Scalars['String']['input'];
  url?: InputMaybe<Scalars['String']['input']>;
};

export type CreatePostPayload = {
  __typename?: 'CreatePostPayload';
  post?: Maybe<Post>;
};

export type CreateUserInput = {
  displayName?: InputMaybe<Scalars['String']['input']>;
  username: Scalars['String']['input'];
};

export type CreateUserPayload = {
  __typename?: 'CreateUserPayload';
  user?: Maybe<User>;
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

/** Root mutation type for GraphQL schema. */
export type MutationRoot = {
  __typename?: 'MutationRoot';
  createPost: CreatePostPayload;
  createUser: CreateUserPayload;
  /** Upsert a generated manifest artifact into Postgres. */
  upsertGeneratedManifest: UpsertGeneratedManifestPayload;
};


/** Root mutation type for GraphQL schema. */
export type MutationRootCreatePostArgs = {
  input: CreatePostInput;
};


/** Root mutation type for GraphQL schema. */
export type MutationRootCreateUserArgs = {
  input: CreateUserInput;
};


/** Root mutation type for GraphQL schema. */
export type MutationRootUpsertGeneratedManifestArgs = {
  input: UpsertGeneratedManifestInput;
};

/** Pagination information for a connection. */
export type PageInfo = {
  __typename?: 'PageInfo';
  /** Cursor for the last edge in this page. */
  endCursor?: Maybe<Scalars['String']['output']>;
  /** Whether there are more pages after the current page. */
  hasNextPage: Scalars['Boolean']['output'];
  /** Whether there are pages before the current page. */
  hasPreviousPage: Scalars['Boolean']['output'];
  /** Cursor for the first edge in this page. */
  startCursor?: Maybe<Scalars['String']['output']>;
};

/** Represents a post in the system. */
export type Post = {
  __typename?: 'Post';
  /** Author of the post. */
  author: User;
  /** Post content. */
  body: Scalars['String']['output'];
  /** Timestamp when the post was created. */
  createdAt: Scalars['DateTime']['output'];
  /** Unique identifier for the post. */
  id: Scalars['UUID']['output'];
  /** Post title. */
  title: Scalars['String']['output'];
  /** Optional URL associated with the post. */
  url?: Maybe<Scalars['String']['output']>;
};

export type QueryRoot = {
  __typename?: 'QueryRoot';
  /**
   * Loads and returns blocks from generated manifest artifacts in Postgres.
   * Pass `sample` to override which sample is served (e.g. `"calcom"`).
   */
  blocks: Array<Block>;
  health: HealthStatus;
  node?: Maybe<User>;
  posts: Array<Post>;
  /** Lists persisted Scryr maps available in Postgres. */
  scryrMaps: Array<ScryrMap>;
  users: UserConnection;
};


export type QueryRootBlocksArgs = {
  sample?: InputMaybe<Scalars['String']['input']>;
  scryIdentifier?: InputMaybe<Scalars['String']['input']>;
};


export type QueryRootNodeArgs = {
  id: Scalars['UUID']['input'];
};


export type QueryRootPostsArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRootUsersArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
  order?: InputMaybe<SortOrder>;
};

/** Sort order for query results. */
export enum SortOrder {
  /** Ascending order. */
  Asc = 'ASC',
  /** Descending order. */
  Desc = 'DESC'
}

/** A persisted Scryr map artifact available in Postgres. */
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
  /** Persisted row identifier. */
  id: Scalars['String']['output'];
  /** Artifact key used by legacy sample-based lookups. */
  key: Scalars['String']['output'];
  /** Human-readable map name from the declared Diagram. */
  name: Scalars['String']['output'];
  /** Clerk organization slug cached from the uploader's active session. */
  orgSlug?: Maybe<Scalars['String']['output']>;
  /** Stable Scryr identifier, normally the top-level Diagram variable name. */
  scryIdentifier: Scalars['String']['output'];
  /** Last update timestamp rendered by Postgres. */
  updatedAt: Scalars['String']['output'];
};

export type SubscriptionRoot = {
  __typename?: 'SubscriptionRoot';
  postCreated: Post;
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

/** Represents a user in the system. */
export type User = {
  __typename?: 'User';
  /** User's display name. */
  displayName?: Maybe<Scalars['String']['output']>;
  /** Unique identifier for the user. */
  id: Scalars['UUID']['output'];
  /** Timestamp when the user joined. */
  joinedAt: Scalars['DateTime']['output'];
  /** User's login name. */
  username: Scalars['String']['output'];
};

/** Connection of users with pagination support. */
export type UserConnection = {
  __typename?: 'UserConnection';
  /** List of user edges. */
  edges: Array<UserEdge>;
  /** Pagination metadata. */
  pageInfo: PageInfo;
};

/** Edge in a user connection, containing a user and cursor. */
export type UserEdge = {
  __typename?: 'UserEdge';
  /** Cursor for pagination. */
  cursor: Scalars['String']['output'];
  /** The user data. */
  node: User;
};

export type HealthCheckQueryVariables = Exact<{ [key: string]: never; }>;


export type HealthCheckQuery = { health: { status: string, databaseOk: boolean, databaseMessage: string } };

export type GetScryrMapsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetScryrMapsQuery = { scryrMaps: Array<{ id: string, key: string, scryIdentifier: string, name: string, clerkOrgId: string, orgSlug: string | null, folderPath: string, fileName: string, gitCommitSha: string | null, updatedAt: string }> };

export type GetBlocksQueryVariables = Exact<{
  sample?: string | null | undefined;
  scryIdentifier?: string | null | undefined;
}>;


export type GetBlocksQuery = { blocks: Array<{ name: string | null, description: string | null, version: string | null, lineNumber: number | null, consumerType: string | null, icon: string | null, language: string | null, frameworks: Array<string>, deployment: string | null, deploymentProvider: string | null, sourceCodeUrl: string | null, connections: Array<string>, docs: Array<string>, ownerTeam: string | null, tags: Array<string>, authType: string | null, monitoring: string | null, logAggregation: string | null, tracing: string | null, iacTool: string | null, cicdTool: string | null, maxReplicas: number | null, minReplicas: number | null, rawJsonString: string, links: Array<{ siteName: string | null, httpUrl: string | null }> }> };



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
    sourceCodeUrl
    connections
    docs
    ownerTeam
    tags
    authType
    monitoring
    logAggregation
    tracing
    iacTool
    cicdTool
    maxReplicas
    minReplicas
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
