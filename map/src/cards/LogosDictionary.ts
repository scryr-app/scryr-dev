export type LogoMetadata = {
	iconFile?: string;
	label?: string;
	docUrl?: string;
};

export const LogosDictionary: Record<string, LogoMetadata> = {
	// Programming languages
	bash: {
		iconFile: "bash.svg",
		docUrl: "https://www.gnu.org/software/bash/manual/",
	},
	c: { iconFile: "c.svg", docUrl: "https://en.cppreference.com/w/c" },
	clojure: {
		iconFile: "clojure.svg",
		docUrl: "https://clojure.org/reference/documentation",
	},
	cpp: {
		iconFile: "c-plus-plus.svg",
		label: "C++",
		docUrl: "https://en.cppreference.com/w/cpp",
	},
	csharp: {
		iconFile: "c-sharp.svg",
		label: "C#",
		docUrl: "https://learn.microsoft.com/dotnet/csharp/",
	},
	crystal: {
		iconFile: "crystal.svg",
		docUrl: "https://crystal-lang.org/reference/",
	},
	d: { iconFile: "dlang.svg", docUrl: "https://dlang.org/spec/spec.html" },
	dart: { iconFile: "dart.svg", docUrl: "https://dart.dev/guides" },
	dlang: {
		iconFile: "dlang.svg",
		label: "D",
		docUrl: "https://dlang.org/spec/spec.html",
	},
	elixir: {
		iconFile: "elixir.svg",
		docUrl: "https://elixir-lang.org/docs.html",
	},
	erlang: { iconFile: "erlang.svg", docUrl: "https://www.erlang.org/docs" },
	go: { iconFile: "go.svg", docUrl: "https://go.dev/doc/" },
	haskell: {
		iconFile: "haskell.svg",
		docUrl: "https://www.haskell.org/documentation/",
	},
	html: {
		iconFile: "html5.svg",
		label: "HTML",
		docUrl: "https://developer.mozilla.org/docs/Web/HTML",
	},
	java: { iconFile: "java.svg", docUrl: "https://docs.oracle.com/en/java/" },
	javascript: {
		iconFile: "javascript.svg",
		label: "JavaScript",
		docUrl: "https://developer.mozilla.org/docs/Web/JavaScript",
	},
	json: { iconFile: "json.svg", docUrl: "https://www.json.org/json-en.html" },
	kotlin: {
		iconFile: "kotlin.svg",
		docUrl: "https://kotlinlang.org/docs/home.html",
	},
	lua: { iconFile: "lua.svg", docUrl: "https://www.lua.org/manual/" },
	nim: { iconFile: "nim.svg", docUrl: "https://nim-lang.org/docs/" },
	ocaml: { iconFile: "ocaml.svg", docUrl: "https://ocaml.org/docs" },
	php: { iconFile: "php.svg", docUrl: "https://www.php.net/docs.php" },
	powershell: {
		iconFile: "powershell.svg",
		docUrl: "https://learn.microsoft.com/powershell/",
	},
	python: { iconFile: "python.svg", docUrl: "https://docs.python.org/3/" },
	ruby: {
		iconFile: "ruby.svg",
		docUrl: "https://www.ruby-lang.org/en/documentation/",
	},
	rust: { iconFile: "rust-light.svg", docUrl: "https://doc.rust-lang.org/" },
	scala: { iconFile: "scala.svg", docUrl: "https://docs.scala-lang.org/" },
	shell: {
		iconFile: "bash.svg",
		docUrl: "https://www.gnu.org/software/bash/manual/",
	},
	solidity: {
		iconFile: "solidity.svg",
		docUrl: "https://docs.soliditylang.org/",
	},
	sql: {
		iconFile: "postgresql.svg",
		label: "SQL",
		docUrl: "https://www.postgresql.org/docs/",
	},
	swift: {
		iconFile: "swift.svg",
		docUrl: "https://www.swift.org/documentation/",
	},
	terraform: {
		iconFile: "terraform.svg",
		docUrl: "https://developer.hashicorp.com/terraform/docs",
	},
	typescript: {
		iconFile: "typescript.svg",
		label: "TypeScript",
		docUrl: "https://www.typescriptlang.org/docs/",
	},
	vue: {
		iconFile: "vuejs.svg",
		label: "Vue",
		docUrl: "https://vuejs.org/guide/",
	},
	wasm: {
		iconFile: "webassembly.svg",
		label: "WebAssembly",
		docUrl: "https://webassembly.org/docs/",
	},
	webassembly: {
		iconFile: "webassembly.svg",
		label: "WebAssembly",
		docUrl: "https://webassembly.org/docs/",
	},

	// Web frameworks
	angular: { iconFile: "angular.svg", docUrl: "https://angular.dev/" },
	astro: { iconFile: "astro.svg", docUrl: "https://docs.astro.build/" },
	backbone: { iconFile: "backbonejs.svg", docUrl: "https://backbonejs.org/" },
	backbonejs: {
		iconFile: "backbonejs.svg",
		label: "Backbone.js",
		docUrl: "https://backbonejs.org/",
	},
	cakephp: { iconFile: "cakephp.svg", docUrl: "https://book.cakephp.org/" },
	codeigniter: {
		iconFile: "codeigniter.svg",
		docUrl: "https://codeigniter.com/user_guide/",
	},
	django: { iconFile: "django.svg", docUrl: "https://docs.djangoproject.com/" },
	express: {
		iconFile: "expressjs-light.svg",
		label: "Express",
		docUrl: "https://expressjs.com/",
	},
	expressjs: {
		iconFile: "expressjs-light.svg",
		label: "Express",
		docUrl: "https://expressjs.com/",
	},
	fastapi: {
		iconFile: "fast-api.svg",
		label: "FastAPI",
		docUrl: "https://fastapi.tiangolo.com/",
	},
	flask: {
		iconFile: "flask-light.svg",
		docUrl: "https://flask.palletsprojects.com/",
	},
	gatsby: { iconFile: "gatsby.svg", docUrl: "https://www.gatsbyjs.com/docs/" },
	laravel: { iconFile: "laravel.svg", docUrl: "https://laravel.com/docs" },
	nestjs: {
		iconFile: "nestjs.svg",
		label: "NestJS",
		docUrl: "https://docs.nestjs.com/",
	},
	nextjs: {
		iconFile: "nextjs.svg",
		label: "Next.js",
		docUrl: "https://nextjs.org/docs",
	},
	nuxtjs: {
		iconFile: "nuxtjs.svg",
		label: "Nuxt",
		docUrl: "https://nuxt.com/docs",
	},
	rails: { iconFile: "rails.svg", docUrl: "https://guides.rubyonrails.org/" },
	remix: { iconFile: "remix-light.svg", docUrl: "https://remix.run/docs" },
	spring: {
		iconFile: "spring.svg",
		docUrl: "https://docs.spring.io/spring-framework/reference/",
	},
	spring_boot: {
		iconFile: "spring.svg",
		label: "Spring Boot",
		docUrl: "https://docs.spring.io/spring-boot/",
	},
	sveltekit: {
		iconFile: "sveltejs.svg",
		label: "SvelteKit",
		docUrl: "https://svelte.dev/docs/kit",
	},
	wordpress: {
		iconFile: "wordpress.svg",
		docUrl: "https://developer.wordpress.org/",
	},

	// Deployment, providers, and platforms
	amplify: {
		iconFile: "aws.svg",
		docUrl: "https://docs.aws.amazon.com/amplify/",
	},
	app_engine: {
		iconFile: "google-cloud.svg",
		label: "App Engine",
		docUrl: "https://cloud.google.com/appengine/docs",
	},
	app_service: {
		iconFile: "azure.svg",
		label: "Azure App Service",
		docUrl: "https://learn.microsoft.com/azure/app-service/",
	},
	aws: { iconFile: "aws.svg", docUrl: "https://docs.aws.amazon.com/" },
	aws_xray: {
		iconFile: "aws.svg",
		label: "AWS X-Ray",
		docUrl: "https://docs.aws.amazon.com/xray/",
	},
	azure: {
		iconFile: "azure.svg",
		docUrl: "https://learn.microsoft.com/azure/",
	},
	cloud_functions: {
		iconFile: "google-cloud.svg",
		label: "Cloud Functions",
		docUrl: "https://cloud.google.com/functions/docs",
	},
	cloud_run: {
		iconFile: "google-cloud.svg",
		label: "Cloud Run",
		docUrl: "https://cloud.google.com/run/docs",
	},
	cloud_storage: {
		iconFile: "google-cloud.svg",
		label: "Cloud Storage",
		docUrl: "https://cloud.google.com/storage/docs",
	},
	cloudflare_pages: {
		iconFile: "cloudflare.svg",
		label: "Cloudflare Pages",
		docUrl: "https://developers.cloudflare.com/pages/",
	},
	cloudflare_workers: {
		iconFile: "cloudflare.svg",
		label: "Cloudflare Workers",
		docUrl: "https://developers.cloudflare.com/workers/",
	},
	cloudfront: {
		iconFile: "aws.svg",
		docUrl: "https://docs.aws.amazon.com/cloudfront/",
	},
	cloudwatch: {
		iconFile: "aws.svg",
		label: "CloudWatch",
		docUrl: "https://docs.aws.amazon.com/cloudwatch/",
	},
	compute_engine: {
		iconFile: "google-cloud.svg",
		label: "Compute Engine",
		docUrl: "https://cloud.google.com/compute/docs",
	},
	digitalocean: {
		iconFile: "digitalocean.svg",
		docUrl: "https://docs.digitalocean.com/",
	},
	digitalocean_app: {
		iconFile: "digitalocean.svg",
		label: "DigitalOcean App Platform",
		docUrl: "https://docs.digitalocean.com/products/app-platform/",
	},
	docker: { iconFile: "docker.svg", docUrl: "https://docs.docker.com/" },
	ec2: {
		iconFile: "ec2.svg",
		label: "Amazon EC2",
		docUrl: "https://docs.aws.amazon.com/ec2/",
	},
	ecr: {
		iconFile: "aws.svg",
		label: "Amazon ECR",
		docUrl: "https://docs.aws.amazon.com/ecr/",
	},
	ecs: {
		iconFile: "aws.svg",
		label: "Amazon ECS",
		docUrl: "https://docs.aws.amazon.com/ecs/",
	},
	eks: {
		iconFile: "kubernetes.svg",
		label: "Amazon EKS",
		docUrl: "https://docs.aws.amazon.com/eks/",
	},
	fargate: {
		iconFile: "aws.svg",
		label: "AWS Fargate",
		docUrl:
			"https://docs.aws.amazon.com/AmazonECS/latest/developerguide/AWS_Fargate.html",
	},
	firebase: {
		iconFile: "firebase.svg",
		docUrl: "https://firebase.google.com/docs",
	},
	fly_io: {
		iconFile: "flyio.svg",
		label: "Fly.io",
		docUrl: "https://fly.io/docs/",
	},
	functions: {
		iconFile: "serverless.svg",
		docUrl: "https://www.serverless.com/framework/docs",
	},
	gcp: {
		iconFile: "google-cloud.svg",
		label: "Google Cloud",
		docUrl: "https://cloud.google.com/docs",
	},
	gke: {
		iconFile: "kubernetes.svg",
		label: "Google Kubernetes Engine",
		docUrl: "https://cloud.google.com/kubernetes-engine/docs",
	},
	google_cloud: {
		iconFile: "google-cloud.svg",
		label: "Google Cloud",
		docUrl: "https://cloud.google.com/docs",
	},
	helm: { iconFile: "kubernetes.svg", docUrl: "https://helm.sh/docs/" },
	heroku: {
		iconFile: "heroku.svg",
		docUrl: "https://devcenter.heroku.com/categories/reference",
	},
	k8s_cluster: {
		iconFile: "kubernetes.svg",
		label: "Kubernetes",
		docUrl: "https://kubernetes.io/docs/",
	},
	kubernetes: {
		iconFile: "kubernetes.svg",
		docUrl: "https://kubernetes.io/docs/",
	},
	lambda: {
		iconFile: "serverless.svg",
		label: "AWS Lambda",
		docUrl: "https://docs.aws.amazon.com/lambda/",
	},
	lambda_: {
		iconFile: "serverless.svg",
		label: "AWS Lambda",
		docUrl: "https://docs.aws.amazon.com/lambda/",
	},
	netlify: { iconFile: "netlify.svg", docUrl: "https://docs.netlify.com/" },
	railway: { iconFile: "railway.svg", docUrl: "https://docs.railway.com/" },
	render: { iconFile: "render.svg", docUrl: "https://render.com/docs" },
	serverless: {
		iconFile: "serverless.svg",
		docUrl: "https://www.serverless.com/framework/docs",
	},
	vercel: { iconFile: "vercel-light.svg", docUrl: "https://vercel.com/docs" },

	// Classification enum values
	public_api: {
		iconFile: "classification-public-api.svg",
		label: "Public API",
	},
	internal_api: {
		iconFile: "classification-internal-api.svg",
		label: "Internal API",
	},
	network_router: {
		iconFile: "classification-network-router.svg",
		label: "Network Gateway",
	},
	network_gateway: {
		iconFile: "classification-network-router.svg",
		label: "Network Gateway",
	},
	public_ui: {
		iconFile: "classification-public-ui.svg",
		label: "Public UI",
	},
	mobile_app: {
		iconFile: "classification-mobile-app.svg",
		label: "Mobile App",
	},
	admin_ui: {
		iconFile: "classification-admin-ui.svg",
		label: "Admin UI",
	},
	cli: {
		iconFile: "classification-cli.svg",
		label: "CLI",
	},
	sdk: {
		iconFile: "classification-sdk.svg",
		label: "SDK",
	},
	datastore: {
		iconFile: "classification-datastore.svg",
		label: "Datastore",
	},
	database: {
		iconFile: "classification-database.svg",
		label: "Database",
	},
	object_storage: {
		iconFile: "classification-object-storage.svg",
		label: "Object Storage",
	},
	document_store: {
		iconFile: "classification-document-store.svg",
		label: "Document Store",
	},
	vector_store: {
		iconFile: "classification-vector-store.svg",
		label: "Vector Store",
	},
	columnar_store: {
		iconFile: "classification-columnar-store.svg",
		label: "Columnar Store",
	},
	cache_store: {
		iconFile: "classification-cache-store.svg",
		label: "Cache Store",
	},
	search_store: {
		iconFile: "classification-search-store.svg",
		label: "Search Store",
	},
	queue: {
		iconFile: "classification-queue.svg",
		label: "Queue",
	},
	source: {
		iconFile: "classification-source.svg",
		label: "Source",
	},
	sink: {
		iconFile: "classification-sink.svg",
		label: "Sink",
	},
	worker: {
		iconFile: "classification-worker.svg",
		label: "Worker",
	},
	scheduler: {
		iconFile: "classification-scheduler.svg",
		label: "Scheduler",
	},
	job_processor: {
		iconFile: "classification-job-processor.svg",
		label: "Job Processor",
	},

	// Authentication enum values
	auth_none: {
		iconFile: "auth-none.svg",
		label: "No Auth",
	},
	auth_api_key: {
		iconFile: "auth-api-key.svg",
		label: "API Key",
	},
	auth_oauth2: {
		iconFile: "auth-oauth2.svg",
		label: "OAuth 2",
	},
	auth_jwt: {
		iconFile: "auth-jwt.svg",
		label: "JWT",
	},
	auth_basic: {
		iconFile: "auth-basic.svg",
		label: "Basic Auth",
	},
	auth_mutual_tls: {
		iconFile: "auth-mutual-tls.svg",
		label: "Mutual TLS",
	},
	auth_service_account: {
		iconFile: "auth-service-account.svg",
		label: "Service Account",
	},

	// Infrastructure as Code enum values
	iac_terraform: {
		iconFile: "iac-terraform.svg",
		label: "Terraform",
	},
	iac_cloudformation: {
		iconFile: "iac-cloudformation.svg",
		label: "CloudFormation",
	},
	iac_arm_template: {
		iconFile: "iac-arm-template.svg",
		label: "ARM Template",
	},
	iac_pulumi: {
		iconFile: "iac-pulumi.svg",
		label: "Pulumi",
	},
	iac_cdktf: {
		iconFile: "iac-cdktf.svg",
		label: "CDKTF",
	},
	iac_helm: {
		iconFile: "iac-helm.svg",
		label: "Helm",
	},
	iac_kustomize: {
		iconFile: "iac-kustomize.svg",
		label: "Kustomize",
	},
	iac_none: {
		iconFile: "iac-none.svg",
		label: "No IaC",
	},

	// Environment enum values
	env_local: {
		iconFile: "env-local.svg",
		label: "Local",
	},
	env_development: {
		iconFile: "env-development.svg",
		label: "Development",
	},
	env_staging: {
		iconFile: "env-staging.svg",
		label: "Staging",
	},
	env_production: {
		iconFile: "env-production.svg",
		label: "Production",
	},
	env_disaster_recovery: {
		iconFile: "env-disaster-recovery.svg",
		label: "Disaster Recovery",
	},

	// Log level enum values
	log_debug: {
		iconFile: "log-debug.svg",
		label: "Debug",
	},
	log_info: {
		iconFile: "log-info.svg",
		label: "Info",
	},
	log_warning: {
		iconFile: "log-warning.svg",
		label: "Warning",
	},
	log_error: {
		iconFile: "log-error.svg",
		label: "Error",
	},
	log_critical: {
		iconFile: "log-critical.svg",
		label: "Critical",
	},

	// Interfaces, telemetry, and data-ish enum values
	datadog: { iconFile: "datadog.svg", docUrl: "https://docs.datadoghq.com/" },
	elk_stack: {
		iconFile: "elastic.svg",
		label: "ELK Stack",
		docUrl: "https://www.elastic.co/docs",
	},
	elastic: { iconFile: "elastic.svg", docUrl: "https://www.elastic.co/docs" },
	elastic_apm: {
		iconFile: "elastic.svg",
		label: "Elastic APM",
		docUrl: "https://www.elastic.co/guide/en/apm/guide/current/index.html",
	},
	event: {
		iconFile: "kafka.svg",
		docUrl: "https://kafka.apache.org/documentation/",
	},
	grafana: { iconFile: "grafana.svg", docUrl: "https://grafana.com/docs/" },
	graphql: { iconFile: "graphql.svg", docUrl: "https://graphql.org/learn/" },
	graphql_datalayer: {
		iconFile: "graphql.svg",
		label: "GraphQL data layer",
		docUrl: "https://graphql.org/learn/",
	},
	kafka: {
		iconFile: "kafka.svg",
		docUrl: "https://kafka.apache.org/documentation/",
	},
	logrocket: {
		iconFile: "logrocket.svg",
		docUrl: "https://docs.logrocket.com/",
	},
	orm: { iconFile: "prisma.svg", docUrl: "https://www.prisma.io/docs/orm" },
	rest: {
		iconFile: "swagger.svg",
		label: "REST",
		docUrl: "https://swagger.io/docs/",
	},
	search: { iconFile: "elastic.svg", docUrl: "https://www.elastic.co/docs" },
	stackdriver: {
		iconFile: "google-cloud.svg",
		docUrl: "https://cloud.google.com/monitoring/docs",
	},
	stream: { iconFile: "stream.svg", docUrl: "https://getstream.io/chat/docs/" },
	web_ui: {
		iconFile: "html5.svg",
		label: "Web UI",
		docUrl: "https://developer.mozilla.org/docs/Web",
	},
};
