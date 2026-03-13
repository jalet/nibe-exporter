# Helm Production Deployment

This guide walks you through deploying the NIBE Exporter to a production Kubernetes cluster using Helm with proper secret management and monitoring integration.

**Prerequisites**: You have working myUplink OAuth2 credentials (see [Getting Started](getting-started.md)) and a Kubernetes cluster with `helm` 3.x installed.

**Target audience**: DevOps engineers and SREs deploying to production Kubernetes.

## Prerequisites

Before starting, ensure you have:

- `helm` 3.x installed (`helm version` should work)
- Access to a Kubernetes cluster (any distribution: EKS, AKS, GKE, self-managed)
- A `monitoring` namespace (or create one with `kubectl create namespace monitoring`)
- Your myUplink OAuth2 Client ID and Client Secret from the myUplink developer portal
- Your NIBE device ID (if filtering by device)

## Step 1: Create a Kubernetes Secret for Credentials

Storing credentials directly in Helm values is a security risk. Instead, create a Kubernetes Secret and reference it.

1. Create the secret in the `monitoring` namespace:

```bash
kubectl create secret generic nibe-exporter-credentials \
  --from-literal=client-secret=YOUR_CLIENT_SECRET \
  -n monitoring
```

Replace `YOUR_CLIENT_SECRET` with your actual Client Secret from the myUplink portal.

2. Verify the secret was created:

```bash
kubectl get secret nibe-exporter-credentials -n monitoring
```

You should see:
```
NAME                            TYPE     DATA   AGE
nibe-exporter-credentials       Opaque   1      5s
```

## Step 2: Create a Helm Values Override File

Create a file called `values-prod.yaml` with your deployment configuration:

```yaml
# values-prod.yaml
image:
  tag: latest  # or pin to a specific version (e.g., v0.1.0)

myuplink:
  # OAuth2 Client ID (store this in your CI/CD secret manager, not in git)
  clientId: "YOUR_CLIENT_ID"

  # Reference the secret created in Step 1
  clientSecretRef:
    name: nibe-exporter-credentials
    key: client-secret

  # API version (v2 or v3, default: v2)
  apiVersion: "v2"

  # Optional: filter to a specific device (leave empty to monitor all devices)
  deviceId: "YOUR_DEVICE_ID"

exporter:
  logLevel: info     # debug for troubleshooting, info for production
  logJson: true      # structured logging for log aggregation
  pollInterval: 60   # seconds between polls to myUplink API

# Enable Prometheus integration
serviceMonitor:
  enabled: true
  interval: 30s      # Prometheus scrape interval
  scrapeTimeout: 10s

# Enable alerting rules (optional)
prometheusRule:
  enabled: true

# Enable network security (optional but recommended)
networkPolicy:
  enabled: true
```

**Important security notes**:

- Store `CLIENT_ID` in your CI/CD system (GitHub Secrets, GitLab Variables, etc.), not in git
- The secret itself (`clientSecretRef`) is already stored securely in Kubernetes
- Use `logJson: true` for structured logging compatible with log aggregation stacks (ELK, DataDog, etc.)

## Step 3: Install the Helm Chart

The chart is published as an OCI artifact. Install directly using the OCI URL:

```bash
# Install the chart
helm install nibe-exporter oci://ghcr.io/jalet/nibe-exporter-rs/nibe-exporter \
  --namespace monitoring \
  --create-namespace \
  -f values-prod.yaml
```

Note: OCI registries do not require `helm repo add` or `helm repo update`. Install and upgrade directly with the OCI URL.

Watch the rollout:

```bash
kubectl rollout status deployment/nibe-exporter -n monitoring
```

Wait for the status to show "deployment 'nibe-exporter' successfully rolled out".

## Step 4: Verify the Deployment

Check that the pod is running and ready:

```bash
kubectl get pods -n monitoring -l app.kubernetes.io/name=nibe-exporter
```

You should see:
```
NAME                                      READY   STATUS    RESTARTS   AGE
nibe-exporter-abc123def456                1/1     Running   0          30s
```

If the pod is in `CrashLoopBackOff` or `Pending`, check the logs:

```bash
kubectl logs -n monitoring -l app.kubernetes.io/name=nibe-exporter
```

Common issues:
- **"client_id is required"**: Your `clientId` in `values-prod.yaml` is missing or empty
- **"Failed to find secret"**: The secret name in `clientSecretRef.name` doesn't exist (check Step 1)
- **"Authentication failed"**: Your credentials are incorrect; verify on myUplink portal

## Step 5: Enable Prometheus Scraping (ServiceMonitor)

If you have Prometheus Operator installed (check with `kubectl get crd servicemonitor`), the chart automatically creates a ServiceMonitor.

Verify Prometheus is scraping the exporter:

1. Port-forward to Prometheus:

```bash
kubectl port-forward -n monitoring svc/prometheus 9090:9090
```

(Adjust the service name if your Prometheus is named differently)

2. Open http://localhost:9090 in your browser

3. Go to **Status** > **Targets**

4. Search for `nibe-exporter` — you should see it in the **Targets** list with status **UP**

If the target shows **DOWN**, check:
- ServiceMonitor labels match your Prometheus configuration
- The exporter pod is ready (`kubectl get pod ... -n monitoring`)
- Network policies allow traffic between Prometheus and the exporter namespace

## Step 6: (Optional) Enable Alerting Rules

If you enabled `prometheusRule.enabled: true` in Step 2, the chart creates PrometheusRule objects with the following alerts:

- **NIBEExporterDown** — Pod down for 5+ minutes (firing status from Prometheus `up` metric)
- **NIBEAuthenticationFailures** — Repeated 401 errors
- **NIBEHighRateLimit** — API rate limiting detected
- **NIBEScrapeErrors** — Metrics collection failing

**Note**: Three alerts (`NIBEAuthenticationFailures`, `NIBEHighRateLimit`, `NIBEScrapeErrors`) reference counter metrics (`auth_failures_total`, `rate_limited_total`, `scrape_errors_total`) that are currently tracked internally and visible in exporter logs at debug/info level, but **not exported as Prometheus metrics**. These alerts will not fire until the exporter is updated to export these counters as gauge metrics. For now, monitor exporter health via:
- The `NIBEExporterDown` alert (which uses the `up` metric from Prometheus)
- Exporter logs with `NIBE_LOG_LEVEL=debug` to see authentication, rate limit, and error events

To view the rules:

```bash
kubectl get prometheusrule -n monitoring
kubectl describe prometheusrule nibe-exporter -n monitoring
```

Ensure your Prometheus is configured to watch PrometheusRules in this namespace (usually done via `ruleNamespaceSelector`).

## Step 7: (Optional) Enable Network Policies

For production clusters, restrict network traffic to/from the exporter pod.

To enable Kubernetes NetworkPolicy (blocks all traffic except explicitly allowed):

1. Update `values-prod.yaml`:

```yaml
networkPolicy:
  enabled: true
  # Exporter needs egress to myUplink API (HTTPS/443)
  # Prometheus scrapes on port 9090
```

2. Upgrade the release:

```bash
helm upgrade nibe-exporter nibe-exporter/nibe-exporter \
  -n monitoring \
  -f values-prod.yaml
```

To enable Cilium NetworkPolicy (if you use Cilium for CNI):

```yaml
ciliumNetworkPolicy:
  enabled: true
```

## Step 8: Verify Metrics Are Being Collected

Port-forward to the exporter and fetch metrics directly:

```bash
kubectl port-forward -n monitoring svc/nibe-exporter 9090:80
```

In another terminal:

```bash
curl http://localhost:9090/metrics | head -20
```

You should see metrics like:

```
# HELP nibe_parameter_40008 BT1 Supply temp
# TYPE nibe_parameter_40008 gauge
nibe_parameter_40008{device_id="...",parameter_id="40008",parameter_name="BT1 Supply temp"} 35.2

# HELP nibe_parameter_40083 BT3 Return temp
# TYPE nibe_parameter_40083 gauge
nibe_parameter_40083{device_id="...",parameter_id="40083",parameter_name="BT3 Return temp"} 28.5
```

If metrics are missing:
- Wait 60 seconds (default poll interval) for the first metrics
- Check exporter logs: `kubectl logs -n monitoring -l app.kubernetes.io/name=nibe-exporter`
- Verify myUplink credentials in the secret: `kubectl get secret nibe-exporter-credentials -n monitoring -o jsonpath='{.data.client-secret}'` (will be base64 encoded)

## Step 9: Upgrade and Rollback

### Upgrade to a New Version

Upgrade to a new chart version using the OCI URL:

```bash
helm upgrade nibe-exporter oci://ghcr.io/jalet/nibe-exporter-rs/nibe-exporter \
  -n monitoring \
  -f values-prod.yaml
```

Check rollout status:

```bash
kubectl rollout status deployment/nibe-exporter -n monitoring
```

### Rollback to Previous Version

If the upgrade causes issues:

```bash
# List previous releases
helm history nibe-exporter -n monitoring

# Rollback to previous release
helm rollback nibe-exporter 1 -n monitoring
```

(Replace `1` with the revision number from the history)

## Troubleshooting

### Pod Not Starting

```bash
kubectl describe pod nibe-exporter-xxx -n monitoring
```

Look for events like **ImagePullBackOff** (registry auth issue) or **CrashLoopBackOff** (credential error).

### Metrics Not Appearing in Prometheus

1. Verify ServiceMonitor exists:

```bash
kubectl get servicemonitor -n monitoring
```

2. Check Prometheus is watching this namespace:

```bash
kubectl describe prometheus -n monitoring | grep -A 5 ruleNamespaceSelector
```

3. Port-forward to exporter and test directly:

```bash
kubectl port-forward -n monitoring svc/nibe-exporter 9090:80
curl http://localhost:9090/metrics
```

### Rate Limiting from myUplink

If you see errors like "429 Too Many Requests" in logs:

1. Increase poll interval in `values-prod.yaml`:

```yaml
exporter:
  pollInterval: 120  # increase from 60
```

2. Upgrade:

```bash
helm upgrade nibe-exporter nibe-exporter/nibe-exporter \
  -n monitoring \
  -f values-prod.yaml
```

### Authentication Failures

If logs show "401 Unauthorized":

1. Verify Client Secret hasn't changed on myUplink portal
2. Update the secret:

```bash
kubectl delete secret nibe-exporter-credentials -n monitoring
kubectl create secret generic nibe-exporter-credentials \
  --from-literal=client-secret=NEW_CLIENT_SECRET \
  -n monitoring
```

3. Restart the exporter:

```bash
kubectl rollout restart deployment/nibe-exporter -n monitoring
```

## Configuration Reference

For all available options, see the chart's `values.yaml`:

```bash
helm show values nibe-exporter/nibe-exporter
```

Common overrides:

| Option | Purpose | Example |
|--------|---------|---------|
| `image.tag` | Container image version | `v0.1.0` or `latest` |
| `exporter.logLevel` | Log verbosity | `debug`, `info`, `warn` |
| `exporter.pollInterval` | Poll frequency (seconds) | `60`, `120` |
| `serviceMonitor.enabled` | Enable Prometheus scraping | `true` or `false` |
| `prometheusRule.enabled` | Enable alerting rules | `true` or `false` |
| `networkPolicy.enabled` | Enable network segmentation | `true` or `false` |
| `resources.limits.memory` | Container memory limit | `256Mi`, `512Mi` |

## Summary

You've successfully:

1. ✅ Stored credentials securely in Kubernetes Secrets
2. ✅ Deployed the exporter using Helm with secure configuration
3. ✅ Configured Prometheus to scrape metrics
4. ✅ (Optional) Enabled alerting and network policies
5. ✅ Verified metrics are being collected

The exporter is now running in production, securely manages credentials via Kubernetes Secrets, and integrates with your Prometheus monitoring stack. Metrics from your NIBE heat pump are being collected every 60 seconds and exposed for Prometheus scraping every 30 seconds.

## Next Steps

- **Grafana Dashboards**: The chart includes a ConfigMap with Grafana dashboard JSON. Import it into Grafana to visualize heat pump data
- **Alert Configuration**: Set up alert notification channels (Slack, PagerDuty, etc.) in Prometheus to be notified of issues
- **Multi-Device Deployment**: Run multiple exporter instances for multiple heat pumps, each with its own `deviceId`
- **Backup Credentials**: Ensure your myUplink Client Secret is backed up securely (e.g., in your organization's secret vault)
