# =============================================================================
# Aletheia Makefile - Memory Engine Benchmarking Framework
# =============================================================================
# 
# Build & Development:
#   build              - Build project in debug mode
#   release            - Build project in release mode (optimized)
#   check              - Run cargo check (fast compilation check)
#   test               - Run all tests
#   clean              - Remove all build artifacts
#   dev                - Quick development cycle (check + test)
#
# Run System:
#   host               - Run the host CLI client
#   node               - Run the memory engine server node
#
# Experiments:
#   scan               - Run scan workload on host
#   vecadd             - Run vector-add workload on host
#   stride             - Run stride-scan memory access pattern test
#   pointer            - Run pointer-chase latency measurement workload
#   wsweep             - Run working-set-sweep cache hierarchy measurement
#   benchmark          - Run full benchmark suite
#
# Visualization:
#   plot-scaling       - Generate dataset scaling performance plots
#   plot-stride        - Generate stride effect performance plots
#
# =============================================================================

.PHONY: help build release check test clean dev host node scan vecadd stride pointer wsweep benchmark plot-scaling plot-stride

# Default target
help:
	@echo "Aletheia Makefile - Available targets:"
	@echo ""
	@echo "Build & Development:"
	@echo "  make build         - Build project in debug mode"
	@echo "  make release       - Build project in release mode"
	@echo "  make check         - Run cargo check"
	@echo "  make test          - Run tests"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make dev           - Run check + test"
	@echo ""
	@echo "Run System:"
	@echo "  make host          - Run host binary"
	@echo "  make node          - Run node binary"
	@echo ""
	@echo "Experiments:"
	@echo "  make scan          - Run scan workload"
	@echo "  make vecadd        - Run vector-add workload"
	@echo "  make stride        - Run stride-scan experiment"
	@echo "  make pointer       - Run pointer-chase workload"
	@echo "  make wsweep        - Run working-set-sweep cache measurement"
	@echo "  make benchmark     - Run full benchmark suite"
	@echo ""
	@echo "Visualization:"
	@echo "  make plot-scaling  - Plot dataset scaling results"
	@echo "  make plot-stride   - Plot stride scan results"

# Build & Development
build:
	cargo build

release:
	cargo build --release

check:
	cargo check

test:
	cargo test

clean:
	cargo clean

dev: check test

# Run System
host:
	cargo run --bin aletheia-host --release

node:
	cargo run --bin aletheia-node --release

# Experiments
scan:
	cargo run --bin aletheia-host --release -- scan

vecadd:
	cargo run --bin aletheia-host --release -- vec-add

stride:
	cargo run --bin aletheia-host --release -- stride-scan

pointer:
	cargo run --bin aletheia-host --release -- pointer-chase

wsweep:
	cargo run --bin aletheia-host --release -- experiment working-set-sweep

benchmark:
	cargo run --bin aletheia-host --release -- benchmark

# Visualization
plot-scaling:
	python viz/plot_dataset_scaling.py

plot-stride:
	python viz/plot_stride_scan.py
