"""Classification enumerations."""

from enum import StrEnum


class Classification(StrEnum):
    """Architectural roles and interfaces used for architecture visualization."""

    # APIs & Gateways
    public_api = "public_api"
    internal_api = "internal_api"
    network_gateway = "network_router"

    # User Interfaces
    public_ui = "public_ui"
    mobile_app = "mobile_app"
    admin_ui = "admin_ui"
    cli = "cli"
    sdk = "sdk"

    # Data Storage
    datastore = "datastore"
    database = "database"
    object_storage = "object_storage"
    document_store = "document_store"
    vector_store = "vector_store"
    columnar_store = "columnar_store"

    # Caching & Search
    cache_store = "cache_store"
    search_store = "search_store"

    # Messaging & Events
    queue = "queue"
    source = "source"
    sink = "sink"

    # Background & Async
    worker = "worker"
    scheduler = "scheduler"
    job_processor = "job_processor"

    # Interfaces and protocols
    web_ui = "web_ui"
    mobile_ui = "mobile_ui"
    desktop_ui = "desktop_ui"
    rest = "rest"
    graphql = "graphql"
    grpc = "grpc"
    websocket = "websocket"
    event = "event"
    soap = "soap"
    jsonrpc = "jsonrpc"
    webhook = "webhook"
    sql = "sql"
    nosql = "nosql"
    graphql_datalayer = "graphql_datalayer"
    orm = "orm"
    dataframe = "dataframe"
    object_store = "object_store"
    stream = "stream"
    cache = "cache"
    search = "search"
    file_system = "file_system"
    api_gateway = "api_gateway"
    unknown = "unknown"
