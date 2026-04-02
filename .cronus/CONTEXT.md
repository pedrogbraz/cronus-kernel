# Project: NOVA CORE
Port: 5175 | DB: sqlite | Theme: dark

## Entities
- Deployment (shared): deploy_id, service, cluster, duration, status, deployed_by
- SecurityEvent (shared): event_id, event_type, source_ip, severity, status
- Endpoint (shared): path, method, requests, avg_latency, p99, error_rate
- KpiSnapshot (shared): label, value, change, icon, subtitle, page
- Notification (shared): title, message, type, read
- User (local): name, email, password, role, avatar

## Pages
- / (custom) — page-header, kpi, chart, table [requires: auth]
- /deployments (custom) — page-header, kpi, table [requires: auth]
- /analytics (custom) — page-header, kpi, chart, table [requires: auth]
- /security (custom) — page-header, kpi, table [requires: auth]
- /server (custom) — page-header, hero [requires: auth]
- /notifications (custom) — page-header, hero [requires: auth]
- /settings (custom) — page-header, hero [requires: auth]
- /login (custom) — hero
- /signup (custom) — hero

## APIs
- /deployments: GET list, GET find, POST create, PATCH update, DELETE remove
- /securityevents: GET list, POST create
- /endpoints: GET list, POST create
- /kpisnapshots: GET list, POST create
- /notifications: GET list, POST create, PATCH update, DELETE remove

## Webhooks
- deployments create -> POST https://hooks.slack.com/services/nova-core/deployments
- deployments delete -> POST https://hooks.slack.com/services/nova-core/alerts
- securityevents create -> POST https://hooks.slack.com/services/nova-core/security

## Rules
- prices in centavos — use formatPrice()
- all data sections require bind
- auth required on all admin endpoints

