"""Interface type enumerations."""

from enum import StrEnum

from .classification import Classification


class InterfaceCategory(StrEnum):
    """High-level interface families."""

    ui = "ui"
    api = "api"
    data_access = "data_access"


# Backward-compatible import; new manifests should use Classification.
InterfaceType = Classification
