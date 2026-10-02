.PHONY: help all clean test build release coverage lint fmt check-fmt \
	markdownlint nixie audit rust-audit test-workflow-contracts spelling \
	install-build-tools check-build-tools check-nextest install-mdtablefix \
	install-markdownlint \
	lint-clippy lint-whitaker

SHELL := bash

# The two lint leaves must remain ordered even when callers use `make -j`.
.NOTPARALLEL: lint

BUILD_TOOLS_PREFIX ?= $(HOME)/.local
export BUILD_TOOLS_PREFIX
export PATH := $(BUILD_TOOLS_PREFIX)/bin:$(PATH)


TARGET ?= libstatelet.rlib

CARGO ?= cargo
BUILD_JOBS ?=
RUST_FLAGS ?=
RUST_FLAGS := -D warnings $(RUST_FLAGS)
# The build standard: every `rustflags` source in `.cargo/config.toml` carries
# the parallel frontend, and the Linux source adds the linker. Assigning `RUSTFLAGS`
# replaces those sources outright, so the gate targets restate the flags here.
# Coverage and release builds deliberately take neither.
STANDARD_THREADS_FLAG ?= -Zthreads=8
STANDARD_MOLD_FLAG ?= -Clink-arg=-fuse-ld=mold
BUILD_HOST_OS := $(shell uname -s)
# The linker is added only when the machine doing the build is Linux (only Make can
# tell whether it has the linker) and the compilation target is Linux too, which is
# the host unless `CARGO_BUILD_TARGET` names another triple.
STANDARD_TARGET_IS_LINUX = $(if $(CARGO_BUILD_TARGET),$(or $(findstring -linux-,$(CARGO_BUILD_TARGET)),$(filter host-tuple,$(CARGO_BUILD_TARGET))),yes)
STANDARD_RUSTFLAGS = $(STANDARD_THREADS_FLAG)$(if $(filter Linux,$(BUILD_HOST_OS)),$(if $(STANDARD_TARGET_IS_LINUX), $(STANDARD_MOLD_FLAG)))
# Release builds take neither flag: assigning `RUSTFLAGS`, even to an empty
# inherited value, displaces every `rustflags` source in the configuration.
RELEASE_RUSTFLAGS = RUSTFLAGS="$${RUSTFLAGS-}"
# Debug builds keep a caller's exported flags and add the standard ones,
# since an inherited `RUSTFLAGS` would otherwise displace the configuration.
DEBUG_RUSTFLAGS = RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(STANDARD_RUSTFLAGS)"
RUSTDOC_FLAGS ?= --cfg docsrs -D warnings
CARGO_FLAGS ?= --all-targets --all-features
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
COVERAGE_LINKER_FLAGS ?= -fuse-ld=lld
COVERAGE_RUST_FLAGS ?= $(RUST_FLAGS) -C link-arg=$(COVERAGE_LINKER_FLAGS)
MARKDOWNLINT_VERSION ?= 0.23.2
MDLINT ?= markdownlint-cli2
MDTABLEFIX_VERSION ?= 0.6.0
# `make fmt` and `make check-fmt` call mdtablefix directly. `--git` selects the
# Markdown files Git tracks and `--include-untracked` adds the untracked files
# Git does not ignore, so a new document is formatted before it is staged.
# Both modes need mdtablefix 0.6.0 or later; CI pins the version at the
# install-mdtablefix step.
MDTABLEFIX ?= mdtablefix
MDTABLEFIX_SELECT = --git --include-untracked
MDTABLEFIX_RULES = --wrap --renumber --breaks --ellipsis --fences
NIXIE ?= nixie
WHITAKER ?= whitaker
WHITAKER_PACKAGES ?= --workspace
UV ?= uv
UV_ENV = UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools
TYPOS_CONFIG_BUILDER_VERSION ?= v0.1.3
TYPOS_CONFIG_BUILDER = $(UV_ENV) $(UV) tool run --python 3.14 --from \
	"git+https://github.com/leynos/typos-config-builder.git@$(TYPOS_CONFIG_BUILDER_VERSION)" \
	typos-config-builder

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: ## Perform a comprehensive check of code
	+$(MAKE) check-fmt
	+$(MAKE) lint
	+$(MAKE) test
	+$(MAKE) spelling

install-build-tools: ## Install the pinned development build tools
	scripts/install-build-tools.sh

install-mdtablefix: check-build-tools ## Install the pinned Markdown table formatter
	$(CARGO) install --locked --version $(MDTABLEFIX_VERSION) mdtablefix

install-markdownlint: ## Install the pinned Markdown linter in BUILD_TOOLS_PREFIX
	npm install --global --prefix "$(BUILD_TOOLS_PREFIX)" "markdownlint-cli2@$(MARKDOWNLINT_VERSION)"

check-build-tools: ## Check the development build tools are installed
	scripts/check-build-tools.sh

check-nextest: check-build-tools ## Check the binary-installed test runner
	@if $(CARGO) nextest --version >/dev/null 2>&1; then \
		:; \
	else \
		printf '%s\n' 'cargo-nextest is required by make test.' \
			'Install the published binary with: cargo binstall --no-confirm --strategies crate-meta-data,quick-install cargo-nextest' >&2; \
		exit 1; \
	fi

clean: ## Remove build artefacts
	$(CARGO) clean

test: check-nextest ## Run tests with warnings treated as errors
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) nextest run $(TEST_FLAGS) $(BUILD_JOBS)
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) test --workspace --doc --all-features $(BUILD_JOBS)

test-workflow-contracts: ## Validate the workflow contracts (mutation testing, CodeScene coverage)
	uv run --with 'pytest>=8' --with 'pyyaml>=6' pytest tests/workflow_contracts -q

target/%/$(TARGET): ## Build binary in debug or release mode
	$(if $(findstring release,$(@)),$(RELEASE_RUSTFLAGS),$(DEBUG_RUSTFLAGS)) $(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release)

# Only the debug artefact needs the development linker and frontend tools.
# This order-only prerequisite runs before Cargo even under `make -j`, without
# making release builds depend on the linker.
target/debug/$(TARGET): | check-build-tools

coverage: ## Generate lcov coverage with lld for llvm-tools compatibility
	@echo "coverage linker flags: $(COVERAGE_LINKER_FLAGS)"
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang RUSTFLAGS="$(COVERAGE_RUST_FLAGS)" \
		CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm \
		CFLAGS="$(COVERAGE_LINKER_FLAGS)" LDFLAGS="$(COVERAGE_LINKER_FLAGS)" \
		$(CARGO) llvm-cov --lcov --output-path lcov.info $(TEST_FLAGS)

lint: ## Run sequential Clippy and Whitaker checks with warnings denied
	+$(MAKE) lint-clippy
	+$(MAKE) lint-whitaker

lint-clippy: check-build-tools ## Run rustdoc and Clippy with warnings denied
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --workspace --no-deps
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) clippy $(CLIPPY_FLAGS)

lint-whitaker: check-build-tools ## Run the rolling Whitaker Dylint suite without repository RUSTFLAGS
	RUSTFLAGS= $(WHITAKER) --all $(WHITAKER_PACKAGES) -- $(CARGO_FLAGS)

typecheck: check-build-tools ## Type-check without building
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) fmt --all
	$(MDTABLEFIX) --in-place $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)
	$(MDLINT) --fix "**/*.md"

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check
	$(MDTABLEFIX) --check $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)

markdownlint: spelling ## Lint Markdown files and enforce spelling
	git ls-files --cached --others --exclude-standard -z -- ':(glob)**/*.md' | \
		xargs -0 $(MDLINT)

spelling: ## Enforce en-GB-oxendict spelling and shared phrase corrections
	$(TYPOS_CONFIG_BUILDER) gate --repository . --scope all




nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

audit: rust-audit ## Audit dependencies for known vulnerabilities

rust-audit: ## Audit the Rust workspace for known vulnerabilities
	set -eo pipefail; \
	manifest_list=$$(mktemp); \
	trap 'rm -f "$$manifest_list"' EXIT; \
	printf "Audit metadata phase: deriving workspace manifests\n"; \
	$(CARGO) metadata --no-deps --format-version 1 | python3 -c 'import json, sys; metadata = json.load(sys.stdin); members = set(metadata["workspace_members"]); print(metadata["workspace_root"]); [print(package["manifest_path"]) for package in metadata["packages"] if package["id"] in members]' > "$$manifest_list"; \
	workspace_root=$$(sed -n '1p' "$$manifest_list"); \
	audit_flags=(); \
	for advisory in $$CARGO_AUDIT_IGNORES; do \
		audit_flags+=(--ignore "$$advisory"); \
	done; \
	printf "Auditing Rust workspace %s\n" "$$workspace_root"; \
	sed -n '2,$$p' "$$manifest_list" | while IFS= read -r manifest; do \
		manifest_dir=$$(dirname "$$manifest"); \
		printf "Workspace Rust manifest %s\n" "$$manifest_dir/Cargo.toml"; \
	done; \
	printf "Audit execution phase: running cargo audit\n"; \
	printf "Audit failures may indicate RustSec advisories, cargo metadata errors, or documented ignores that need CARGO_AUDIT_IGNORES entries.\n"; \
	(cd "$$workspace_root" && $(CARGO) audit "$${audit_flags[@]}")

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
