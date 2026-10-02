# Rust baseline execution record

This record tracks the baseline-hardening follow-up after spelling-only PR #80
merged. It records the current single delivery branch and separates its
measurements from historical results on PR #80 and scratch trees. The final PR
number, head, hosted checks, and any remaining external prerequisites are
updated after publication.

The follow-up branch is `rust-baseline-completion-20261001`, rebased onto
`main` at `d83f693f61385a5ec3cd3ffb8eaddc59e33f6571` (#98). Its changes
preserve the merged spelling gate from #80, Dependabot policy from #81,
step-level `GITHUB_TOKEN` additions from #82, StateName work from #71, and the
current setup-rust pin from #88.

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
overlay now has SHA-256
`3065100e60b1def2d74905c92ce6f738ad14de157896d0d1d2ecd1d6b782deaf`, and the
generated `typos.toml` has SHA-256
`748f461abca1bdb97d9b66387cb2f4ae1ad22e25972e24888422e66d8e9890a0`. These are
the current recorded generation inputs and output. The shared-base cache files
are ignored by Git; the live shared dictionary remains a mutable input, not an
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
rewriting the Whitaker recipe as a directly auditable command. Those results
were on the pre-rebase tree; the selected packages were rerun below after the
final formatter.

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
v0.1.3 regenerated `typos.toml`; the full-scope spelling gate passed again on
the rebased candidate after its final formatter.

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
Cargo fallback was disabled. No source build ran. This was a local probe of the
binary package, not a failure of the shared action. Statelet PR #94
subsequently merged the approved `install-whitaker` action route at
`6dea5677a84fec60ca51b07202570e3af12ffdb4`, with `cranelift: true` and no
manual installer script. Hosted run
[36858219212](https://github.com/leynos/statelet/actions/runs/36858219212)
passed its Install Whitaker, Lint, and workflow-contract steps on main commit
`84235c6d8d3d5dc2880a97543385bda51d44faa2`. The action's default installer is
0.2.9 and the suite remains rolling. The retrieved run metadata does not expose
the exact installer-managed toolchain string or installed suite revision; those
values remain unclaimed. This branch's rebased sequential Make lint gate still
requires candidate-head CI evidence. The ambient installer is 0.2.7 and was not
used. `mdtablefix` 0.6.0 and `markdownlint-cli2` 0.23.2 are installed for their
local routes.

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

| Onboarding                | Implemented contract                                                | Remaining evidence                                        |
| ------------------------- | ------------------------------------------------------------------- | --------------------------------------------------------- |
| Development build default | Cranelift, -Zthreads=8, Linux linker, pinned tool preflight         | Main CI passed; candidate-head CI pending                 |
| CV-005                    | PR ratchet and protected main publisher contracts                   | Candidate CI and live secret placement remain unverified  |
| Markdown                  | Direct pinned tools, CI provisioning, formatting selection contract | Candidate local gates and audit passed; hosted CI pending |
| install-whitaker          | Approved shared action pin and ordered Make composite gate          | Main CI action and lint passed; candidate CI pending      |
| Typos builder             | v0.1.3 full-scope gate and narrow local overlay                     | Spelling gate and Concordat audit passed                  |

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

## Historical local verification before rebase

The code and workflow tree measured before the 2026-10-02 rebase was
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

The seven selected Concordat audits also passed on that earlier tree:
`rust-build-defaults` 0.1.1, `rust-makefile-baseline` 0.3.2,
`markdown-formatting-baseline` 0.2.0, `main-owned-codescene-coverage` 0.3.0,
`spelling-config-baseline` 0.1.0, `whitaker-provisioning` 0.1.0, and
`dependabot-update-shape` 0.1.0. They used Concordat 0.1.0 at
`8a4a1faba1290687c0b6b221e1fc96439ba3ba43` and makeutil 0.1.0 at
`f405b89e4a903718`; individual logs are under
`/tmp/final4-audit-*-rust-baseline-completion-20261001.out`.

The build checks observed `nightly-2026-09-13`. A controlled Clippy probe
confirmed that `.expect()` in plain `#[test]` and `#[rstest]` bodies is
accepted, while fixture and ordinary helper calls are denied by `expect_used`.
`rstest-bdd` is not a dependency or test macro in this consumer, so its
recognition behaviour was not measured. The scratch probe was removed.

The exact shared `install-whitaker` action is now active on main, and its
hosted Install Whitaker and Lint steps passed in run 36858219212. That run's
available metadata does not disclose the installed suite SHA or
installer-managed toolchain string. The candidate branch's sequential composite
Make lint and full validation still require fresh CI after publication. The
Codescene environment is main-only, but GitHub returned HTTP 403 for secret
metadata, so placement of `CS_ACCESS_TOKEN` in that environment and removal of
repository-level exposure remain unverified. The protected-main publisher's
post-merge outcomes also remain pending. A draft description is prepared, but
no PR exists because the remote branch has not been published. The command-line
GitHub OAuth token lacks the `workflow` scope required to push changed workflow
files. Publication is assigned to the authenticated `leynos` GitHub connector
after current-head checks.

## Rebase and current candidate state

The delivery branch was rebased from base
`69203753e61cc5500d94a80cb700f021c32933ab` onto the live main commit
`84235c6d8d3d5dc2880a97543385bda51d44faa2` on 2026-10-02. Recovery refs
preserve the old head, old base, and target under
`refs/recovery/statelet-rust-baseline-20261002/`. The replay had conflicts only
in `ci.yml` and `coverage-main.yml`. The resolution retains the merged #94
Whitaker action and `cranelift: true`, plus #91's current coverage-generation
pin. `git diff --check` passed after replay.

The first workflow-contract run against the rebased tree exposed stale
expectations for the #91 coverage action SHA and the #94 YAML boolean/input
shape. Those contracts were corrected to validate the live action pins and to
allow omitted `RUSTFLAGS` while rejecting non-empty injection. The corrected
`make test-workflow-contracts` passed 212 tests in 8.48 seconds on the rebased
candidate; its log is `/tmp/statelet-post-rebase-contracts-fixed-20261002.out`.
The full post-rebase validation sequence, including another formatter pass, is
recorded below. Historical results above remain tied to their earlier trees.

The local `make install-markdownlint` target completed successfully from the
Makefile's pinned `markdownlint-cli2@0.23.2` package. The selected executable
reports version 0.23.2; its install log is
`/tmp/statelet-install-markdownlint-rust-baseline-completion-20261001.out`. The
ambient `whitaker-installer` reports 0.2.7 and `whitaker --version` reports
`cargo-dylint 6.0.1`; neither is accepted as the suite's installer or rolling
revision evidence and neither was used for the suite gate. Main's merged PR #94
run 36858219212 used the approved action and passed Install Whitaker, Lint, and
Workflow contract tests at PR head `8028ee38e276d2f99c3593e3468b334d42e65c29`.

The canonical spelling block was checked after formatting against the pinned
builder v0.1.3 source file at commit
`c8a4f95d7cf7f6a1b7517f2775d122d47d5721eb`. The start and end markers each
occur once on separate lines in `AGENTS.md`, and the content between them
matches the source after whitespace normalization.

## Post-rebase final validation

These results apply to the formatted candidate after removing the redundant
Cargo `homepage` and explicit `readme` keys that caused two manifest warnings.
The checks ran sequentially after the shared Cargo cache lock cleared.

| Command                                | Result                                                       | Evidence                                                                         |
| -------------------------------------- | ------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| `make fmt`                             | Passed; Rust and Markdown formatting applied                 | `/tmp/statelet-final11-fmt-rust-baseline-completion-20261001.out`                |
| `make check-fmt`                       | Passed; 29 Markdown files unchanged                          | `/tmp/statelet-final11-check-fmt-rust-baseline-completion-20261001.out`          |
| `GIT_CONFIG_COUNT=0 make spelling`     | Passed; builder v0.1.3, `--scope all`, config current        | `/tmp/statelet-final11-spelling-rust-baseline-completion-20261001.out`           |
| `make lint-clippy`                     | Passed; Rustdoc and Clippy, no findings or manifest warnings | `/tmp/statelet-final5-lint-clippy-rust-baseline-completion-20261001.out`         |
| `make typecheck`                       | Passed for all targets and features                          | `/tmp/statelet-final5-typecheck-rust-baseline-completion-20261001.out`           |
| `make test`                            | Passed; 143 tests and one doctest                            | `/tmp/statelet-final5-test-rust-baseline-completion-20261001.out`                |
| `make test-workflow-contracts`         | Passed; 212 tests                                            | `/tmp/statelet-final11-workflow-contracts-rust-baseline-completion-20261001.out` |
| `GIT_CONFIG_COUNT=0 make markdownlint` | Passed; pinned markdownlint-cli2 0.23.2                      | `/tmp/statelet-final11-markdownlint-rust-baseline-completion-20261001.out`       |
| `make nixie`                           | Passed; all Mermaid diagrams validated                       | `/tmp/statelet-final5-nixie-rust-baseline-completion-20261001.out`               |
| `GIT_CONFIG_COUNT=0 make audit`        | Passed; 1,279 RustSec advisories loaded                      | `/tmp/statelet-final6-audit-rust-baseline-completion-20261001.out`               |
| `mbake validate Makefile`              | Passed                                                       | `/tmp/statelet-final11-mbake-rust-baseline-completion-20261001.out`              |

The workflow-contract tests exercise a deliberate formatting defect in a
temporary, non-ignored untracked Markdown file: `check-fmt` rejects it, `fmt`
repairs it, and `check-fmt` then passes. They also confirm ignored generated
Markdown under both `target/` and `.pytest_cache/` is excluded, formatter and
linter failures propagate through Make, the CI Markdown action's installer
order and file globs are binding, and the Whitaker failure propagates through
`make -j lint` after Rustdoc and Clippy. The controlled Whitaker executable
exits 23 in that test. The real rolling suite passed in Statelet PR #94's
hosted run, but that run predates this branch's sequential Make composite.

The local `markdownlint` target now obtains tracked and non-ignored untracked
Markdown from Git, matching the formatter's selection policy. This excludes
ignored generated files such as `.pytest_cache/README.md`; the CI action keeps
its required `**/*.md` glob and the markdownlint configuration excludes the
pytest cache directory.

After this selection correction, Concordat 0.1.0 at source commit
`8a4a1faba1290687c0b6b221e1fc96439ba3ba43` reran the affected
`rust-makefile-baseline` and `markdown-formatting-baseline` packages. Both
returned `compliant` with no findings; their logs are
`/tmp/statelet-final10-audit-rust-makefile-baseline-rust-baseline-completion-20261001.out`
and
`/tmp/statelet-final10-audit-markdown-formatting-baseline-rust-baseline-completion-20261001.out`.

The seven selected Concordat packages returned `compliant` with no findings on
the post-rebase candidate before the later Markdown selection correction:
`rust-build-defaults` 0.1.1, `rust-makefile-baseline` 0.3.2,
`markdown-formatting-baseline` 0.2.0, `main-owned-codescene-coverage` 0.3.0,
`spelling-config-baseline` 0.1.0, `whitaker-provisioning` 0.1.0, and
`dependabot-update-shape` 0.1.0. Concordat reported version 0.1.0 from source
commit `8a4a1faba1290687c0b6b221e1fc96439ba3ba43`. The Makefile and Markdown
audits used this command shape:

```sh
GIT_CONFIG_COUNT=0 PATH="$tool_bin:$PATH" \
  UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools uv tool run --python 3.14 \
  --from 'git+https://github.com/leynos/concordat.git@8a4a1faba1290687c0b6b221e1fc96439ba3ba43' \
  concordat artefact rule run "$rule" --repo "$PWD" --format json
```

Those two audits used `makeutil` 0.1.0 from source commit
`f405b89e4a903718b188d812044acbe4749bdbab`; its installed binary SHA-256 was
`71a187a4842012326c3e8eb49ae579d9bd703f5ba21602c27744f5e46152c262`. The ambient
binary came from `29fc5a16` and failed the Makefile audit before it returned a
policy verdict because its parser lacked a required variable-assignment
accessor. The pinned parser was installed to the ignored
`.uv-tools/makeutil-f405b89` prefix with:

```sh
GIT_CONFIG_COUNT=0 cargo install --root "$tool_prefix" \
  --git https://github.com/leynos/makeutil.git \
  --rev f405b89e4a903718b188d812044acbe4749bdbab --locked makeutil
```

The corrected Makefile and Markdown audits returned compliant with no findings.
Initial attempts using the ambient parser are preserved in the
`/tmp/statelet-final2-audit-*-rust-baseline-completion-20261001.out` logs;
corrected per-rule outputs are recorded under
`/tmp/statelet-final6-audit-*-rust-baseline-completion-20261001.out`.
`mbake validate Makefile` is recorded at
`/tmp/statelet-final6-mbake-rust-baseline-completion-20261001.out`.

Local composite `make lint` was not run because the ambient Whitaker installer
is 0.2.7 and cannot establish the pinned installer or rolling-suite provenance.
No manual Whitaker installation was performed. The approved action at full SHA
`6dea5677a84fec60ca51b07202570e3af12ffdb4` passed its real hosted suite and
lint in PR #94; available run metadata does not expose the installed suite
revision or installer-managed toolchain. Candidate-head CI remains required to
exercise this branch's sequential composite with the action-provisioned suite.

## Integration onto current main

On 2026-10-02, the three-commit series based on
`84235c6d8d3d5dc2880a97543385bda51d44faa2` was replayed onto fetched `main` at
`d83f693f61385a5ec3cd3ffb8eaddc59e33f6571`. The pre-rebase head
`ecfae45adbe4a0e8a90953fb483aa48afd4db9ca` is retained at
`refs/recovery/rust-baseline-completion-20261001/pre-rebase`. The three
replayed commits ended at `3638811eb42db8eed4df64e6d68436fd006641c5` before
this record update. The parent of the first replayed commit is the fetched
target.

The first replay had two conflicts, each limited to one shared-action `uses`
pin. `coverage-main.yml` keeps current main's
`upload-codescene-coverage@ff1dd759dfffc0db3459e30e833f52437ee62b57` from #98.
`mutation-testing.yml` keeps current main's
`mutation-cargo.yml@ff1dd759dfffc0db3459e30e833f52437ee62b57` from #95. The
branch's build-tool installation, binary-only Nextest route, coverage
publication control, and mutation setup commands are preserved around those
pins. Current main's #97 `setup-uv` bump in `ci.yml` also remains present. The
range-diff shows the second commit patch-identical. The first differs only by
omitting its superseded pin changes; the third updates this execution record
for the new target. `git diff --check` passed, and no merge markers or rebase
state remain.

The gate results above predate this integration onto `d83f693`. They are
historical evidence, not acceptance results for the new head. The Scrutineer
will run the required gates sequentially against the finalized commit before
publication. Hosted candidate CI, the exact rolling Whitaker suite revision,
and CodeScene secret placement remain unverified.

The first current-head `make check-fmt` stopped at this record's paragraph
wrapping; Rust formatting passed and no later gate ran. `make fmt` then passed,
changing only paragraph wrapping in this section, and Markdownlint reported
zero issues across 29 files. The Scrutineer will restart the gate sequence on
the amended head.

The next `make test-workflow-contracts` run reached 211 passes and one failure:
the CodeScene publisher contract still expected the uploader SHA used before
Statelet PR #98, although the workflow correctly retained current main's
`ff1dd759` pin. The expected upload SHA was updated to the PR #98 revision
without changing the distinct `install-whitaker` action at `6dea5677`. This
newly exposed contract blind spot stopped that gate sequence; the corrected
tree's results are recorded below.

## Validated current candidate

Commit `4b7dd350de1e2cd9b528169f13935a46d1db1aad` has source tree
`692dce1dd34d80f05ad66e0d3748cb4edd54a9e1`. The Scrutineer ran the following
gates sequentially against that exact tree after the CodeScene contract
correction. The earlier one-failure workflow-contract run remains the evidence
for the rebase blind spot; its replacement passed all 212 tests.

| Command                                   | Result                                          | Toolchain or version              | Log                                                                                                 |
| ----------------------------------------- | ----------------------------------------------- | --------------------------------- | --------------------------------------------------------------------------------------------------- |
| `make check-fmt`                          | Passed; 29 Markdown files unchanged             | rustfmt; mdtablefix 0.6.0         | `/tmp/check-fmt-rust-baseline-completion-20261001-rust-baseline-completion-20261001-3.out`          |
| `make spelling`                           | Passed; generated config current, `--scope all` | typos-config-builder v0.1.3       | `/tmp/spelling-rust-baseline-completion-20261001-rust-baseline-completion-20261001-2.out`           |
| `make lint-clippy`                        | Passed; Rustdoc and Clippy with warnings denied | nightly-2026-09-13; `mold` 2.41.0 | `/tmp/lint-rust-baseline-completion-20261001-rust-baseline-completion-20261001-2.out`               |
| `make typecheck`                          | Passed; all targets and features                | nightly-2026-09-13; `mold` 2.41.0 | `/tmp/typecheck-rust-baseline-completion-20261001-rust-baseline-completion-20261001-2.out`          |
| `make test`                               | Passed; 143 tests and one doctest               | nightly-2026-09-13; `mold` 2.41.0 | `/tmp/test-rust-baseline-completion-20261001-rust-baseline-completion-20261001-2.out`               |
| `make test-workflow-contracts`            | Passed; 212 tests                               | pytest via `uv`                   | `/tmp/workflow-contracts-rust-baseline-completion-20261001-rust-baseline-completion-20261001-2.out` |
| `make markdownlint`                       | Passed; 0 issues across 29 files                | markdownlint-cli2 0.23.2          | `/tmp/markdownlint-rust-baseline-completion-20261001-rust-baseline-completion-20261001.out`         |
| `make nixie`                              | Passed; all diagrams validated                  | version not recorded              | `/tmp/nixie-rust-baseline-completion-20261001-rust-baseline-completion-20261001.out`                |
| `GIT_CONFIG_COUNT=0 make audit`           | Passed; 1,280 advisories loaded                 | cargo-audit 0.22.1                | `/tmp/audit-rust-baseline-completion-20261001-rust-baseline-completion-20261001.out`                |
| `mbake validate Makefile`                 | Passed                                          | version not recorded              | `/tmp/mbake-rust-baseline-completion-20261001-rust-baseline-completion-20261001.out`                |
| `git diff --check` and rebase diff checks | Passed; no whitespace or conflict markers       | version not recorded              | `/tmp/diff-check-rust-baseline-completion-20261001-rust-baseline-completion-20261001.out`           |

The local Whitaker suite was not run: only the ambient 0.2.7 installer and
`cargo-dylint` 6.0.1 are available, and no source fallback was used. CI uses
the approved `install-whitaker@6dea5677a84fec60ca51b07202570e3af12ffdb4` action
with `cranelift: true`. Statelet PR #94's hosted installation and lint pass is
historical evidence. This candidate has no hosted CI run yet, so the rolling
suite revision and installer-managed toolchain are still unmeasured.

## Python lint and typecheck batch

The Python gateways were added to the same draft PR in commit
`090bb9f1569b95022759ac4e83069debb15d8444` (tree
`800206946e9bdcc4615f2191456fe1a59c77c862`). The commit was replayed onto the
published PR head `c1145490af83f4ae28ad7431a6bf31209526af96`; its tree matched
the measured pre-rebase tree exactly. The range-diff reports the patch
unchanged. GitHub read-back confirms PR #101 remains open and draft.

The Makefile uses managed CPython 3.14 for workflow-contract tests, Pylint
4.0.9 with all default diagnostics active, and `df12-python-lints` v0.3.0 at
`4cf41736cce2f7ba2778882a5c629c044568a0e5`. It enables all 13 df12 messages. The
`ty` typechecker is pinned to 0.0.74. Both gateways share one recursive
inventory under `.github`, `tests`, `scripts`, `benches`, and `benchmarks`, so
workflow and action Python modules are included. Generated and ignored trees
are pruned. No Pylint, df12, or `ty` suppressions were added; findings were
fixed in source.

Candidate CI run `37009632090` passed formatting, spelling, Makefile checks,
Whitaker installation, Rust/Python lint, typechecking, and all 223 workflow
contract tests, then failed in coverage detection with
`Mixed projects only support cobertura format`. Adding the configuration-only
`pyproject.toml` made the shared coverage action's default detection classify
this Rust crate as mixed-language. The pinned action at
`013346ccfd37bd1e02eb430525233f1a4cd942b8` documents `language: rust` for Rust
projects with a configuration-only `pyproject.toml`; both coverage calls now
set that input, and the CV-005 contract requires it with mutations for missing
or incorrect values. Candidate CI must be rerun on the corrected head.

The Scrutineer ran these checks sequentially on the measured tree. For commit
`090bb9f`, CI run `37009632090` was in progress and Act Validation run
`37009632165` had passed. These statuses are for the Python batch commit; a new
execution-record commit will trigger fresh candidate checks.

- `make check-fmt`: passed; 29 Markdown files unchanged. Log:
  `/tmp/check-fmt-rust-baseline-completion-20261001-rust-baseline-completion-20261001-7.out`.
- `make lint-python`: passed; Pylint score 10.00/10. Log:
  `/tmp/lint-python-rust-baseline-completion-20261001-rust-baseline-completion-20261001-7.out`.
- `make typecheck-python`: passed. Log:
  `/tmp/typecheck-python-rust-baseline-completion-20261001-rust-baseline-completion-20261001-5.out`.
- `make test-workflow-contracts`: passed; 223 tests. Log:
  `/tmp/test-workflow-contracts-rust-baseline-completion-20261001-rust-baseline-completion-20261001-5.out`.
- `make markdownlint`: passed; spelling passed and Markdown reported zero
  issues. Log:
  `/tmp/markdownlint-rust-baseline-completion-20261001-rust-baseline-completion-20261001-4.out`.
- `mbake validate Makefile`: passed. Log:
  `/tmp/mbake-validate-rust-baseline-completion-20261001-rust-baseline-completion-20261001-3.out`.
- `git diff --check`: passed. Log:
  `/tmp/diff-check-rust-baseline-completion-20261001-rust-baseline-completion-20261001-3.out`.

After adding this execution-record section, `make fmt` passed and left 29
Markdown files with zero issues. The docs-only Scrutineer pass then found that
the standalone `make spelling` command failed to fetch the pinned builder
because of a transient Git resolver error. Its immediately following
`make markdownlint` run passed the same spelling gate and reported zero issues
across 29 files. `make check-fmt`, `mbake validate Makefile`, and
`git diff --check` also passed. The separate spelling attempt is recorded at
`/tmp/spelling-rust-baseline-completion-20261001-rust-baseline-completion-20261001-1.out`;
the passing spelling gate is in
`/tmp/markdownlint-rust-baseline-completion-20261001-rust-baseline-completion-20261001-5.out`.
A further standalone `make spelling` run passed with builder v0.1.3 and
`--scope all`; its log is
`/tmp/spelling-rust-baseline-completion-20261001-rust-baseline-completion-20261001-2.out`.

The coverage-scope correction was checked sequentially with the Python gateways
on working-tree fingerprint
`9c0be3bd9e14ead553baf32426af93d49717355587fc94be616f115d0ba449c6`. All checks
passed; the Scrutineer confirmed the five changed paths were unchanged by the
gates.

- `make check-fmt`: passed; 29 files unchanged. Log:
  `/tmp/check-fmt-rust-baseline-completion-20261001-rust-baseline-completion-20261001-10.out`.
- `make spelling`: passed with builder v0.1.3 and `--scope all`. Log:
  `/tmp/spelling-rust-baseline-completion-20261001-rust-baseline-completion-20261001-3.out`.
- `make lint-python`: passed; Pylint score 10.00/10. Log:
  `/tmp/lint-python-rust-baseline-completion-20261001-rust-baseline-completion-20261001-8.out`.
- `make typecheck-python`: passed. Log:
  `/tmp/typecheck-python-rust-baseline-completion-20261001-rust-baseline-completion-20261001-6.out`.
- `make test-workflow-contracts`: passed; 225 tests. Log:
  `/tmp/test-workflow-contracts-rust-baseline-completion-20261001-rust-baseline-completion-20261001-6.out`.
- `make markdownlint`: passed with markdownlint-cli2 v0.23.2; 29 files, zero
  issues. Log:
  `/tmp/markdownlint-rust-baseline-completion-20261001-rust-baseline-completion-20261001-7.out`.
- `mbake validate Makefile`: passed. Log:
  `/tmp/mbake-validate-rust-baseline-completion-20261001-rust-baseline-completion-20261001-6.out`.
- `git diff --check`: passed. Log:
  `/tmp/diff-check-rust-baseline-completion-20261001-rust-baseline-completion-20261001-6.out`.

## Corrected-head hosted validation

GitHub run
[37011322792](https://github.com/leynos/statelet/actions/runs/37011322792)
passed on commit `0b4b671c25cc57e80921dbfc6c4c94b654f096a5`. Its Linux
`build-test` job passed formatting, Markdown lint, spelling, audit, Rust lint,
Python lint, typechecking, all 225 workflow-contract tests, and coverage. The
coverage action now treats the repository as Rust despite the lint-only
`pyproject.toml`. Act Validation run
[37011322676](https://github.com/leynos/statelet/actions/runs/37011322676) also
passed.

The run used the approved
`install-whitaker@6dea5677a84fec60ca51b07202570e3af12ffdb4` action with
installer 0.2.9. Its log records `suite=default-branch-tip`,
`suite-source=prebuilt`, and asset
`whitaker-lints-318d8d7-nightly-2026-05-28-x86_64-unknown-linux-gnu.tar.zst`.
The rolling Whitaker source revision was
[`318d8d76b35b2a8c14a7d8ab6ef526f391889963`](https://github.com/leynos/whitaker/commit/318d8d76b35b2a8c14a7d8ab6ef526f391889963);
the installer-managed toolchain was `nightly-2026-05-28`. The repository's
build toolchain was `nightly-2026-09-13`. The suite gate passed, and the log
reports prebuilt artefacts rather than source fallback.

The successful CI and Act Validation results apply to `0b4b671`; a subsequent
execution-record-only commit requires its own current-head hosted checks. The
CodeScene Code Health Review on `0b4b671` failed. The protected `codescene`
environment is main-only, but secret placement remains unverified because the
GitHub integration cannot read its secret metadata. This is still an open
administrative gate; the successful CI run does not complete that requirement.
