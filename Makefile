.PHONY: help build test check clippy fmt doc clean docker-build helm-lint kind-smoke all

CARGO ?= cargo
DOCKER ?= docker
HELM ?= helm

help:
	@echo "nibe-exporter-rs development targets"
	@echo ""
	@echo "  build          - Build binary"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-int       - Run integration tests only"
	@echo "  check          - Run cargo check"
	@echo "  clippy         - Run clippy linter"
	@echo "  fmt            - Format code"
	@echo "  fmt-check      - Check code formatting"
	@echo "  doc            - Generate documentation"
	@echo "  clean          - Clean build artifacts"
	@echo "  docker-build   - Build Docker image locally"
	@echo "  helm-lint      - Lint Helm chart"
	@echo "  kind-smoke     - Run smoke tests on Kind cluster"
	@echo "  all            - Run check, test, clippy"

build:
	$(CARGO) build --release

test:
	$(CARGO) test --all

test-unit:
	$(CARGO) test --lib

test-int:
	$(CARGO) test --test '*' -- --test-threads=1

check:
	$(CARGO) check --all

clippy:
	$(CARGO) clippy --all -- -D warnings

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

doc:
	$(CARGO) doc --no-deps --open

clean:
	$(CARGO) clean

docker-build:
	$(DOCKER) buildx bake dev

helm-lint:
	$(HELM) lint charts/nibe-exporter

kind-smoke:
	bash hack/kind-smoke.sh

all: check test clippy
	@echo "All checks passed!"
