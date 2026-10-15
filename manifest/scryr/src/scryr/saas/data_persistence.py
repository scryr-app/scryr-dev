"""Infrastructure as Code tooling enumerations."""

from enum import StrEnum


class IaCToolType(StrEnum):
    """Infrastructure as Code tooling."""

    terraform = "terraform"
    cloudformation = "cloudformation"
    arm_template = "arm_template"
    pulumi = "pulumi"
    cdktf = "cdktf"
    helm = "helm"
    kustomize = "kustomize"
    none = "none"
