# Getting Started with NIBE Exporter

This tutorial walks you through setting up the NIBE Exporter for the first time. By the end, you'll have OAuth2 credentials registered, your heat pump device ID, and a running exporter showing live metrics.

**Estimated time**: 20 minutes

## Prerequisites

Before starting, ensure you have:

- A myUplink account (create one at https://myuplink.com if you don't have one)
- Docker installed locally (`docker --version` should work), OR Rust 1.85.0+ for local build
- `curl` or a browser to make HTTP requests
- Your NIBE heat pump already registered in myUplink

## Step 1: Register an OAuth2 Application on myUplink

The exporter needs OAuth2 credentials to fetch data from your heat pump. You'll create these on the myUplink developer portal.

1. Go to https://dev.myuplink.com and log in with your myUplink account credentials

2. In the left menu, click **API Clients** (or navigate to the Applications section)

3. Click **Create Application** (or **New Application**)

4. Fill in the form:
   - **Name**: `nibe-exporter` (any name is fine; this helps you identify the app later)
   - **Description**: `Local Prometheus exporter for heat pump metrics`
   - **Application Type**: Select "Confidential" (this is the secure option for server-side apps)

5. After creation, you'll see a screen with:
   - **Client ID** (looks like `abc123...`)
   - **Client Secret** (looks like `xyz789...`)

   **Save these to a secure location** — you'll need them in the next step.

6. (Optional) In the **Redirect URI** field, if prompted, you can leave it empty or set to `http://localhost` (the exporter doesn't use redirects)

Once you have your Client ID and Secret, proceed to the next step.

## Step 2: Find Your Heat Pump Device ID

The exporter needs your device ID to know which heat pump to monitor. You'll fetch this from the myUplink API using your new credentials.

1. Open a terminal and run this command, replacing `CLIENT_ID` and `CLIENT_SECRET` with your values from Step 1:

```bash
curl -s -X POST "https://api.myuplink.com/oauth/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials" \
  -d "client_id=CLIENT_ID" \
  -d "client_secret=CLIENT_SECRET" | head -c 200
```

You should see JSON output similar to:
```json
{"access_token":"eyJ0eXAi...","token_type":"Bearer","expires_in":3600}
```

2. Copy the `access_token` value (the long string inside the quotes), then run:

```bash
TOKEN="your-access-token-here"
curl -s "https://api.myuplink.com/v2/devices" \
  -H "Authorization: Bearer $TOKEN" | grep -o '"deviceId":"[^"]*"'
```

You should see output like:
```
"deviceId":"abc123def456"
```

Save this Device ID. If you have multiple devices, pick the one for your NIBE heat pump.

## Step 3: Run the Exporter with Docker

Now you'll start the exporter with your credentials.

1. In your terminal, set environment variables for your credentials:

```bash
export NIBE_CLIENT_ID="your-client-id-from-step-1"
export NIBE_CLIENT_SECRET="your-client-secret-from-step-1"
export NIBE_DEVICE_ID="your-device-id-from-step-2"
```

2. Start the exporter:

```bash
docker run --rm \
  -e NIBE_CLIENT_ID \
  -e NIBE_CLIENT_SECRET \
  -e NIBE_DEVICE_ID \
  -e NIBE_LOG_LEVEL=info \
  -p 9090:9090 \
  ghcr.io/jalet/nibe-exporter:latest
```

3. Watch the logs. You should see output like:

```
{"timestamp":"2026-03-09T...","level":"INFO","message":"Starting NIBE exporter"}
{"timestamp":"2026-03-09T...","level":"INFO","message":"Listening on 0.0.0.0:9090"}
{"timestamp":"2026-03-09T...","level":"DEBUG","message":"Polling heat pump metrics"}
```

If you see errors like "invalid client_id" or "access denied", double-check your credentials from Step 1.

## Step 4: Verify It's Working

The exporter exposes three HTTP endpoints. Test each one to confirm everything is running.

1. **Health check** (should always return 200 OK):

```bash
curl http://localhost:9090/healthz
# Expected output: OK
```

2. **Readiness check** (returns 200 OK once metrics are available):

```bash
curl http://localhost:9090/ready
# Expected output: OK (after ~10 seconds when first metrics are fetched)
```

3. **Prometheus metrics** (the actual metrics endpoint):

```bash
curl http://localhost:9090/metrics
```

You should see metrics in OpenMetrics format, like:

```
# HELP nibe_supply_temperature_celsius Supply temperature
# TYPE nibe_supply_temperature_celsius gauge
nibe_supply_temperature_celsius{device_id="abc123def456",name="BT1 Supply temp",parameter_id="40008"} 35.2

# HELP nibe_return_temperature_celsius Return temperature
# TYPE nibe_return_temperature_celsius gauge
nibe_return_temperature_celsius{device_id="abc123def456",name="BT3 Return temp",parameter_id="40083"} 28.5

# HELP nibe_external_temperature_celsius External temperature
# TYPE nibe_external_temperature_celsius gauge
nibe_external_temperature_celsius{device_id="abc123def456",name="BT20 External temp",parameter_id="40045"} 5.1

# HELP nibe_polls_total Total poll attempts
# TYPE nibe_polls_total counter
nibe_polls_total{device_id="abc123def456"} 2
```

If you see metrics, **you're done!** The exporter is successfully fetching and exposing your heat pump data.

### Troubleshooting

| Problem | Solution |
|---------|----------|
| `curl: (7) Failed to connect to localhost port 9090` | Container may not be running. Check `docker ps` or restart with `docker run` command above. |
| `/ready` returns 503 | Exporter is still authenticating. Wait 10-15 seconds and try again. |
| Metrics show all zeros or are missing | Credentials may be incorrect. Verify Client ID and Secret from myUplink portal. |
| Rate limit errors in logs | Poll interval is too aggressive. Increase `NIBE_POLL_INTERVAL` (default: 60 seconds). |

## Next Steps

- **Kubernetes Deployment**: See [Helm Production Deployment](helm-deployment.md) to deploy to a Kubernetes cluster with proper secret management
- **Prometheus Integration**: Configure Prometheus to scrape `http://localhost:9090/metrics` every 30 seconds
- **Grafana Dashboards**: Use the dashboard ConfigMap included in the Helm chart for visualizations
- **Custom Configuration**: See the README for all environment variables and advanced options

## Summary

You've successfully:

1. ✅ Created OAuth2 credentials on myUplink
2. ✅ Retrieved your device ID
3. ✅ Run the exporter in Docker
4. ✅ Verified metrics are being exposed

The exporter is now polling your NIBE heat pump every 60 seconds and exposing metrics in Prometheus format. You're ready to integrate it into your monitoring stack.
