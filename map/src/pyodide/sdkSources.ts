import githubShimSource from "./githubShim.py?raw";
import sampleMernSource from "./index.scry?raw";

const cicdToolSource = `
"""CI/CD tooling enumerations."""

from enum import StrEnum


class CICDToolType(StrEnum):
    """CI/CD pipeline platforms."""

    github_actions = "github_actions"
    gitlab_ci = "gitlab_ci"
    circleci = "circleci"
    jenkins = "jenkins"
    travis_ci = "travis_ci"
    bitbucket_pipelines = "bitbucket_pipelines"
    azure_pipelines = "azure_pipelines"
    teamcity = "teamcity"
    buildkite = "buildkite"
    drone = "drone"
    argo_workflows = "argo_workflows"
    tekton = "tekton"
    concourse = "concourse"
    none = "none"
`.trim();

const dataPersistenceSource = `
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
`.trim();

const deploymentTargetSource = `
"""Deployment target enumerations for cloud providers and runtime kinds."""

from enum import Enum, StrEnum


class CloudProvider(StrEnum):
    """Cloud provider enum."""

    aws = "aws"
    azure = "azure"
    gcp = "gcp"
    kubernetes = "kubernetes"
    metal = "metal"
    serverless = "serverless"


class RuntimeKind(StrEnum):
    """Runtime kind enum."""

    container = "container"
    function = "function"
    process = "process"
    vm = "vm"
    wasm = "wasm"
    edge_worker = "edge_worker"
    static = "static"  # e.g. frontend assets
    batch = "batch"
    interactive = "interactive"  # REPL, console, etc.


class DeploymentTarget(Enum):
    """Deployment target enum combining provider and service."""

    # ----------------------- AWS -----------------------
    ecs = (CloudProvider.aws, "ecs")
    fargate = (CloudProvider.aws, "fargate")
    ec2 = (CloudProvider.aws, "ec2")
    elastic_beanstalk = (CloudProvider.aws, "elastic_beanstalk")
    lightsail = (CloudProvider.aws, "lightsail")
    amplify = (CloudProvider.aws, "amplify")
    cloudfront = (CloudProvider.aws, "cloudfront")
    sagemaker = (CloudProvider.aws, "sagemaker")
    batch = (CloudProvider.aws, "batch")
    ecr = (CloudProvider.aws, "ecr")
    eks = (CloudProvider.aws, "eks")

    # ----------------------- Azure -----------------------
    app_service = (CloudProvider.azure, "app_service")
    aks = (CloudProvider.azure, "aks")
    container_instances = (CloudProvider.azure, "container_instances")
    vm = (CloudProvider.azure, "vm")
    static_web_apps = (CloudProvider.azure, "static_web_apps")
    service_fabric = (CloudProvider.azure, "service_fabric")
    batch_azure = (CloudProvider.azure, "batch")

    # ----------------------- GCP -----------------------
    cloud_run = (CloudProvider.gcp, "cloud_run")
    app_engine = (CloudProvider.gcp, "app_engine")
    compute_engine = (CloudProvider.gcp, "compute_engine")
    gke = (CloudProvider.gcp, "gke")
    cloud_storage = (CloudProvider.gcp, "cloud_storage")
    firebase = (CloudProvider.gcp, "firebase")
    dataflow = (CloudProvider.gcp, "dataflow")

    # ----------------------- Kubernetes -----------------------
    k8s_cluster = (CloudProvider.kubernetes, "k8s_cluster")
    helm = (CloudProvider.kubernetes, "helm")
    argo = (CloudProvider.kubernetes, "argo")
    istio = (CloudProvider.kubernetes, "istio")
    knative = (CloudProvider.kubernetes, "knative")
    openshift = (CloudProvider.kubernetes, "openshift")
    rancher = (CloudProvider.kubernetes, "rancher")

    # ----------------------- Metal / On-Prem -----------------------
    bare_metal = (CloudProvider.metal, "bare_metal")
    vmware = (CloudProvider.metal, "vmware")
    proxmox = (CloudProvider.metal, "proxmox")
    openstack = (CloudProvider.metal, "openstack")
    docker_swarm = (CloudProvider.metal, "docker_swarm")
    nomad = (CloudProvider.metal, "nomad")
    render = (CloudProvider.metal, "render")
    fly_io = (CloudProvider.metal, "fly_io")
    heroku = (CloudProvider.metal, "heroku")
    digitalocean_app = (CloudProvider.metal, "digitalocean_app")
    vultr = (CloudProvider.metal, "vultr")
    linode = (CloudProvider.metal, "linode")

    # ----------------------- Serverless -----------------------
    lambda_ = (CloudProvider.serverless, "lambda")
    cloud_functions = (CloudProvider.serverless, "cloud_functions")
    functions = (CloudProvider.serverless, "functions")
    vercel = (CloudProvider.serverless, "vercel")
    netlify = (CloudProvider.serverless, "netlify")
    cloudflare_pages = (CloudProvider.serverless, "cloudflare_pages")
    cloudflare_workers = (CloudProvider.serverless, "cloudflare_workers")
    railway = (CloudProvider.serverless, "railway")
    amplify_serverless = (CloudProvider.serverless, "amplify_serverless")

    unknown = (CloudProvider.metal, "unknown")

    @property
    def provider(self) -> CloudProvider:
        """Get the cloud provider."""
        return self.value[0]

    @property
    def id(self) -> str:
        """Get the deployment target ID."""
        return self.value[1]

    def __str__(self) -> str:
        """Return string representation."""
        return self.id
`.trim();

const environmentSource = `
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
`.trim();

const interfaceSource = `
"""Interface type enumerations."""

from enum import Enum, StrEnum


class InterfaceCategory(StrEnum):
    """High-level interface families."""

    ui = "ui"
    api = "api"
    data_access = "data_access"


class InterfaceType(Enum):
    """Interface type enum."""

    web_ui = (InterfaceCategory.ui, "web_ui", "HTML/JS rendered client")
    mobile_ui = (InterfaceCategory.ui, "mobile_ui", "Native mobile interface")
    desktop_ui = (InterfaceCategory.ui, "desktop_ui", "Electron or desktop app")
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
    unknown = (
        InterfaceCategory.api,
        "unknown",
        "Unknown or unspecified interface type",
    )

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
`.trim();

const programmingLanguagesSource = `
"""Programming language enumerations."""

from enum import StrEnum


class ProgrammingLanguage(StrEnum):
    """Programming language enum."""

    ada = "ada"
    apex = "apex"
    assembly = "assembly"
    awk = "awk"
    bash = "bash"
    basic = "basic"
    c = "c"
    csharp = "csharp"
    cpp = "cpp"
    clojure = "clojure"
    cobol = "cobol"
    crystal = "crystal"
    cuda = "cuda"
    dart = "dart"
    delphi = "delphi"
    dlang = "d"
    elixir = "elixir"
    elm = "elm"
    erlang = "erlang"
    fsharp = "fsharp"
    fortran = "fortran"
    go = "go"
    groovy = "groovy"
    haskell = "haskell"
    html = "html"
    java = "java"
    javascript = "javascript"
    julia = "julia"
    kotlin = "kotlin"
    latex = "latex"
    lisp = "lisp"
    lua = "lua"
    matlab = "matlab"
    nim = "nim"
    nix = "nix"
    objective_c = "objective_c"
    ocaml = "ocaml"
    perl = "perl"
    php = "php"
    powershell = "powershell"
    protobuf = "protobuf"
    python = "python"
    r = "r"
    racket = "racket"
    ruby = "ruby"
    rust = "rust"
    scala = "scala"
    shell = "shell"
    solidity = "solidity"
    sql = "sql"
    swift = "swift"
    typescript = "typescript"
    terraform = "terraform"
    tcl = "tcl"
    v = "v"
    vbnet = "vbnet"
    verilog = "verilog"
    vhdl = "vhdl"
    visual_basic = "visual_basic"
    vue = "vue"
    wasm = "wasm"
    yaml = "yaml"
    zig = "zig"
`.trim();

const telemetrySource = `
"""Telemetry and observability platform enumerations."""

from enum import StrEnum


class MonitoringType(StrEnum):
    """Observability and monitoring platforms."""

    prometheus = "prometheus"
    datadog = "datadog"
    new_relic = "new_relic"
    elastic = "elastic"
    splunk = "splunk"
    cloudwatch = "cloudwatch"
    stackdriver = "stackdriver"
    dynatrace = "dynatrace"
    grafana = "grafana"
    sumologic = "sumologic"
    none = "none"


class LogAggregationType(StrEnum):
    """Log aggregation and analysis platforms."""

    elk_stack = "elk_stack"
    splunk = "splunk"
    datadog = "datadog"
    cloudwatch = "cloudwatch"
    stackdriver = "stackdriver"
    papertrail = "papertrail"
    loggly = "loggly"
    sumo_logic = "sumo_logic"
    loki = "loki"
    none = "none"


class TracingType(StrEnum):
    """Distributed tracing and APM platforms."""

    jaeger = "jaeger"
    zipkin = "zipkin"
    datadog = "datadog"
    new_relic = "new_relic"
    elastic_apm = "elastic_apm"
    dynatrace = "dynatrace"
    aws_xray = "aws_xray"
    honeycomb = "honeycomb"
    none = "none"
`.trim();

const versionSource = `
"""Typed version models for SemVer, Calendar, and Incremental versioning."""

from __future__ import annotations

import re
from dataclasses import dataclass
from functools import total_ordering
from typing import Final, overload

SEMVER_RE: Final[re.Pattern[str]] = re.compile(
    r"^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)"
    r"(?:-([0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*))?"
    r"(?:\\+([0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*))?$"
)
CALENDAR_RE: Final[re.Pattern[str]] = re.compile(r"^(\\d{4})\\.(0?[1-9]|1[0-2])$")
INCREMENTAL_RE: Final[re.Pattern[str]] = re.compile(r"^(0|[1-9]\\d*)$")


@dataclass(frozen=True, slots=True, init=False)
@total_ordering
class SemVer:
    """Semantic version with proper precedence semantics.

    Build metadata is preserved in the value but ignored for ordering.
    """

    major: int
    minor: int
    patch: int
    prerelease: tuple[str, ...] = ()
    build: tuple[str, ...] = ()

    @overload
    def __init__(self, value: str, /) -> None: ...

    @overload
    def __init__(
        self,
        major: int,
        minor: int,
        patch: int,
        prerelease: tuple[str, ...] = (),
        build: tuple[str, ...] = (),
    ) -> None: ...

    def __init__(
        self,
        major: str | int,
        minor: int | None = None,
        patch: int | None = None,
        prerelease: tuple[str, ...] = (),
        build: tuple[str, ...] = (),
    ) -> None:
        """Create SemVer from either a full string or numeric components."""
        if isinstance(major, str):
            if minor is not None or patch is not None or prerelease or build:
                msg = "String SemVer input does not accept additional components"
                raise TypeError(msg)
            parsed = self.parse(major)
            object.__setattr__(self, "major", parsed.major)
            object.__setattr__(self, "minor", parsed.minor)
            object.__setattr__(self, "patch", parsed.patch)
            object.__setattr__(self, "prerelease", parsed.prerelease)
            object.__setattr__(self, "build", parsed.build)
            return

        if minor is None or patch is None:
            msg = "SemVer numeric input requires major, minor, and patch"
            raise TypeError(msg)

        object.__setattr__(self, "major", major)
        object.__setattr__(self, "minor", minor)
        object.__setattr__(self, "patch", patch)
        object.__setattr__(self, "prerelease", prerelease)
        object.__setattr__(self, "build", build)

    @classmethod
    def parse(cls, value: str) -> SemVer:
        """Parse and validate a semantic version string."""
        normalized = _strip_optional_v_prefix(value)
        match = SEMVER_RE.fullmatch(normalized)
        if match is None:
            msg = (
                "Invalid SemVer. Expected MAJOR.MINOR.PATCH (optionally prefixed with 'v') "
                "with optional "
                "-PRERELEASE and +BUILD metadata"
            )
            raise ValueError(msg)

        major = int(match.group(1))
        minor = int(match.group(2))
        patch = int(match.group(3))

        prerelease_raw = match.group(4)
        build_raw = match.group(5)
        prerelease = tuple(prerelease_raw.split(".")) if prerelease_raw else ()
        build = tuple(build_raw.split(".")) if build_raw else ()

        for identifier in prerelease:
            if identifier.isdigit() and len(identifier) > 1 and identifier.startswith("0"):
                msg = "Numeric prerelease identifiers must not contain leading zeros"
                raise ValueError(msg)

        return cls(major, minor, patch, prerelease, build)

    def __str__(self) -> str:
        """Return the canonical string form."""
        base = f"{self.major}.{self.minor}.{self.patch}"
        prerelease = f"-{'.'.join(self.prerelease)}" if self.prerelease else ""
        build = f"+{'.'.join(self.build)}" if self.build else ""
        return f"{base}{prerelease}{build}"

    def __lt__(self, other: object) -> bool:
        """Compare semantic versions using SemVer precedence rules."""
        if not isinstance(other, SemVer):
            return NotImplemented

        core_self = (self.major, self.minor, self.patch)
        core_other = (other.major, other.minor, other.patch)
        if core_self != core_other:
            return core_self < core_other

        if not self.prerelease and other.prerelease:
            return False
        if self.prerelease and not other.prerelease:
            return True
        if not self.prerelease and not other.prerelease:
            return False

        return _compare_prerelease(self.prerelease, other.prerelease) < 0

    def __eq__(self, other: object) -> bool:
        """SemVer equality by precedence (build metadata does not affect equality)."""
        if not isinstance(other, SemVer):
            return False
        return (
            self.major,
            self.minor,
            self.patch,
            self.prerelease,
        ) == (
            other.major,
            other.minor,
            other.patch,
            other.prerelease,
        )

    def __hash__(self) -> int:
        """Hash using fields involved in equality/precedence semantics."""
        return hash((self.major, self.minor, self.patch, self.prerelease))


@dataclass(frozen=True, slots=True, init=False)
class CalendarVersion:
    """Calendar version in YEAR.MONTH format."""

    year: int
    month: int

    @overload
    def __init__(self, value: str, /) -> None: ...

    @overload
    def __init__(self, year: int, month: int) -> None: ...

    def __init__(self, year: str | int, month: int | None = None) -> None:
        """Create calendar version from either YEAR.MONTH or integer components."""
        if isinstance(year, str):
            if month is not None:
                msg = "String calendar input does not accept a separate month"
                raise TypeError(msg)
            parsed = self.parse(year)
            object.__setattr__(self, "year", parsed.year)
            object.__setattr__(self, "month", parsed.month)
            return

        if month is None:
            msg = "Calendar numeric input requires both year and month"
            raise TypeError(msg)

        object.__setattr__(self, "year", year)
        object.__setattr__(self, "month", month)

    @classmethod
    def parse(cls, value: str) -> CalendarVersion:
        """Parse and validate a calendar version string."""
        normalized = _strip_optional_v_prefix(value)
        match = CALENDAR_RE.fullmatch(normalized)
        if match is None:
            msg = "Invalid calendar version. Expected YEAR.MONTH (optionally prefixed with 'v')"
            raise ValueError(msg)
        return cls(int(match.group(1)), int(match.group(2)))

    def __str__(self) -> str:
        """Return canonical calendar format."""
        return f"{self.year}.{self.month}"


@dataclass(frozen=True, slots=True, init=False)
class IncrementalVersion:
    """Incremental numeric version (single integer)."""

    number: int

    @overload
    def __init__(self, value: str, /) -> None: ...

    @overload
    def __init__(self, value: int, /) -> None: ...

    def __init__(self, number: str | int) -> None:
        """Create incremental version from either a string or an integer."""
        if isinstance(number, str):
            parsed = self.parse(number)
            object.__setattr__(self, "number", parsed.number)
            return

        if number < 0:
            msg = "Invalid incremental version. Expected a non-negative integer"
            raise ValueError(msg)
        object.__setattr__(self, "number", number)

    @classmethod
    def parse(cls, value: str) -> IncrementalVersion:
        """Parse and validate an incremental version string."""
        normalized = _strip_optional_v_prefix(value)
        match = INCREMENTAL_RE.fullmatch(normalized)
        if match is None:
            msg = (
                "Invalid incremental version. Expected a non-negative integer "
                "(optionally prefixed with 'v')"
            )
            raise ValueError(msg)
        return cls(int(match.group(1)))

    def __str__(self) -> str:
        """Return canonical incremental format."""
        return str(self.number)


Version = SemVer | CalendarVersion | IncrementalVersion
Incremental = IncrementalVersion


def parse_version(value: str) -> Version:
    """Parse version using the supported formats in specificity order."""
    for parser in (SemVer.parse, CalendarVersion.parse, IncrementalVersion.parse):
        try:
            return parser(value)
        except ValueError:
            continue

    msg = (
        "Unsupported version format. Supported: SemVer (MAJOR.MINOR.PATCH), "
        "Calendar (YEAR.MONTH), Incremental (Numbering). "
        f"Input: {value!r}."
    )
    raise ValueError(msg)


def _compare_prerelease(left: tuple[str, ...], right: tuple[str, ...]) -> int:
    """Compare prerelease identifiers according to SemVer precedence rules."""
    for left_item, right_item in zip(left, right, strict=False):
        if left_item == right_item:
            continue

        left_is_num = left_item.isdigit()
        right_is_num = right_item.isdigit()

        if left_is_num and right_is_num:
            return -1 if int(left_item) < int(right_item) else 1

        if left_is_num and not right_is_num:
            return -1
        if not left_is_num and right_is_num:
            return 1

        return -1 if left_item < right_item else 1

    if len(left) == len(right):
        return 0
    return -1 if len(left) < len(right) else 1


def _strip_optional_v_prefix(value: str) -> str:
    """Strip a single leading v prefix when present."""
    return value.removeprefix("v")
`.trim();

const webFrameworksSource = `
"""Web framework enumerations."""

from enum import Enum

from .programming_languages import ProgrammingLanguage as Language


class WebFramework(Enum):
    """A web framework enum that also stores its associated Language.

    Each member = (Language, framework_name).
    """

    django = (Language.python, "django")
    flask = (Language.python, "flask")
    fastapi = (Language.python, "fastapi")
    starlette = (Language.python, "starlette")
    tornado = (Language.python, "tornado")
    pyramid = (Language.python, "pyramid")
    sanic = (Language.python, "sanic")
    aiohttp = (Language.python, "aiohttp")
    falcon = (Language.python, "falcon")
    masonite = (Language.python, "masonite")
    bottle = (Language.python, "bottle")
    quart = (Language.python, "quart")
    express = (Language.javascript, "express")
    nextjs = (Language.javascript, "nextjs")
    nuxtjs = (Language.typescript, "nuxtjs")
    nestjs = (Language.typescript, "nestjs")
    remix = (Language.typescript, "remix")
    astro = (Language.typescript, "astro")
    sveltekit = (Language.typescript, "sveltekit")
    koa = (Language.javascript, "koa")
    hapi = (Language.javascript, "hapi")
    adonisjs = (Language.typescript, "adonisjs")
    meteor = (Language.javascript, "meteor")
    rails = (Language.ruby, "rails")
    sinatra = (Language.ruby, "sinatra")
    hanami = (Language.ruby, "hanami")
    padrino = (Language.ruby, "padrino")
    laravel = (Language.php, "laravel")
    symfony = (Language.php, "symfony")
    codeigniter = (Language.php, "codeigniter")
    yii = (Language.php, "yii")
    cakephp = (Language.php, "cakephp")
    drupal = (Language.php, "drupal")
    wordpress = (Language.php, "wordpress")
    spring = (Language.java, "spring")
    spring_boot = (Language.java, "spring_boot")
    micronaut = (Language.java, "micronaut")
    quarkus = (Language.java, "quarkus")
    sparkjava = (Language.java, "sparkjava")
    vertx = (Language.java, "vertx")
    grails = (Language.groovy, "grails")
    play = (Language.scala, "play")
    gin = (Language.go, "gin")
    fiber = (Language.go, "fiber")
    echo = (Language.go, "echo")
    beego = (Language.go, "beego")
    revel = (Language.go, "revel")
    chi = (Language.go, "chi")
    actix = (Language.rust, "actix")
    rocket = (Language.rust, "rocket")
    warp = (Language.rust, "warp")
    axum = (Language.rust, "axum")
    poem = (Language.rust, "poem")
    salvo = (Language.rust, "salvo")
    perseus = (Language.rust, "perseus")
    leptos = (Language.rust, "leptos")
    aspnet = (Language.csharp, "aspnet")
    blazor = (Language.csharp, "blazor")
    phoenix = (Language.elixir, "phoenix")
    ring = (Language.clojure, "ring")
    luminus = (Language.clojure, "luminus")
    fresh = (Language.javascript, "fresh")
    aleph = (Language.javascript, "aleph")
    ktor = (Language.kotlin, "ktor")
    vapor = (Language.swift, "vapor")
    dart_frog = (Language.dart, "dart_frog")
    yesod = (Language.haskell, "yesod")
    scotty = (Language.haskell, "scotty")
    servant = (Language.haskell, "servant")
    lapis = (Language.lua, "lapis")
    sailor = (Language.lua, "sailor")
    dancer = (Language.perl, "dancer")
    mojolicious = (Language.perl, "mojolicious")
    catalyst = (Language.perl, "catalyst")
    web_server = (Language.racket, "web_server")
    finatra = (Language.scala, "finatra")
    scalatra = (Language.scala, "scalatra")
    ocsigen = (Language.ocaml, "ocsigen")
    dream = (Language.ocaml, "dream")
    jester = (Language.nim, "jester")
    zap = (Language.zig, "zap")
    caveman2 = (Language.lisp, "caveman2")
    genie = (Language.julia, "genie")
    cowboy = (Language.erlang, "cowboy")
    coldfusion = (Language.cobol, "coldfusion")

    @property
    def language(self) -> Language:
        """Get the programming language."""
        return self.value[0]

    @property
    def framework(self) -> str:
        """Get the framework name."""
        return self.value[1]

    def __str__(self) -> str:
        """Return string representation."""
        return self.framework
`.trim();

const textShimSource = `
"""Pyodide-compatible semantic string types used by the browser SDK shim."""

class Label(str):
    """A short, single-line display label."""


class Markdown(str):
    """A Markdown-formatted text field."""


class Url(str):
    """A URL string field."""
`.trim();

const manifestShimSource = `
"""Pyodide-compatible manifest models used in the browser runner."""

from __future__ import annotations

from typing import Any

from .text import Label, Markdown, Url
from .types import (
    CalendarVersion,
    CICDToolType,
    DeploymentTarget,
    IaCToolType,
    IncrementalVersion,
    LogAggregationType,
    MonitoringType,
    ProgrammingLanguage,
    SemVer,
    TracingType,
    WebFramework,
)


Version = SemVer | CalendarVersion | IncrementalVersion


class Link:
    """A lightweight site link model."""

    def __init__(self, site_name: Label | str = "", http_url: Url | str = "") -> None:
        self.site_name = Label(site_name)
        self.http_url = Url(http_url)

    def to_dict(self) -> dict[str, str]:
        return {
            "site_name": str(self.site_name),
            "http_url": str(self.http_url),
        }


class _Section:
    """Base class for lightweight browser manifest sections."""

    def _compact(self, data: dict[str, Any]) -> dict[str, Any]:
        return {key: value for key, value in data.items() if value is not None}

    def _value(self, value: Any) -> Any:
        if isinstance(value, (Label, Markdown, Url)):
            return str(value)
        if isinstance(value, (SemVer, CalendarVersion, IncrementalVersion)):
            return str(value)
        if isinstance(value, list):
            return [self._value(item) for item in value]
        if hasattr(value, "to_dict"):
            return value.to_dict()
        return getattr(value, "value", value)


class Info(_Section):
    """Manifest section for Info data."""

    def __init__(
        self,
        *,
        description: Markdown | str = "",
        version: Version | str = "0.1.0",
        language: ProgrammingLanguage = ProgrammingLanguage.python,
        frameworks: list[WebFramework] | None = None,
        deployment: DeploymentTarget = DeploymentTarget.ec2,
        owner_team: Label | str = "",
        auth_type: Any = None,
        monitoring: MonitoringType = MonitoringType.none,
        log_aggregation: LogAggregationType = LogAggregationType.none,
        tracing: TracingType = TracingType.none,
        iac_tool: IaCToolType = IaCToolType.none,
        max_replicas: int = 1,
        min_replicas: int = 1,
        links: list[Link] | None = None,
        docs: list[Url | str] | None = None,
    ) -> None:
        self.description = Markdown(description)
        self.version = self._parse_version(version)
        self.language = language
        self.frameworks = frameworks or []
        self.deployment = deployment
        self.owner_team = Label(owner_team)
        self.auth_type = auth_type
        self.monitoring = monitoring
        self.log_aggregation = log_aggregation
        self.tracing = tracing
        self.iac_tool = iac_tool
        self.max_replicas = max_replicas
        self.min_replicas = min_replicas
        self.links = links or []
        self.docs = [Url(doc) for doc in (docs or [])]
        if self.min_replicas > self.max_replicas:
            raise ValueError("min_replicas must be less than or equal to max_replicas")

    def _parse_version(self, value: Version | str) -> Version:
        if isinstance(value, (SemVer, CalendarVersion, IncrementalVersion)):
            return value
        for constructor in (SemVer, CalendarVersion, IncrementalVersion):
            try:
                return constructor(value)
            except ValueError:
                continue
        raise ValueError(
            "Unsupported version format. Supported: SemVer, Calendar, Incremental."
        )

    def dict(self) -> dict[str, Any]:
        return {
            "description": str(self.description),
            "version": str(self.version),
            "language": self.language.value,
            "frameworks": [framework.framework for framework in self.frameworks],
            "deployment": self.deployment.id,
            "ownerTeam": str(self.owner_team),
            "authType": self.auth_type.value if self.auth_type is not None else None,
            "monitoring": self.monitoring.value,
            "logAggregation": self.log_aggregation.value,
            "tracing": self.tracing.value,
            "iacTool": self.iac_tool.value,
            "maxReplicas": self.max_replicas,
            "minReplicas": self.min_replicas,
            "links": [link.to_dict() for link in self.links],
            "docs": [str(doc) for doc in self.docs],
        }


class Github(_Section):
    """Manifest section for Github data."""

    def __init__(self, **values: Any) -> None:
        self.values = values

    def dict(self) -> dict[str, Any]:
        aliases = {
            "repo_url": "repoUrl",
            "open_issues": "openIssues",
            "open_prs": "openPRs",
            "last_commit": "lastCommit",
            "primary_language": "primaryLanguage",
            "lines_of_code": "linesOfCode",
            "outdated_deps": "outdatedDeps",
            "active_contributors": "activeContributors",
            "latest_release": "latestRelease",
            "build_status": "buildStatus",
        }
        return self._compact(
            {aliases.get(key, key): self._value(value) for key, value in self.values.items()}
        )


class CredentialRef:
    """Public name of a server connection; browser previews never resolve secrets."""

    def __init__(self, *, name):
        self.name = name

    def to_dict(self):
        return {"name": self.name}


class PrometheusSource:
    """Preserve declarative metric sources in offline browser previews."""

    def __init__(self, *, credentials, query_endpoint=None, dashboard_url=None,
                 environment="production", refresh="on_diagram_load", window=900,
                 step=60, cache_ttl=60, ingestion_delay=120, queries, units=None):
        self.values = dict(kind="prometheus", credentials=credentials.to_dict(),
                           queryEndpoint=query_endpoint, dashboardUrl=dashboard_url,
                           environment=environment, refresh=refresh, window=window,
                           step=step, cacheTtl=cache_ttl, ingestionDelay=ingestion_delay,
                           queries=queries, units=units or {})

    def to_dict(self):
        return self.values.copy()


class Metrics(_Section):
    """Manifest section for Metrics data."""

    def __init__(self, **values: Any) -> None:
        self.values = values

    def dict(self) -> dict[str, Any]:
        aliases = {
            "response_time_p50": "responseTimeP50",
            "response_time_p95": "responseTimeP95",
            "response_time_p99": "responseTimeP99",
            "request_rate": "requestRate",
            "error_rate": "errorRate",
            "success_rate": "successRate",
            "active_connections": "activeConnections",
            "cpu_usage": "cpuUsage",
            "memory_usage": "memoryUsage",
        }
        return self._compact(
            {aliases.get(key, key): self._value(value) for key, value in self.values.items()}
        )


class CICD(_Section):
    """Manifest section for CICD data."""

    def __init__(self, **values: Any) -> None:
        self.values = values

    def dict(self) -> dict[str, Any]:
        aliases = {
            "build_status": "buildStatus",
            "last_build": "lastBuild",
            "deploy_status_prod": "deployStatusProd",
            "deploy_status_staging": "deployStatusStaging",
            "deploy_frequency": "deployFrequency",
            "pipeline_duration": "pipelineDuration",
            "failed_builds": "failedBuilds",
            "github_actions": "githubActions",
        }
        return self._compact(
            {aliases.get(key, key): self._value(value) for key, value in self.values.items()}
        )


class Tests(_Section):
    """Manifest section for Tests data."""

    def __init__(self, **values: Any) -> None:
        self.values = values

    def dict(self) -> dict[str, Any]:
        aliases = {
            "coverage_trend": "coverageTrend",
            "flaky_tests": "flakyTests",
            "execution_time": "executionTime",
            "last_run": "lastRun",
        }
        return self._compact(
            {aliases.get(key, key): self._value(value) for key, value in self.values.items()}
        )


class Dependencies(_Section):
    """Manifest section for Dependencies data."""

    def __init__(self, **values: Any) -> None:
        self.values = values

    def dict(self) -> dict[str, Any]:
        aliases = {
            "total_deps": "totalDeps",
            "outdated_deps": "outdatedDeps",
            "vulnerable_deps": "vulnerableDeps",
            "max_severity": "maxSeverity",
            "direct_deps": "directDeps",
            "transitive_deps": "transitiveDeps",
            "update_lag": "updateLag",
            "license_compliance": "licenseCompliance",
        }
        return self._compact(
            {aliases.get(key, key): self._value(value) for key, value in self.values.items()}
        )


class Performance(_Section):
    """Manifest section for Performance data."""

    def __init__(self, **values: Any) -> None:
        self.values = values

    def dict(self) -> dict[str, Any]:
        aliases = {
            "cpu_history": "cpuHistory",
            "cpu_current": "cpuCurrent",
            "cpu_avg": "cpuAvg",
            "cpu_peak": "cpuPeak",
            "memory_usage": "memoryUsage",
            "time_window": "timeWindow",
        }
        return self._compact(
            {aliases.get(key, key): self._value(value) for key, value in self.values.items()}
        )


class OtherDiagram(_Section):
    """Manifest section for OtherDiagram data."""

    def __init__(self, *, diagrams: list[Label | str] | None = None) -> None:
        self.diagrams = [Label(diagram) for diagram in (diagrams or [])]

    def dict(self) -> dict[str, Any]:
        return {"diagrams": [str(diagram) for diagram in self.diagrams]}


class Manifest:
    """A lightweight browser port of the SDK Manifest model."""

    Info = Info
    Github = Github
    Metrics = Metrics
    CICD = CICD
    Tests = Tests
    Dependencies = Dependencies
    Performance = Performance
    OtherDiagram = OtherDiagram

    def __init__(
        self,
        *,
        manifest_id: str | None = None,
        name: Label | str = "",
        icon: Label | str = "",
        tags: list[Label | str] | None = None,
        connections: list[Label | str] | None = None,
        forges: list[Label | str] | None = None,
        classification: Any = None,
        consumer_type: Any = None,
        info: Info | None = None,
        github: Github | None = None,
        metrics: Metrics | None = None,
        cicd: CICD | None = None,
        tests: Tests | None = None,
        dependencies: Dependencies | None = None,
        performance: Performance | None = None,
        other_diagram: OtherDiagram | None = None,
        description: Markdown | str | None = None,
        version: Version | str | None = None,
        language: ProgrammingLanguage | None = None,
        frameworks: list[WebFramework] | None = None,
        deployment: DeploymentTarget | None = None,
        owner_team: Label | str | None = None,
        auth_type: Any = None,
        monitoring: MonitoringType | None = None,
        log_aggregation: LogAggregationType | None = None,
        tracing: TracingType | None = None,
        iac_tool: IaCToolType | None = None,
        max_replicas: int | None = None,
        min_replicas: int | None = None,
        links: list[Link] | None = None,
        docs: list[Url | str] | None = None,
        repo_url: Url | str | None = None,
        cicd_tool: CICDToolType | str | None = None,
    ) -> None:
        self.manifest_id = manifest_id
        self.name = Label(name)
        self.icon = Label(icon)
        self.tags = [Label(tag) for tag in (tags or [])]
        self.connections = [Label(connection) for connection in (connections or [])]
        self.forges = [Label(forge) for forge in (forges or [])]
        self.classification = classification if classification is not None else consumer_type
        info_values = {
            "description": description,
            "version": version,
            "language": language,
            "frameworks": frameworks,
            "deployment": deployment,
            "owner_team": owner_team,
            "auth_type": auth_type,
            "monitoring": monitoring,
            "log_aggregation": log_aggregation,
            "tracing": tracing,
            "iac_tool": iac_tool,
            "max_replicas": max_replicas,
            "min_replicas": min_replicas,
            "links": links,
            "docs": docs,
        }
        info_values = {
            key: value for key, value in info_values.items() if value is not None
        }
        if info_values:
            self.info = Info(**{**getattr(info, "__dict__", {}), **info_values})
        else:
            self.info = info or Info()
        self.github = (
            Github(**{**getattr(github, "values", {}), "repo_url": repo_url})
            if repo_url is not None
            else github
        )
        self.metrics = metrics
        self.cicd = (
            CICD(**{**getattr(cicd, "values", {}), "platform": cicd_tool})
            if cicd_tool is not None
            else cicd
        )
        self.tests = tests
        self.dependencies = dependencies
        self.performance = performance
        self.other_diagram = other_diagram
    def framework_names(self) -> list[str]:
        return [str(framework) for framework in self.info.frameworks]

    def dict(self) -> dict[str, Any]:
        data = {
            "manifestId": self.manifest_id,
            "name": str(self.name),
            "icon": str(self.icon),
            "tags": [str(tag) for tag in self.tags],
            "connections": [str(connection) for connection in self.connections],
            "forges": [str(forge) for forge in self.forges],
            "info": self.info.dict(),
        }
        if self.classification is not None:
            data["classification"] = getattr(self.classification, "value", self.classification)
        sections = {
            "github": self.github,
            "metrics": self.metrics,
            "cicd": self.cicd,
            "tests": self.tests,
            "dependencies": self.dependencies,
            "performance": self.performance,
            "otherDiagram": self.other_diagram,
        }
        for key, section in sections.items():
            if section is not None:
                data[key] = section.dict()
        return data

    def to_dict(self) -> dict[str, Any]:
        return self.dict()


class Diagram:
    """A lightweight browser port of the SDK Diagram model."""

    def __init__(
        self,
        *,
        name: Label | str = "",
        description: Markdown | str = "",
        manifests: list[Manifest] | None = None,
    ) -> None:
        self.name = Label(name)
        self.description = Markdown(description)
        self.manifests = manifests or []

    def dict(self) -> dict[str, Any]:
        return {
            "name": str(self.name),
            "description": str(self.description),
            "manifests": [manifest.to_dict() for manifest in self.manifests],
        }

    def to_dict(self) -> dict[str, Any]:
        return self.dict()
`.trim();

const typesShimSource = `
"""Pyodide-compatible type export surface for the browser SDK shim."""

from scryr.saas.cicd_tool import CICDToolType
from scryr.saas.data_persistence import IaCToolType
from scryr.saas.deployment_target import DeploymentTarget
from scryr.saas.environment import AuthType
from scryr.saas.interface import InterfaceType
from scryr.saas.programming_languages import ProgrammingLanguage
from scryr.saas.telemetry import LogAggregationType, MonitoringType, TracingType
from scryr.saas.web_frameworks import WebFramework
from scryr.text import Label, Markdown, Url
from scryr.version import CalendarVersion, Incremental, IncrementalVersion, SemVer, Version

__all__ = [
    "AuthType",
    "CICDToolType",
    "CalendarVersion",
    "DeploymentTarget",
    "IaCToolType",
    "Incremental",
    "IncrementalVersion",
    "InterfaceType",
    "Label",
    "LogAggregationType",
    "Markdown",
    "MonitoringType",
    "ProgrammingLanguage",
    "SemVer",
    "TracingType",
    "Url",
    "Version",
    "WebFramework",
]
`.trim();

const packageInitSource = `
"""Browser package initializer for the Scryr SDK shim."""

from .github import ActionStatusEvent, GithubActionRun, GithubActionsLog

from .manifest import (
    CICD,
    CredentialRef,
    PrometheusSource,
    Dependencies,
    Diagram,
    Github,
    Info,
    Link,
    Manifest,
    Metrics,
    OtherDiagram,
    Performance,
    Tests,
)
from .types import (
    AuthType,
    CICDToolType,
    CalendarVersion,
    DeploymentTarget,
    IaCToolType,
    Incremental,
    IncrementalVersion,
    InterfaceType,
    Label,
    LogAggregationType,
    Markdown,
    MonitoringType,
    ProgrammingLanguage,
    SemVer,
    TracingType,
    Url,
    Version,
    WebFramework,
)

__all__ = [
    "ActionStatusEvent",
    "GithubActionRun",
    "GithubActionsLog",
    "AuthType",
    "CICD",
    "CredentialRef",
    "PrometheusSource",
    "CICDToolType",
    "CalendarVersion",
    "Dependencies",
    "DeploymentTarget",
    "Diagram",
    "Github",
    "IaCToolType",
    "Info",
    "Incremental",
    "IncrementalVersion",
    "InterfaceType",
    "Label",
    "Link",
    "LogAggregationType",
    "Manifest",
    "Markdown",
    "Metrics",
    "MonitoringType",
    "OtherDiagram",
    "Performance",
    "ProgrammingLanguage",
    "SemVer",
    "Tests",
    "TracingType",
    "Url",
    "Version",
    "WebFramework",
]
`.trim();

export const browserSdkFiles: Record<string, string> = {
	"scryr/__init__.py": packageInitSource,
	"scryr/manifest.py": manifestShimSource,
	"scryr/github.py": githubShimSource,
	"scryr/text.py": textShimSource,
	"scryr/types/__init__.py": typesShimSource,
	"scryr/saas/__init__.py": "",
	"scryr/saas/cicd_tool.py": cicdToolSource,
	"scryr/saas/data_persistence.py": dataPersistenceSource,
	"scryr/saas/deployment_target.py": deploymentTargetSource,
	"scryr/saas/environment.py": environmentSource,
	"scryr/saas/interface.py": interfaceSource,
	"scryr/saas/programming_languages.py": programmingLanguagesSource,
	"scryr/saas/telemetry.py": telemetrySource,
	"scryr/saas/web_frameworks.py": webFrameworksSource,
	"scryr/version.py": versionSource,
};

export const sampleMernPython = sampleMernSource;
