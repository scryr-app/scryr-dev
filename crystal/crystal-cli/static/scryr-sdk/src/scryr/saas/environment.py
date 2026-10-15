"""Authentication enumerations."""

from enum import StrEnum


class AuthType(StrEnum):
    """Authentication method types."""

    none = "none"
    api_key = "api_key"
    oauth2 = "oauth2"
    jwt = "jwt"
    basic = "basic"
    mutual_tls = "mutual_tls"
    service_account = "service_account"
