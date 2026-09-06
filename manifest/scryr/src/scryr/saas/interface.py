"""Interface type enumerations."""

from enum import Enum, StrEnum


class InterfaceCategory(StrEnum):
    """High-level interface families."""

    ui = "ui"
    api = "api"
    data_access = "data_access"


class InterfaceType(Enum):
    """Interface type enum."""

    # -------------------------- UI Interfaces --------------------------
    web_ui = (InterfaceCategory.ui, "web_ui", "HTML/JS rendered client")
    mobile_ui = (InterfaceCategory.ui, "mobile_ui", "Native mobile interface")
    desktop_ui = (InterfaceCategory.ui, "desktop_ui", "Electron or desktop app")

    # -------------------------- API Interfaces --------------------------
    rest = (InterfaceCategory.api, "rest", "RESTful HTTP API (OpenAPI/JSON)")
    graphql = (InterfaceCategory.api, "graphql", "GraphQL query and mutation interface")
    grpc = (InterfaceCategory.api, "grpc", "Binary RPC via gRPC/protobuf")
    websocket = (
        InterfaceCategory.api,
        "websocket",
        "Realtime bidirectional stream over WebSocket",
    )
    event = (
        InterfaceCategory.api,
        "event",
        "Event-driven pub/sub (Kafka, SNS, NATS, etc.)",
    )
    soap = (InterfaceCategory.api, "soap", "Legacy XML/SOAP-based interface")
    jsonrpc = (InterfaceCategory.api, "jsonrpc", "RPC-style interface over JSON")
    webhook = (InterfaceCategory.api, "webhook", "Callback-based push interface")

    # ----------------------- Data Access Interfaces -----------------------
    cli = (InterfaceCategory.ui, "cli", "Command-line interface")
    sdk = (InterfaceCategory.ui, "sdk", "Software development kit (client library)")
    sql = (
        InterfaceCategory.data_access,
        "sql",
        "Relational database access (SQL/ODBC/JDBC)",
    )
    nosql = (
        InterfaceCategory.data_access,
        "nosql",
        "Document or key-value data access",
    )
    graphql_datalayer = (
        InterfaceCategory.data_access,
        "graphql_datalayer",
        "GraphQL schema exposing data models",
    )
    orm = (
        InterfaceCategory.data_access,
        "orm",
        "ORM-based data layer (SQLAlchemy, Prisma, etc.)",
    )
    dataframe = (
        InterfaceCategory.data_access,
        "dataframe",
        "Analytical access via dataframe interface (Pandas, Arrow)",
    )
    object_store = (
        InterfaceCategory.data_access,
        "object_store",
        "Blob/object store interface (S3, GCS, etc.)",
    )
    stream = (
        InterfaceCategory.data_access,
        "stream",
        "Streaming data (Kafka, Kinesis, Pub/Sub)",
    )
    cache = (
        InterfaceCategory.data_access,
        "cache",
        "In-memory key-value data access (Redis, Memcached)",
    )
    search = (
        InterfaceCategory.data_access,
        "search",
        "Text or vector search interface (Elasticsearch, Meilisearch)",
    )
    file_system = (
        InterfaceCategory.data_access,
        "file_system",
        "File or distributed FS interface (NFS, Ceph, etc.)",
    )
    api_gateway = (
        InterfaceCategory.data_access,
        "api_gateway",
        "Data federation or API composition layer",
    )

    # --------------------------- Generic / Meta ---------------------------
    unknown = (
        InterfaceCategory.api,
        "unknown",
        "Unknown or unspecified interface type",
    )

    # --------------------------- Utility Properties ---------------------------
    @property
    def category(self) -> InterfaceCategory:
        """High-level interface domain (UI, API, or Data Access)."""
        return self.value[0]

    @property
    def id(self) -> str:
        """Short identifier used in configs or diagrams."""
        return self.value[1]

    @property
    def description(self) -> str:
        """Human-readable description of the interface."""
        return self.value[2]

    def __str__(self) -> str:
        """Return string representation."""
        return self.id
