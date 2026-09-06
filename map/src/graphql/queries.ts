/**
 * GraphQL queries and TypeScript types for the Scryr application
 */

// TypeScript types matching the GraphQL schema
export interface ScryrComponent {
	name?: string;
	description?: string;
	version?: string;
	icon?: string;
	language?: string;
	frameworks: string[];
	deployment?: string;
	sourceCodeUrl?: string;
	connections: string[];
	docs: string[];
	rawJsonString: string;
}

export interface MernComponentQueryResult {
	mernComponent: ScryrComponent;
}

export interface HealthCheckQueryResult {
	health: string;
}

// GraphQL query strings
export const GET_MERN_COMPONENT_QUERY = `
  query GetMernComponent {
    mernComponent {
      name
      description
      version
      icon
      language
      frameworks
      deployment
      sourceCodeUrl
      connections
      docs
      rawJsonString
    }
  }
`;

export const HEALTH_CHECK_QUERY = `
  query HealthCheck {
    health
  }
`;
