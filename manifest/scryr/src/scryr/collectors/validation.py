"""Validate stable identities and SBOM references without inspecting local files."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .grant import GrantLicenseCollector
from .grype import GrypeScanCollector
from .syft import SyftInventoryCollector

if TYPE_CHECKING:
    from collections.abc import Sequence

    from scryr.manifest import Manifest

EVIDENCE_SECTIONS = ("repository", "checks", "metrics", "tests", "dependencies", "performance")


def validate_manifest_collectors(manifest: Manifest) -> None:
    """Check per-section identity and same-manifest inventory references."""
    for section in EVIDENCE_SECTIONS:
        collectors = getattr(manifest, section)
        if collectors and not manifest.manifest_id:
            msg = "Evidence collectors require an explicit manifest_id"
            raise ValueError(msg)
        ids = [collector.id for collector in collectors]
        if len(ids) != len(set(ids)):
            msg = f"Duplicate collector IDs in {section}; assign explicit unique IDs"
            raise ValueError(msg)
    _validate_inventory_refs(manifest, {manifest.manifest_id: manifest}, local_only=True)


def _validate_inventory_refs(
    manifest: Manifest, pool: dict[str | None, Manifest], *, local_only: bool = False
) -> None:
    for collector in manifest.dependencies:
        if not isinstance(collector, (GrantLicenseCollector, GrypeScanCollector)):
            continue
        ref = collector.sbom
        target_id = ref.manifest_id if ref and ref.manifest_id else manifest.manifest_id
        if local_only and target_id != manifest.manifest_id:
            continue
        target = pool.get(target_id)
        if target is None:
            msg = f"SBOM reference names missing manifest {target_id!r}"
            raise ValueError(msg)
        inventories = [c for c in target.dependencies if isinstance(c, SyftInventoryCollector)]
        if ref:
            inventories = [c for c in inventories if c.id == ref.collector_id]
        if len(inventories) != 1:
            msg = f"{collector.id}: SBOM reference must resolve to exactly one Syft inventory"
            raise ValueError(msg)


def validate_collector_references(manifests: Sequence[Manifest]) -> None:
    """Validate one complete source envelope, including cross-manifest references.

    Only Syft produces an SBOM; only Grant/Grype consume one. Restricting the
    reference target to Syft makes cycles impossible, including cross-manifest
    cycles. A scanner can never masquerade as an inventory producer.
    """
    pool: dict[str | None, Manifest] = {}
    for manifest in manifests:
        validate_manifest_collectors(manifest)
        if not manifest.manifest_id:
            continue
        previous = pool.get(manifest.manifest_id)
        if previous is not None and previous is not manifest:
            msg = f"Duplicate manifest_id: {manifest.manifest_id}"
            raise ValueError(msg)
        pool[manifest.manifest_id] = manifest
    for manifest in manifests:
        _validate_inventory_refs(manifest, pool)
