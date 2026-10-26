# Macro cost benchmark note template

Copy the fenced form below into `docs/validation-notes/<task>-<subject>.md`,
keeping its marker and register markers. Create one note for each comparison:
`mdtablefix` and `synthetic-20`. The real and synthetic experiments may be
recorded independently. The macro publication decision at task 4.3.1 requires
both notes for the same macro and design revisions.

Fill every value and evidence cell. Cite supporting material as
`<repo>@<full-sha>:<path>`. Keep full command output and raw samples in the
cited artefacts; summarize the results here. A note can record a failed or
inconclusive measurement, but that result cannot pass a budget gate.

```markdown
# Macro cost benchmark note

<!-- macro-cost-note -->

<!-- macro-cost-register:begin -->
| Field | Value | Evidence |
| --- | --- | --- |
| comparison-kind | <mdtablefix or synthetic-20> | <repo>@<sha>:<path> |
| candidate-id | macro=<macro-sha>; design=<design-sha> | <repo>@<sha>:<path> |
| revisions | Baseline/macro commits; fixture paths; lockfile; annotation diff | <repo>@<sha>:<path> |
| environment | CPU/RAM; OS; target; compiler/tools; features; linker; flags/jobs | <repo>@<sha>:<path> |
| clean-build-protocol | Commands; clean dirs; caches; wrappers; admission/load; order; repeats | <repo>@<sha>:<path> |
| clean-debug-baseline-seconds | Raw baseline samples; median T0; range | <repo>@<sha>:<path> |
| clean-debug-macro-seconds | Raw macro samples; median T1; range | <repo>@<sha>:<path> |
| annotation-count | Positive integer only; source locations go in Evidence; synthetic-20 = 20 | <repo>@<sha>:<path> |
| marginal-seconds | Inputs and calculated marginal seconds per annotation | <repo>@<sha>:<path> |
| marginal-percent | Inputs and calculated marginal percentage | <repo>@<sha>:<path> |
| release-baseline-bytes | Exact baseline executable or library path and byte count | <repo>@<sha>:<path> |
| release-macro-bytes | Matching macro executable or library path and byte count | <repo>@<sha>:<path> |
| release-growth-percent | Inputs and calculated percentage for the matching artefact kind | <repo>@<sha>:<path> |
| stress-fixture-results | 20-transition results; exact real-comparison exception | <repo>@<sha>:<path> |
| fallible-expansion | `cargo expand` command, output citation, and readability review | <repo>@<sha>:<path> |
| infallible-expansion | `cargo expand` command, output citation, and readability review | <repo>@<sha>:<path> |
| budget-verdict | Time, size and expansion conclusions; design revision | <repo>@<sha>:<path> |
| review | Author; reviewer; date; limitations; related Phase 4 evidence | <repo>@<sha>:<path> |
<!-- macro-cost-register:end -->
```

Use positive baseline time `T0`, macro time `T1`, annotation count `N`,
baseline artefact size `S0 > 0`, and macro artefact size `S1`:

```text
marginal seconds = (T1 - T0) / N
marginal percent = 100 * (T1 - T0) / (N * T0)
release growth percent = 100 * (S1 - S0) / S0
synthetic time acceptance: marginal percent < 0.5, N = 20
size acceptance for each comparison: release growth percent <= 2
```

For example, with T0 = 100 seconds and N = 20, T1 = 109.8 gives 0.49%, below
the limit, while T1 = 110 gives 0.5% and fails the strict limit. With S0 =
100000 bytes, S1 = 102000 gives 2% and passes, while S1 = 102001 exceeds 2% and
fails the inclusive size limit.

`T0` and `T1` are medians of at least five paired clean-debug runs. Alternate
baseline-first and macro-first order, retain every sample, and report each
range. Use the same workload semantics, observations, dependencies other than
the macro implementation, and features. Record cache state, incremental
settings, wrappers, compiler admission and host load. A new target directory
does not prove that shared dependency or compiler caches were cold. If queue
time, cache asymmetry or unexplained variance dominates, record the measurement
as inconclusive.

Compare corresponding release executables or libraries built with matching
features, stripping, link-time optimization and code-generation settings.
Identify retained entrypoints or exports so dead-code removal cannot erase the
measured transitions. Do not compare whole target-directory sizes or different
artefact kinds. Preserve negative marginal deltas; do not clamp them or hide
them through rounding.

For `synthetic-20`, cite the downstream fixture and show that all 20 annotated
transitions are represented and checked. For `mdtablefix`, use exactly
`Not applicable: real comparison` in `stress-fixture-results`; this lets the
real evidence be committed before the independent synthetic experiment. Cite
the source and output for one fallible and one infallible expansion, and ask a
reviewer to assess their readability.

Every completed copied note needs revision-bound evidence in every row. The
`candidate-id` binds the pair of notes to one macro commit and the governing
design commit. Empty cells and values beginning with the case-insensitive
marker words `TBD`, `TODO`, `pending`, or `N/A` are incomplete. Standalone
angle-bracket placeholders are incomplete, including inside backtick spans.
Expansion text may contain attached Rust generics such as `Option<Foo>`; do not
use an attached form such as `Type<sha>` to represent missing evidence. Only
`review` may say exactly `Pending: G3 decision not yet recorded`. A failed
measurement may use `Unavailable: <reason>` with a cited failure record; its
`budget-verdict` must say `Inconclusive`. An adverse note is evidence of
failure, not approval.

The contract checks completeness and roadmap wiring. It does not verify that a
citation exists, that workloads are comparable, or that a measurement is true.
Reviewers assess those claims. Completing task 4.2.2 requires the selected
synthetic note; completing task 4.3.1 requires both selected notes and a
candidate comment beside the roadmap claim. Neither a complete form nor a
passing cost result alone selects the v0.1 exit; apply ADR 003 and all other
Phase 4 gates.
