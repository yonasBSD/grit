# Convenience wrapper around the cargo/test workflows documented in AGENTS.md.
#
# Targets:
#   make build        - release build of the grit CLI
#   make debug        - debug build of the grit CLI
#   make test         - run the Rust unit/integration tests
#   make harness      - run the full upstream test harness (in-scope files)
#   make harness-t<N> - run one harness group, e.g. `make harness-t4`
#   make clippy       - lint all crates
#   make fmt          - format all crates
#   make clean        - remove build artifacts

CARGO ?= cargo

.PHONY: all build debug test harness clippy fmt clean

all: build

build:
	$(CARGO) build --release -p grit-git

debug:
	$(CARGO) build -p grit-git

test:
	$(CARGO) test --workspace

harness: build
	./scripts/run-tests.sh

harness-%: build
	./scripts/run-tests.sh $*

clippy:
	$(CARGO) clippy --workspace --all-targets

fmt:
	$(CARGO) fmt --all

clean:
	$(CARGO) clean
