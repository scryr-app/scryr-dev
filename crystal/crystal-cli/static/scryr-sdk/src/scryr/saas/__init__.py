"""SaaS-related enumerations and types.

This module contains deployment targets, programming languages,
web frameworks, and interface types.
"""

from .classification import Classification
from .deployment_target import CloudProvider, DeploymentTarget, RuntimeKind
from .interface import InterfaceCategory, InterfaceType
from .programming_languages import ProgrammingLanguage
from .web_frameworks import WebFramework

__all__ = [
    "Classification",
    "CloudProvider",
    "DeploymentTarget",
    "InterfaceCategory",
    "InterfaceType",
    "ProgrammingLanguage",
    "RuntimeKind",
    "WebFramework",
]
