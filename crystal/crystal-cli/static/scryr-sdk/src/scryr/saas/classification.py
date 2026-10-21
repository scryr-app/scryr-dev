"""Classification enumerations."""

from enum import StrEnum


class Classification(StrEnum):
    """Architectural roles and interfaces used for architecture visualization."""

    # APIs & Gateways
    public_api = "public_api"
    internal_api = "internal_api"
    network_gateway = "network_router"

    # Traffic & Networking
    load_balancer = "load_balancer"
    reverse_proxy = "reverse_proxy"
    cdn = "cdn"
    dns = "dns"

    # Security & Identity
    firewall = "firewall"
    identity_provider = "identity_provider"
    secrets_manager = "secrets_manager"

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
    data_pipeline = "data_pipeline"
    workflow_engine = "workflow_engine"

    # Observability
    monitoring = "monitoring"
    log_collector = "log_collector"

    # Machine Learning
    model_service = "model_service"

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
