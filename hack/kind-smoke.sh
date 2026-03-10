#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== NIBE Exporter Kind Smoke Tests ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
check_prerequisites() {
    echo "Checking prerequisites..."

    if ! command -v kind &> /dev/null; then
        echo -e "${RED}✗ kind not found${NC}"
        echo "  Install from: https://kind.sigs.k8s.io/docs/user/quick-start"
        exit 1
    fi
    echo -e "${GREEN}✓ kind found${NC}"

    if ! command -v kubectl &> /dev/null; then
        echo -e "${RED}✗ kubectl not found${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ kubectl found${NC}"

    if ! command -v helm &> /dev/null; then
        echo -e "${RED}✗ helm not found${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ helm found${NC}"

    if ! command -v docker &> /dev/null; then
        echo -e "${RED}✗ docker not found${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ docker found${NC}"
}

# Create Kind cluster
create_cluster() {
    echo ""
    echo "Creating Kind cluster..."

    CLUSTER_NAME="nibe-exporter-test"

    # Check if cluster exists
    if kind get clusters | grep -q "$CLUSTER_NAME"; then
        echo "Cluster $CLUSTER_NAME already exists"
        kind delete cluster --name "$CLUSTER_NAME" || true
    fi

    kind create cluster --name "$CLUSTER_NAME" --wait 5m

    echo -e "${GREEN}✓ Cluster created${NC}"
}

# Build Docker image
build_image() {
    echo ""
    echo "Building Docker image..."

    # Load into Kind cluster
    docker build -t nibe-exporter:smoke-test "$PROJECT_ROOT"
    kind load docker-image --name nibe-exporter-test nibe-exporter:smoke-test

    echo -e "${GREEN}✓ Image built and loaded${NC}"
}

# Deploy exporter
deploy_exporter() {
    echo ""
    echo "Deploying nibe-exporter..."

    kubectl create namespace nibe-exporter || true

    # Create secret with dummy credentials
    kubectl create secret generic nibe-credentials \
        --from-literal=client-id="test-client-id" \
        --from-literal=client-secret="test-client-secret" \
        -n nibe-exporter \
        --dry-run=client -o yaml | kubectl apply -f -

    # Deploy using Helm with test overrides
    helm install nibe-exporter "$PROJECT_ROOT/charts/nibe-exporter" \
        --namespace nibe-exporter \
        --set image.repository=nibe-exporter \
        --set image.tag=smoke-test \
        --set image.pullPolicy=Never \
        --set myuplink.clientSecretRef.name=nibe-credentials \
        --set myuplink.clientSecretRef.key=client-secret \
        --set myuplink.clientId=test-client-id \
        --set exporter.logLevel=debug

    # Wait for deployment
    kubectl wait --for=condition=available --timeout=300s \
        deployment/nibe-exporter -n nibe-exporter

    echo -e "${GREEN}✓ Exporter deployed${NC}"
}

# Run smoke tests
run_tests() {
    echo ""
    echo "Running smoke tests..."

    # Forward port
    kubectl port-forward -n nibe-exporter \
        svc/nibe-exporter 9090:80 &
    FORWARD_PID=$!
    sleep 2

    PASS=0
    FAIL=0

    # Test healthz
    echo -n "Testing /healthz endpoint... "
    if curl -s http://localhost:9090/healthz | grep -q "OK"; then
        echo -e "${GREEN}✓${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗${NC}"
        ((FAIL++))
    fi

    # Test ready (might be unavailable initially)
    echo -n "Testing /ready endpoint... "
    if curl -s http://localhost:9090/ready 2>/dev/null | grep -qE "Ready|Not ready"; then
        echo -e "${GREEN}✓${NC}"
        ((PASS++))
    else
        echo -e "${YELLOW}⚠${NC} (expected, API not mocked)"
        # Don't count as failure since API isn't mocked
    fi

    # Test metrics endpoint
    echo -n "Testing /metrics endpoint... "
    if curl -s http://localhost:9090/metrics | grep -q "# EOF"; then
        echo -e "${GREEN}✓${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗${NC}"
        ((FAIL++))
    fi

    # Kill port-forward
    kill $FORWARD_PID 2>/dev/null || true

    echo ""
    echo "Test Results: ${GREEN}${PASS} passed${NC}, ${RED}${FAIL} failed${NC}"

    if [ $FAIL -gt 0 ]; then
        return 1
    fi
    return 0
}

# Cleanup
cleanup() {
    echo ""
    echo "Cleaning up..."

    kind delete cluster --name nibe-exporter-test

    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Main
main() {
    check_prerequisites
    create_cluster
    build_image
    deploy_exporter

    if run_tests; then
        cleanup
        echo ""
        echo -e "${GREEN}=== All smoke tests passed ===${NC}"
        exit 0
    else
        cleanup
        echo ""
        echo -e "${RED}=== Some smoke tests failed ===${NC}"
        exit 1
    fi
}

# Run main
main "$@"
