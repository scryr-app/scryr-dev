"""Vulnerability matching against an explicitly linked inventory artifact."""

from datetime import timedelta
from typing import Literal

from pydantic import Field

from .common import Identifier, SbomRef, Schedule, _CollectorConfig


class GrypeScanCollector(_CollectorConfig):
    """Match known advisories with Grype without creating a second inventory."""

    kind: Literal["grype_scan"] = "grype_scan"
    id: Identifier = "grype_scan"
    sbom: SbomRef | None = None
    schedule: Schedule = Field(
        default_factory=lambda: Schedule(
            startup=True, upstream_changed=True, every=timedelta(days=1)
        )
    )
