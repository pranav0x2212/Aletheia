# =============================================================================
# Aletheia Makefile - Memory Engine Benchmarking Framework
# =============================================================================
#
# Build & Development
#   build      Build project in debug mode
#   release    Build project in release mode (optimized)
#   check      Run cargo check
#   fmt        Format source code
#   lint       Run Clippy lints
#   test       Run all tests
#   clean      Remove build artifacts
#   dev        Format, check and test
#
# Runtime
#   host       Run the host CLI client
#   node       Run the memory engine server
#
# Workloads
#   scan       Run memory scan workload
#   vecadd     Run vector addition workload
#   stride     Run stride scan workload
#   pointer    Run pointer chase workload
#   benchmark  Run comparative benchmark
#
# Experiments
#   scaling    Run dataset scaling experiment
#   strides    Run stride testing experiment
#   wsweep     Run working-set sweep experiment
#
# Visualization
#   plot-scaling  Generate dataset scaling plots
#   plot-stride   Generate stride scan plots
#   plots         Generate all plots
#
# =============================================================================

.DEFAULT_GOAL := help

CARGO  := cargo
PYTHON := python3

NODE_ADDR ?= 127.0.0.1:9000

HOST := $(CARGO) run --bin aletheia-host --release --
NODE := $(CARGO) run --bin aletheia-node --release

.PHONY: \
	help \
	build release check fmt lint test clean dev \
	host node \
	scan vecadd stride pointer benchmark \
	scaling strides wsweep \
	plot-scaling plot-stride plots

help:
	@echo "Aletheia - Available targets"
	@echo ""
	@echo "Build & Development:"
	@echo "  build         Build project"
	@echo "  release       Build optimized binaries"
	@echo "  check         Run cargo check"
	@echo "  fmt           Format source code"
	@echo "  lint          Run Clippy"
	@echo "  test          Run tests"
	@echo "  clean         Remove build artifacts"
	@echo "  dev           Format, check and test"
	@echo ""
	@echo "Runtime:"
	@echo "  host          Run host client"
	@echo "  node          Run memory node"
	@echo ""
	@echo "Workloads:"
	@echo "  scan          Memory scan"
	@echo "  vecadd        Vector addition"
	@echo "  stride        Stride scan"
	@echo "  pointer       Pointer chase"
	@echo "  benchmark     Comparative benchmark"
	@echo ""
	@echo "Experiments:"
	@echo "  scaling       Dataset scaling"
	@echo "  strides       Stride testing"
	@echo "  wsweep        Working-set sweep"
	@echo ""
	@echo "Visualization:"
	@echo "  plot-scaling  Dataset scaling plots"
	@echo "  plot-stride   Generate stride scan plots"
	@echo "  plots         Generate all plots"
	@echo ""
	@echo "Configuration:"
	@echo "  NODE_ADDR     Remote node address (default: 127.0.0.1:9000)"

# =============================================================================
# Build & Development
# =============================================================================

build:
	$(CARGO) build

release:
	$(CARGO) build --release

check:
	$(CARGO) check

fmt:
	$(CARGO) fmt

lint:
	$(CARGO) clippy --all-targets --all-features

test:
	$(CARGO) test

clean:
	$(CARGO) clean

dev: fmt check test

# =============================================================================
# Runtime
# =============================================================================

host:
	$(HOST)

node:
	$(NODE)

# =============================================================================
# Workloads
# =============================================================================

scan:
	$(HOST) --node $(NODE_ADDR) scan

vecadd:
	$(HOST) --node $(NODE_ADDR) vec-add

stride:
	$(HOST) --node $(NODE_ADDR) stride-scan

pointer:
	$(HOST) --node $(NODE_ADDR) pointer-chase

benchmark:
	$(HOST) --node $(NODE_ADDR) benchmark

# =============================================================================
# Experiments
# =============================================================================

scaling:
	$(HOST) --node $(NODE_ADDR) experiment dataset-scaling

strides:
	$(HOST) --node $(NODE_ADDR) experiment stride-testing

wsweep:
	$(HOST) --node $(NODE_ADDR) experiment working-set-sweep --mode pointer
	$(HOST) --node $(NODE_ADDR) experiment working-set-sweep --mode random
	$(HOST) --node $(NODE_ADDR) experiment working-set-sweep --mode sequential

# =============================================================================
# Visualization
# =============================================================================

plot-scaling:
	$(PYTHON) viz/plot_dataset_scaling.py

plot-stride:
	$(PYTHON) viz/plot_stride_scan.py

plots: plot-scaling plot-stride
