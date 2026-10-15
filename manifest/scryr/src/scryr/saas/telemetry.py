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
