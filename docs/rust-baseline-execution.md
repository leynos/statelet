# Rust baseline execution record

This record tracks the baseline-hardening follow-up after spelling-only PR #80
merged. It records the current single delivery branch and separates its
measurements from historical results on PR #80 and scratch trees. The final PR
number, head, hosted checks, and any remaining external prerequisites are
updated after publication.

The follow-up branch is `rust-baseline-completion-20261001`, based on `main` at
`69203753e61cc5500d94a80cb700f021c32933ab` (#88). Its changes preserve the
merged spelling gate from #80, Dependabot policy from #81, step-level
`GITHUB_TOKEN` additions from #82, StateName work from #71, and the current
setup-rust pin from #88.

## Earlier PR #80 delivery history

| Item                       | Observed value                             |
| -------------------------- | ------------------------------------------ |
| Repository                 | `leynos/statelet`                          |
| Delivery PR                | #80, merged                                |
| Merge commit               | `cfbc15efa4bc31079dd0ceffa030d5e8bf9eeac1` |
| Original base              | `f53a239502c21586be9e3084b10446e4dc061d83` |
| Original head              | `d0252b39742f5b2e9f9b1248f527e1eecb71c292` |
| Fetched integration target | `9f501ff1cace9d47958ea9cc997cb0f4208727b1` |
| Rebased local head         | `c8cc0a8410a9d53708b6233cbf9c61395ee0d6b6` |
| Published PR head          | `d0252b39742f5b2e9f9b1248f527e1eecb71c292` |

*Table 1: Historical PR #80 identities; not evidence for this follow-up.*

The original two-commit series is exactly `f53a239..d0252b3`, with no merge
commits. `git range-diff` reports both commits patch-identical after replay onto
`9f501ff`. The target contains #37 and #79, including part of the build
standard. No tracked or ambient Git attributes select Weave on the inspected
files; the replay used Git's `zdiff3` text merge and had no conflicts. Recovery
refs are retained under `refs/recovery/statelet-rust-baseline-20260929/`.

The repository uses a bare primary checkout. `git donkey` rejected its worktree
layout, so the delivery checkout was created with `git worktree add`. The
supervisor's checkout is read-only during integration.

The requested Terra High Journeyman model/role was not available in the agent
catalogue. The supervisor assigned an explicit Journeyman replacement for
integration and bounded review batches; this run does not claim the unavailable
role was invoked. CodeGraph MCP is likewise unavailable.

After a SHA precheck at `d0252b3`, the authenticated `leynos` account used
GitHub's official branch-rename endpoint to rename the remote head. GitHub's
documented behaviour closed PR #80 because the renamed branch was its head. The
branch was renamed back, and the same PR was reopened as a draft. GitHub
read-back again shows the original head name and commit. No new PR or merge was
created. The delivery branch name will remain unchanged to preserve this PR.

## Policy and compatibility

The policy snapshot for this run is Concordat
`8a4a1faba1290687c0b6b221e1fc96439ba3ba43`, which contains the merged spelling
package from #223. The executable package files are present. The selected rule
versions are:

| Rule package                    | Version |
| ------------------------------- | ------- |
| `rust-build-defaults`           | 0.1.1   |
| `rust-makefile-baseline`        | 0.3.2   |
| `markdown-formatting-baseline`  | 0.2.0   |
| `main-owned-codescene-coverage` | 0.3.0   |
| `spelling-config-baseline`      | 0.1.0   |
| `whitaker-provisioning`         | 0.1.0   |
| `dependabot-update-shape`       | 0.1.0   |

*Table 2: Frozen Concordat rule-package versions for this run.*

The repository has one tracked `Cargo.toml`: one Rust 2024 library package and
its integration tests, with no discovered nested or excluded Cargo manifest.
The pinned repository toolchain is `nightly-2026-09-13`; its declaration now
has Clippy, rustfmt, LLVM tools, Cranelift and rust-analyzer. The earlier
`nightly-2026-05-28` observation belongs to a pre-merge PR #80 lint log and is
historical only. CodeGraph MCP was unavailable, so code reconnaissance used
Leta where possible and direct source inspection.

The tracked dependency ecosystems are GitHub Actions at `/`, Cargo at `/`, and
rust-toolchain at `/`; there are no local action manifests or nested package
roots. The frozen Concordat `dependabot-update-shape` rule is v0.1.0. The
current file inherited from #81 has daily schedules and a final minor/patch
group in all three entries, including `rust-toolchain`. The earlier intake note
that proposed omitting that group was based on an unverified
SemVer-applicability assumption. The selected rule's measured result is
recorded below; its outcome governs this consumer rather than that earlier
assumption. The file's SHA-256 is
`31fd58c055de73db4b6b4e315b880a7cc0926460839e3eacae8b014c225caf2e`.

The authenticated `leynos` API read initially returned 28 repository labels,
including the four dependency/channel labels used by those entries. Its
paginated snapshot is `/tmp/statelet-rust-baseline-pr80-labels-20260929.json`
(SHA-256 `5903328e9275bfc4bcf7ef3947356faacc98fa8eee56f28ed0d25008757b31fd`).
The frozen Concordat checkout contains only a priority-label model, not the
prompt's full severity/theme/dependency/channel model. The approved supplement
recorded in the shared Ortho Config baseline selects Netsuke main
`924cb215d3841048dbf6649767a510d2a5dfb1b7` `live-labels.json` (SHA-256
`bdd8fa7d9da6e7ef6d2c46a1db2cddff0e803b95797bcae55cad26c18d513133`), whose 31
descriptions fit GitHub's 100-character limit. After an authenticated `leynos`
identity check, the supervisor created `Windows`, `codex`, and `epic`, and
updated only the `Issue`, `Roadmap`, and `performance` descriptions. A fresh
API readback matches all 31 source names, colours, and descriptions, with no
extra or missing labels. The readback is
`/tmp/statelet-rust-baseline-pr80-labels-after-20260929.json` (SHA-256
`7c3bde9aed45e043c85e7cfd16244be273df7ac5dd355fa49405f5db2573d2cf`).

The v0.1.3 builder resolved to source commit
`c8a4f95d7cf7f6a1b7517f2775d122d47d5721eb`. Its live shared base had SHA-256
`d67b4110813615a4eda3e8962e898466191e4af25b2e28baedcbab348696aeac`; the local
overlay had SHA-256
`28749d8f7e60fb35d33d8a76e981f1aa77a7b122688281735e821e5cda3633ae`, and the
generated `typos.toml` had SHA-256
`da1ec11757e5305ef6d5c5d6cfe2548c64e1eaaae06bc9f0f9a175419d4b1703`. These are
the recorded generation inputs and output. The shared-base cache files are
ignored by Git; the live shared dictionary remains a mutable input, not an
immutable release dependency. The generation command was:

```sh
GIT_CONFIG_COUNT=0 UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools \
  uv tool run --python 3.14 \
  --from 'git+https://github.com/leynos/typos-config-builder.git@v0.1.3' \
  typos-config-builder --repository .
```

The selected Concordat packages have no declared `disallowed-methods` list. The
approved supplement recorded in the shared Ortho Config execution record
selects frozen Netsuke main `924cb215d3841048dbf6649767a510d2a5dfb1b7`. Its
`clippy_toml.content` has SHA-256
`2972c4a9261c63ab9abbd8230dde19af8430492b054dcb9c3521972eaa7e6c41` and lists
`std::env::{var,var_os,vars,vars_os,set_var,remove_var}` plus
`std::env::set_current_dir`. Only that method list and its diagnostic reasons
are approved here. The source's backlog-management `#[expect]` advice conflicts
with this task's fixes-first rule and will not be copied. The first scratch
zero-finding result did not include this list; the subsequent scratch
measurement applied it and found no configured lint findings. The final branch
measurement remains separate and is reported below.

For the shared-actions consumer repin, the selected `whitaker-provisioning`
package allows the exact #522 merge commit
`6dea5677a84fec60ca51b07202570e3af12ffdb4`. A later shared-actions main commit
is outside that frozen allow-list, so this PR will use the listed pin after its
action contents are independently verified. The old Statelet pin
`4fb8eb7ad52454678a0662865d81d3cd17aa6e0e` and the CV-005 minimum `a5765019`
are ancestors of `6dea5677` in the local shared-actions Git graph. The exact
`6dea5677` action was then inspected: installer version defaults to 0.2.9 with
a floor, `suite-version` is refused, the suite is rolling, both Cranelift and
ordinary paths pass `--no-source-fallback`, and installer status and
stdout/stderr fallback reports are binding. Published assets, digests and
Windows executable/PATH handling are present. The setup-rust, coverage,
CodeScene upload and mdtablefix action manifests are byte-identical to the old
pin. Consumer-level CI and gate behaviour still require proof.

The rolling Whitaker manifest identifies a suite Git revision and toolchain,
but the selected installer action logs only the toolchain and prebuilt/source
route. Its cache key does not include the rolling suite revision, and preflight
and installation fetch the manifest separately. A manifest snapshot alongside a
gate run records the observed live input; it cannot prove the exact installed
suite revision if a republish races the run. Evidence will distinguish those
facts and will not pin the rolling suite.

## Inspected consumer references

All nine exemplar PRs were read back as merged through the live GitHub API and
their changed artefacts inspected at the resulting merge commits. They are
examples of a consumer contract, not policy revisions to copy wholesale.

| Reference                                                                          | Merge commit                               | Inspected contract or artefact                                                                         |
| ---------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| [netsuke #733](https://github.com/leynos/netsuke/pull/733)                         | `00f48f77588016d64a056a220b93f46f397f38dc` | `.cargo/config.toml` frontend/Linux `mold` and `scripts/*build-tools.sh`; Cranelift exception is local |
| [netsuke #793](https://github.com/leynos/netsuke/pull/793)                         | `4af7b348aa09de5a25fbd2d4f9c396999baedbc8` | `coverage-main.yml` installation before suite and `nextest_lane_rules.py` reachability                 |
| [fingermouse #75](https://github.com/leynos/fingermouse/pull/75)                   | `800e99eb72ae9dfe3ff839a9c880956a80649cb2` | `coverage-main.yml` publisher and `codescene_publisher_rules.py`                                       |
| [spycatcher-harness #124](https://github.com/leynos/spycatcher-harness/pull/124)   | `34541c78053c83e11c7a0fd278a63b7ae3922912` | `tests/cv005/{publisher,rules}.rs` token and PR-lane constraints                                       |
| [mriya #92](https://github.com/leynos/mriya/pull/92)                               | `24087b6eb00dade0493d3fcacc21f02fa9babb15` | `coverage-main.yml` and `codescene_environment_rules.py` placement                                     |
| [frankie #118](https://github.com/leynos/frankie/pull/118)                         | `db39c62c28d0407c3d54effc665340fd426d9f73` | Direct-tool `Makefile`, Markdown config, and CI mdtablefix installer                                   |
| [odw-lint #18](https://github.com/leynos/odw-lint/pull/18)                         | `c40d06891e7e00b4758c07438f022fa16397126a` | `markdownlint.yml` action glob and canonical config                                                    |
| [netsuke #617](https://github.com/leynos/netsuke/pull/617)                         | `79de545399e652f85f975571c8bd6d45c589b77d` | Earlier shared Whitaker consumer in `ci.yml`; installer contract is superseded by #522                 |
| [typos-config-builder #96](https://github.com/leynos/typos-config-builder/pull/96) | `74d201bf89b1154561ca57b5b7571f44f306ae77` | `docs/agents-md-spelling.md` block source; v0.1.3 was verified separately                              |

*Table 3: Live merged exemplar provenance and inspected entrypoints.*

## Measurements on this follow-up

The earlier rebased head c8cc0a8 passed the then-existing gate set, including
40 tests, one doctest, and 126 workflow-contract tests. Those results are
historical: they predate this branch's final policy, workflow changes, and
contracts. Their logs are under
/tmp/statelet-rust-baseline/pr80-c8cc0a8-main-9f501-run2/.

Detached scratch measurements applied the complete Clippy, Rust, and rustdoc
tables, the four complexity thresholds, and the seven approved environment
methods to the single crate. Clippy and rustdoc passed with no source findings
or skipped targets. Cargo emitted two manifest warnings. These were scratch
measurements on the pre-merge branch, not results for the follow-up branch.

The selected Concordat packages were audited with Concordat 0.1.0 from source
commit `8a4a1faba1290687c0b6b221e1fc96439ba3ba43`. The command was:

```sh
UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools uv tool run --python 3.14 \
  --from 'git+https://github.com/leynos/concordat.git@8a4a1faba1290687c0b6b221e1fc96439ba3ba43' \
  concordat artefact rule run "$rule" --repo "$PWD" --format json
```

Each selected package returned `compliant` with no findings:
`rust-build-defaults` 0.1.1, `rust-makefile-baseline` 0.3.2,
`markdown-formatting-baseline` 0.2.0, `main-owned-codescene-coverage` 0.3.0,
`spelling-config-baseline` 0.1.0, `whitaker-provisioning` 0.1.0, and
`dependabot-update-shape` 0.1.0. The runner used `makeutil` 0.1.0 from source
revision `f405b89e4a903718`, SHA-256
`6ac22335da4d1757d5248a89871ba0d2718babce34779989a28eb5e74f504abb`. The
Markdown pass followed two measured repairs: replacing an unsupported
`npm exec` route with the direct pinned `markdownlint-cli2` executable and
rewriting the Whitaker recipe as a directly auditable command. These results
must be repeated after the final formatter pass.

The first integrated gate run was against tree
`d027503818fb33300c34a41443def9f4db4e94d4` on 2026-10-01. `make fmt`,
`make check-fmt`, `make lint-clippy`, `make typecheck`, `make test`,
`make nixie`, and `GIT_CONFIG_COUNT=0 make audit` passed. The test run reported
143/143 Nextest tests and one doctest; the audit loaded 1,277 advisories. The
Clippy/rustdoc run had no source findings and retained two Cargo manifest
warnings (`manual_readme` and `redundant_homepage`). The seven Concordat audits
listed above also passed on that tree.

That run found 146 spelling findings. They were triaged by exact source: linker
identifiers, Cargo and GitHub API spellings, historical evidence in the
byte-exact ExecPlan, and the exact OpenTofu fixture were narrowly accounted
for; avoidable prose and test identifiers were repaired. The historical
`docs/execplans/1-1-3-define-state-name-consumption-question.md` remains
byte-identical (SHA-256
`c9a0e2c157e401f88aabd70d68c4e5f53f49d92183b8e79551aa8aec52b35af5`). The
spelling overlay has no file exclusions or blanket inline-code exemption, and
negative controls retain both quoted and unquoted ordinary prose. The builder
v0.1.3 regenerated `typos.toml`; its final check remains pending.

`make test-workflow-contracts` reported 207 passed and four failures, all in
test expectations: the Whitaker leaf-command selector, the distinct
`setup-rust` action pin, the coverage-format mutation needle, and GNU `xargs`
exit-code mapping. Those four contract tests have been corrected. The Markdown
selection test passed in that run: a non-ignored untracked formatting defect
failed `check-fmt`, `fmt` repaired it, the check passed, and ignored generated
Markdown stayed untouched. `make markdownlint` stopped at its spelling
prerequisite, so that run did not establish a separate Markdown-lint result.
The corrected spelling, workflow-contract, Markdown-lint, and all other
non-mutating checks must run again after the final formatter.

The current branch is based on main commit
69203753e61cc5500d94a80cb700f021c32933ab. The local binary-only installation
attempt used:

```sh
cargo binstall --no-confirm --force --strategies crate-meta-data,quick-install \
  --version 0.2.9 whitaker-installer
```

On 2026-10-01 it exited 94 because the published Linux archive did not contain
the required `whitaker-installer` executable; the installer reported that its
Cargo fallback was disabled. No source build ran. The ambient installer is
0.2.7 and does not satisfy the selected action's 0.2.9 floor. The approved
binary-only route is therefore externally blocked, and neither a suite revision
nor an installer-managed toolchain can be claimed for this branch. The Whitaker
suite and composite `make lint` remain unrun. The first integrated
`make lint-clippy` run passed separately; the next final run must repeat it.
`mdtablefix` 0.6.0 and `markdownlint-cli2` 0.23.2 are installed for their local
routes.

The direct Markdown tools and CI consumer contracts are present. The local
formatter contract exercises a non-ignored untracked Markdown defect, checks
that check-fmt rejects it, verifies fmt repairs it, and checks that an ignored
generated file remains untouched. Its contract also checks that both
markdownlint-cli2 and the `make markdownlint` file walk exclude `target/`. It
injects a controlled Markdown-linter failure through both Make routes and
checks that each returns nonzero. The final workflow-contract run must execute
these checks on this branch. Local markdownlint uses npm's exact
markdownlint-cli2 0.23.2 package, matching the package bundled by the pinned CI
action; mdtablefix 0.6.0 has a pinned Cargo install target and matching shared
CI action.

## Five onboarding dispositions

| Onboarding                | Implemented contract                                                | Remaining evidence                                    |
| ------------------------- | ------------------------------------------------------------------- | ----------------------------------------------------- |
| Development build default | Cranelift, -Zthreads=8, Linux linker, pinned tool preflight         | Local gates passed; hosted CI pending                 |
| CV-005                    | PR ratchet and protected main publisher contracts                   | Hosted CI and live secret placement remain unverified |
| Markdown                  | Direct pinned tools, CI provisioning, formatting selection contract | Local gates and Concordat audit passed                |
| install-whitaker          | Approved shared action pin and ordered Make composite gate          | 0.2.9 archive lacks its binary; suite unrun           |
| Typos builder             | v0.1.3 full-scope gate and narrow local overlay                     | Spelling gate and Concordat audit passed              |

*Table 4: Consumer implementation and outstanding evidence.*

The selected Markdown policy retains MD010.code_blocks = false for hard-tab
handling and MD013.code_block_line_length = 120 for long code lines. These
settings serve different rules and both remain in the consumer configuration.

## Known policy and hosted-state limits

The first intake analysis treated the `rust-toolchain` group as a SemVer
applicability problem. The current inherited configuration includes the
required group, and the selected rule's audit result above supersedes that
analysis. No local exception or weaker consumer policy was added.

Concordat's selected rule packages do not supply a disallowed_methods list. The
seven-method supplement is sourced from the approved Netsuke snapshot and its
exact revision and content hash are recorded above. The frozen
whitaker-provisioning rule approves shared-actions commit
6dea5677a84fec60ca51b07202570e3af12ffdb4; the consumer uses that exact commit.
The rolling suite is not pinned. The shared action logs its installer-managed
toolchain but not the exact installed suite SHA, so that field must remain
unclaimed if the run does not expose it.

GitHub's codescene environment was observed with a deployment policy that
allows main only. The GitHub integration returned HTTP 403 when asked for
repository or environment secret metadata. The move of CS_ACCESS_TOKEN into
that environment and removal of repository-level exposure therefore remain
unverified. A protected main-only publisher run after merge is also pending;
report generation, baseline persistence, upload, and intentional token-absent
skip are distinct outcomes.

## Final local verification before publication

The final code and workflow tree measured before this record refresh was
`1a5ad6a3b0fee79dbd6a10d59326ddac5316fc5b`, based on main at
`69203753e61cc5500d94a80cb700f021c32933ab`. The formatter and every local
non-mutating gate below passed on that unchanged code tree:

| Command                         | Result                                                   | Evidence                                                                    |
| ------------------------------- | -------------------------------------------------------- | --------------------------------------------------------------------------- |
| `make fmt`                      | Passed; 30 Markdown files linted with no issues          | `/tmp/final4-format-rust-baseline-completion-20261001.out`                  |
| `make check-fmt`                | Passed; 29 Markdown files unchanged                      | `/tmp/final4-check-fmt-rust-baseline-completion-20261001.out`               |
| `make spelling`                 | Passed; generated `typos.toml` remained byte-stable      | `/tmp/final4-spelling-rust-baseline-completion-20261001.out`                |
| `make lint-clippy`              | Passed; Rustdoc and Clippy; two manifest warnings remain | `/tmp/final4-lint-clippy-rust-baseline-completion-20261001.out`             |
| `make typecheck`                | Passed for all targets and features                      | `/tmp/final4-typecheck-rust-baseline-completion-20261001.out`               |
| `make test`                     | Passed; 143 tests and one doctest                        | `/tmp/final4-test-rust-baseline-completion-20261001.out`                    |
| `make test-workflow-contracts`  | Passed; 213 tests                                        | `/tmp/final4-test-workflow-contracts-rust-baseline-completion-20261001.out` |
| `make markdownlint`             | Passed, including its spelling prerequisite              | `/tmp/final4-markdownlint-rust-baseline-completion-20261001.out`            |
| `make nixie`                    | Passed; all diagrams validated                           | `/tmp/final4-nixie-rust-baseline-completion-20261001.out`                   |
| `GIT_CONFIG_COUNT=0 make audit` | Passed; 1,277 advisories loaded                          | `/tmp/final4-audit-rust-baseline-completion-20261001.out`                   |
| `mbake validate Makefile`       | Passed                                                   | `/tmp/final4-mbake-validate-rust-baseline-completion-20261001.out`          |

The seven selected Concordat audits also passed: `rust-build-defaults` 0.1.1,
`rust-makefile-baseline` 0.3.2, `markdown-formatting-baseline` 0.2.0,
`main-owned-codescene-coverage` 0.3.0, `spelling-config-baseline` 0.1.0,
`whitaker-provisioning` 0.1.0, and `dependabot-update-shape` 0.1.0. They used
Concordat 0.1.0 at `8a4a1faba1290687c0b6b221e1fc96439ba3ba43` and makeutil
0.1.0 at `f405b89e4a903718`; individual logs are under
`/tmp/final4-audit-*-rust-baseline-completion-20261001.out`.

The build checks observed `nightly-2026-09-13`. A controlled Clippy probe
confirmed that `.expect()` in plain `#[test]` and `#[rstest]` bodies is
accepted, while fixture and ordinary helper calls are denied by `expect_used`.
`rstest-bdd` is not a dependency or test macro in this consumer, so its
recognition behaviour was not measured. The scratch probe was removed.

The approved binary-only Whitaker install attempt still exits 94 because the
published Linux 0.2.9 archive lacks `whitaker-installer`; source fallback was
disabled. The installer-managed toolchain and rolling-suite revision therefore
remain unobserved, and neither the Whitaker suite nor composite `make lint` has
run. Hosted CI has not run yet. The Codescene environment is main-only, but
GitHub returned HTTP 403 for secret metadata, so placement of `CS_ACCESS_TOKEN`
in that environment and removal of repository-level exposure remain unverified.
The protected-main publisher's post-merge outcomes also remain pending. The
branch is prepared for publication as a draft PR. The PR number and hosted
results will be recorded after publication.
