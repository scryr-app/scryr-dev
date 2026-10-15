"""CI/CD tooling enumerations."""

from enum import StrEnum


class CICDToolType(StrEnum):
    """CI/CD pipeline platforms."""

    github_actions = "github_actions"
    gitlab_ci = "gitlab_ci"
    circleci = "circleci"
    jenkins = "jenkins"
    travis_ci = "travis_ci"
    bitbucket_pipelines = "bitbucket_pipelines"
    azure_pipelines = "azure_pipelines"
    teamcity = "teamcity"
    buildkite = "buildkite"
    drone = "drone"
    argo_workflows = "argo_workflows"
    tekton = "tekton"
    concourse = "concourse"
    none = "none"
