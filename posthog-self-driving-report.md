# PostHog Self-driving setup report

## Summary

PostHog Self-driving has been configured with Session Replay, Error Tracking, and Support enabled, along with setup-health, error, and support-ticket signal sources. The scout troop has been kept deliberately small and focused on general product coverage plus web traffic health.

Findings will start appearing in the [Self-driving inbox](https://us.posthog.com/project/623629/inbox) within about 30 minutes as data and scheduled checks begin to arrive.

## AI data processing

Approved. The organization-level approval gate was confirmed by the setup wizard before this run.

## GitHub

Connected before this setup run through the PostHog GitHub App.

## Products enabled

| Product | Status | Notes |
|---|---|---|
| Session Replay | enabled | This is a browser-facing documentation site. Its `posthog.init` configuration has no session-recording opt-out. No recordings exist yet. |
| Error Tracking | enabled | The browser configuration enables exception capture and does not disable it. |
| Support (Conversations) | enabled | Tickets will begin arriving only after an inbound email, inbox, or Slack channel is connected in PostHog. |

## Signal sources

| Source product | Source type | Action | Notes |
|---|---|---|---|
| `health_checks` | `health_issue` | enabled | Surfaces actionable instrumentation and configuration health issues. |
| `error_tracking` | `issue_created` | enabled | Surfaces newly created error issues. |
| `error_tracking` | `issue_reopened` | enabled | Surfaces error issues that recur after resolution. |
| `error_tracking` | `issue_spiking` | enabled | Surfaces error issues with rapidly increasing volume. |
| `conversations` | `ticket` | enabled | Dormant until an inbound Support channel is connected. |
| `signals_scout` | `cross_source_issue` | on by default | No config row is needed unless the source is explicitly opted out. |
| `session_replay` | `session_analysis_cluster` | skipped | This retired route is deliberately not used; Replay Vision scanners are the supported route. |
| `replay_vision` | scanner `emits_signals` | deferred | Scanners were not safely created; see Replay Vision scanners and Follow-ups. |

## Connected tools

No external issue tracker, support desk, error tracker, or other connected tool was selected. No connected-tool responder was enabled.

## Scout troop

**Enabled (2):**

| Scout | Why it is enabled |
|---|---|
| `signals-scout-general` | Covers cross-product correlations and broadly investigates surfaces without a dedicated active specialist. |
| `signals-scout-web-analytics` | The repository includes an analytics-instrumented browser documentation site, so it watches traffic, attribution, landing-page health, bounce changes, and 404 changes. |

**Disabled (25):**

| Scout | Reason |
|---|---|
| `signals-scout-ai-observability` | No confirmed LLM analytics surface. |
| `signals-scout-anomaly-detection` | No confirmed established insight/dashboard surface to monitor. |
| `signals-scout-apm` | No confirmed distributed-tracing surface. |
| `signals-scout-conversations` | Support was just enabled and has no inbound channel yet. |
| `signals-scout-csp-violations` | No confirmed CSP reporting surface. |
| `signals-scout-customer-analytics` | No confirmed account/group analytics surface. |
| `signals-scout-data-pipelines` | No confirmed CDP, batch export, or Hog Flow surface. |
| `signals-scout-data-warehouse` | No warehouse source is connected. |
| `signals-scout-error-tracking` | Error tracking is covered by the native Self-driving error sources. |
| `signals-scout-experiments` | No confirmed active experiment surface. |
| `signals-scout-feature-flags` | No confirmed active feature-flag surface. |
| `signals-scout-inbox-validation` | Fresh setup with no resolved Self-driving reports yet; keeping the troop selective. |
| `signals-scout-insight-alerts` | No confirmed insight-alert surface. |
| `signals-scout-logs` | No confirmed PostHog Logs surface. |
| `signals-scout-mcp-tool-calls` | No confirmed MCP telemetry surface. |
| `signals-scout-observability-gaps` | Kept off until event and insight coverage is established. |
| `signals-scout-pr-follow-up` | No confirmed deployed-fix feedback loop. |
| `signals-scout-product-analytics` | No confirmed saved product-flow/retention analysis surface. |
| `signals-scout-replay-vision` | Kept off to avoid duplicate coverage; Replay Vision scanners are the intended route once configured. |
| `signals-scout-revenue-analytics` | No confirmed revenue data source. |
| `signals-scout-session-replay` | Kept off to avoid duplicate coverage; Replay Vision scanners are the intended route once configured. |
| `signals-scout-skills-store` | No confirmed active project skills-store maintenance surface. |
| `signals-scout-surveys` | No active surveys were found. |
| `signals-scout-tasks` | No confirmed PostHog Tasks surface. |
| `signals-scout-web-vitals` | Web traffic is the evidenced active surface; Web Vitals can be enabled later once it has sufficient data. |

**Run budget:** 100 runs per day; 0 used today; 100 remaining. The current announcement says: “Scouts are in early access. Each project gets up to 100 scout runs a day. Contact team-self-driving@posthog.com if you need more.”

## Custom scouts

No custom scouts were created. Two project-specific candidates were proposed and declined:

- **Documentation-to-app journey:** would watch for sustained drops in quick-start selection and hosted-app entry, beyond the built-in web scout’s traffic and landing-page checks.
- **Developer language demand:** would watch for material shifts or silence in language-selection activity, which could indicate a documentation coverage gap.

Other specialist surfaces (revenue, AI observability, surveys, experiments, feature flags, logs, warehouse, and APM) were ruled out because the repository scan and server-side probes did not confirm their use. If a future custom scout is noisy, set `emit: false` on its config in PostHog to leave it running in dry-run mode without sending findings to the inbox.

## Replay Vision scanners

A Replay Vision scanner is an LLM that watches individual session recordings on a schedule and pushes confirmed defects to the inbox. It is the only component in this setup that spends Replay Vision quota; individual findings have half weight and require independent corroboration before a report is promoted.

| Monitor brief | Status | Intended scope | Sampling / estimate |
|---|---|---|---|
| Breakage monitor | skipped | The `/getting-started/` documentation flow, the repository’s clearest completion-oriented path. | Planned narrow scope: focused, 10% sample; 0 matching sessions in the seven-day estimate window; 0 estimated observations and 0 estimated monthly credits. |
| Frustration monitor | skipped | Frustration interactions such as rage clicks, intentionally separate from the URL-scoped breakage monitor. | Not estimated because the required locked query scaffold was unavailable. |

No session recordings or existing scanners were found. Replay Vision has 2,500 credits remaining and is not exhausted, so quota is not the blocker. The setup workflow required shared scanner skills containing locked monitor prompts and query definitions; those skills were not available from the local skill catalog, and creating approximate scanners would risk overlapping coverage or incorrect targeting. This blocker was reported to the PostHog team.

## Follow-ups

- [ ] Connect an inbound Support channel (email, inbox, or Slack) in PostHog so the enabled Support ticket source can receive tickets.
- [ ] Make the shared Replay Vision scanner skill bundle available, then re-run this setup to create the two required signal-emitting monitors safely.
- [ ] When Replay Vision scanners are created, they will begin working as soon as recordings arrive; Session Replay is already enabled.
- [ ] Enable additional specialists from the inbox if the project later adopts their surfaces, especially product analytics, Web Vitals, feature flags, surveys, revenue analytics, or logs.

## Repository changes

| File | Change |
|---|---|
| `posthog-self-driving-report.md` | Created this setup and handoff report. |

No application source files were modified. The existing browser PostHog initialization already keeps Session Replay available and enables exception capture.

## What happens next

The scout coordinator picks up fresh configurations within about 30 minutes. Scout runs draw from the daily run budget, findings cluster into reports in the inbox, and immediately actionable reports can begin coding tasks.
