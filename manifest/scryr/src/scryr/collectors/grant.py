"""Expression-aware license policy evaluation over an inventory SBOM."""

from typing import Literal

from pydantic import Field

from .common import Identifier, LicensePolicy, SbomRef, Schedule, _CollectorConfig


class GrantLicenseCollector(_CollectorConfig):
    """Evaluate a typed policy using Grant and private generated configuration."""

    kind: Literal["grant_license"] = "grant_license"
    id: Identifier = "grant_license"
    sbom: SbomRef | None = None
    policy: LicensePolicy = Field(default_factory=LicensePolicy)
    schedule: Schedule = Field(
        default_factory=lambda: Schedule(startup=True, upstream_changed=True)
    )
