.PHONY: help all clean test build release coverage lint fmt check-fmt \
	markdownlint nixie audit rust-audit test-workflow-contracts spelling \
	install-build-tools check-build-tools check-nextest install-mdtablefix \
	install-markdownlint \
	lint-clippy lint-whitaker lint-python typecheck typecheck-python typecheck-rust

SHELL := bash

# Compiler and Python gateways remain ordered even when callers use `make -j`.
.NOTPARALLEL: all lint typecheck test

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
MDTABLEFIX_VERSION ?= 0.6.1
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

# The CV-005 CodeScene contracts live in shared-actions and run from a full
# commit, so a fix is a pin bump. `.github/cv005.toml` holds this repository's
# only parameters.
CV005_CONTRACTS_REF ?= 88977798a5c3bae1549afb99642529488c665276
CV005_CONTRACTS = $(UV_ENV) $(UV) tool run --python 3.13 \
	--from 'git+https://github.com/leynos/shared-actions@$(CV005_CONTRACTS_REF)\#subdirectory=packages/cv005-contracts' \
	cv005-contracts

PYTHON_BASELINE ?= 3.14
PYLINT_VERSION ?= 4.0.9
PYTEST_VERSION ?= 9.0.2
TY_VERSION ?= 0.0.74
PYTHON_DEPENDENCIES = --with pytest==$(PYTEST_VERSION) --with 'pyyaml>=6'
DF12_PYTHON_LINTS_REF ?= 4cf41736cce2f7ba2778882a5c629c044568a0e5
DF12_PYTHON_LINTS = git+https://github.com/leynos/df12-python-lints.git@$(DF12_PYTHON_LINTS_REF)
DF12_PYLINT_MESSAGES = R9101,C9102,R9103,R9104,C9105,C9106,C9107,R9108,R9109,R9110,R9111,R9112,C9112
PYLINT = $(UV_ENV) $(UV) tool run --managed-python --python $(PYTHON_BASELINE) \
	--from 'pylint==$(PYLINT_VERSION)' --with '$(DF12_PYTHON_LINTS)' \
	$(PYTHON_DEPENDENCIES) pylint --load-plugins=df12_python_lints \
	--enable=$(DF12_PYLINT_MESSAGES)
TY = $(UV_ENV) $(UV) tool run --managed-python --python $(PYTHON_BASELINE) \
	--from ty==$(TY_VERSION) $(PYTHON_DEPENDENCIES) ty

# Share one recursive inventory across lint and typecheck. `.github` contains
# workflow and action modules; the other roots cover tests, scripts, and both
# common benchmark directory names. Generated and vendored trees are pruned.
PYTHON_SOURCE_ROOTS ?= .github tests scripts benches benchmarks
PYTHON_EXISTING_SOURCE_ROOTS = $(wildcard $(PYTHON_SOURCE_ROOTS))
PYTHON_PRUNED_DIRECTORIES = \
	-name .git -prune -o -name .venv -prune -o -name venv -prune -o \
	-name .uv-cache -prune -o -name .uv-tools -prune -o \
	-name target -prune -o -name vendor -prune -o -name node_modules -prune -o \
	-name __pycache__ -prune -o -name .pytest_cache -prune -o \
	-name .mypy_cache -prune -o -name .ruff_cache -prune -o
PYTHON_SOURCES = $(strip $(shell find $(PYTHON_EXISTING_SOURCE_ROOTS) \
	$(PYTHON_PRUNED_DIRECTORIES) -type f -name '*.py' -print | sort))
# Workflow contracts import sibling modules as top-level modules; pass all
# source roots so ty resolves the same import layout as pytest.
PYTHON_IMPORT_ROOTS = $(addprefix --extra-search-path ,$(PYTHON_EXISTING_SOURCE_ROOTS))
TYPOS_CONFIG_BUILDER_VERSION ?= v0.1.3
TYPOS_CONFIG_BUILDER = $(UV_ENV) $(UV) tool run --managed-python --python $(PYTHON_BASELINE) --from \
	"git+https://github.com/leynos/typos-config-builder.git@$(TYPOS_CONFIG_BUILDER_VERSION)" \
	typos-config-builder

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: ## Perform a comprehensive check of code
	+$(MAKE) check-fmt
	+$(MAKE) lint
	+$(MAKE) typecheck
	+$(MAKE) test
	+$(MAKE) spelling
	+$(MAKE) test-workflow-contracts

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
	+$(MAKE) test-workflow-contracts

test-workflow-contracts: ## Validate the workflow contracts (mutation testing, CodeScene coverage)
	$(CV005_CONTRACTS) check --repository .
	$(UV_ENV) $(UV) run --no-project --managed-python --python $(PYTHON_BASELINE) \
		$(PYTHON_DEPENDENCIES) pytest tests/workflow_contracts -q

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

lint: ## Run Rust and Python lint gateways with warnings denied
	+$(MAKE) lint-clippy
	+$(MAKE) lint-whitaker
	+$(MAKE) lint-python

lint-clippy: check-build-tools ## Run rustdoc and Clippy with warnings denied
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --workspace --no-deps
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) clippy $(CLIPPY_FLAGS)

lint-whitaker: check-build-tools ## Run the rolling Whitaker Dylint suite without repository RUSTFLAGS
	RUSTFLAGS= $(WHITAKER) --all $(WHITAKER_PACKAGES) -- $(CARGO_FLAGS)

typecheck: check-build-tools ## Type-check Python and Rust sources without building
	+$(MAKE) typecheck-python
	+$(MAKE) typecheck-rust

typecheck-python: ## Type-check all repository Python sources with ty
	@test -n "$(PYTHON_SOURCES)" || { echo 'typecheck-python: no Python sources found under $(PYTHON_SOURCE_ROOTS)' >&2; exit 2; }
	$(TY) check --python-version $(PYTHON_BASELINE) $(PYTHON_IMPORT_ROOTS) $(PYTHON_SOURCES)

typecheck-rust: check-build-tools ## Type-check Rust sources without building
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) check $(CARGO_FLAGS)

lint-python: ## Run Pylint and the pinned df12 house lints on every Python source
	@test -n "$(PYTHON_SOURCES)" || { echo 'lint-python: no Python sources found under $(PYTHON_SOURCE_ROOTS)' >&2; exit 2; }
	$(PYLINT) $(PYTHON_SOURCES)

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
	$(CARGO) metadata --no-deps --format-version 1 | $(UV_ENV) $(UV) run --no-project --managed-python --python $(PYTHON_BASELINE) python -c 'import json, sys; metadata = json.load(sys.stdin); members = set(metadata["workspace_members"]); print(metadata["workspace_root"]); [print(package["manifest_path"]) for package in metadata["packages"] if package["id"] in members]' > "$$manifest_list"; \
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
