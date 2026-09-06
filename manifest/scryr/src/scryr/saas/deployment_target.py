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

    # ---------------- Utility properties ----------------
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
