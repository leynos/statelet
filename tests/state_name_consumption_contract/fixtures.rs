//! Canonical Markdown fixtures shared by the consumption-contract scenarios.
//!
//! These are deliberately *not* the live documents. The register fixtures are
//! the pin: `INV-REGISTERS` compares the live registers to them row for row, so
//! an editor of ADR 004 updates the fixture beside it or the suite fails. They
//! are written with single-space cell padding, so a negative control's text
//! replacement never has to restate a column width.
//!
//! Each fixture carries the heading its parser bounds against, because parsing
//! is section-scoped: a table alone would fail with `MissingSection` rather than
//! exercising the check under test.
//!
//! Tables are assembled from arrays of row literals, and the note's rows are
//! exported as constants. A negative control replaces exact cell text, so the
//! fixture and the control must read the same string: a control that restated a
//! row in its own wrapping would become a silent no-op the first time the
//! fixture was rewrapped, and would then assert a failure the check no longer
//! produces.

/// The section heading the status register lives under.
pub(crate) const STATUS_HEADING: &str = "## Status register";

/// The section heading the aggregation register lives under.
pub(crate) const AGGREGATION_HEADING: &str = "## Aggregation register";

/// The section heading the gate table lives under.
pub(crate) const GATES_HEADING: &str = "## Gates";

/// A complete, valid status register with its delimiters and its section.
pub(crate) fn status_register() -> String {
    [
        STATUS_HEADING,
        "",
        "<!-- status-register:begin -->",
        "| Field | Status | Admissible | Contributes |",
        "| --- | --- | --- | --- |",
        "| state-display-name | Enumerated | yes | nothing |",
        "| state-display-name | Not a named type | no | nothing |",
        "| identifier-need | None | yes | Sufficient |",
        "| identifier-need | Property required | yes | Insufficient |",
        "| metrics-cardinality | Bounded | yes | nothing |",
        "| metrics-cardinality | Unbounded | no | nothing |",
        "| tracing-use | Full | yes | nothing |",
        "| tracing-use | Partial | yes | nothing |",
        "| tracing-use | None | yes | nothing |",
        "<!-- status-register:end -->",
        "",
    ]
    .join("\n")
}

/// A complete, valid aggregation register with its delimiters and its section.
///
/// The outcome column is headed `Outcome if publication proceeds` rather than
/// `Outcome`. Every row's outcome is conditional on ADR 003's gate G2 not having
/// selected exit E1, and the header says so; the header is recognized
/// structurally, so it is reproduced here exactly.
pub(crate) fn aggregation_register() -> String {
    [
        AGGREGATION_HEADING,
        "",
        "<!-- aggregation-register:begin -->",
        "| Contributing notes | Any insufficient | Outcome if publication proceeds |",
        "| --- | --- | --- |",
        "| None | n/a | Blocked: no admissible evidence |",
        "| One or more | No | Ratify the current return type |",
        "| One or more | Yes | Amend design 6.1 before publish |",
        "<!-- aggregation-register:end -->",
        "",
    ]
    .join("\n")
}

/// A complete, valid gate table with its delimiters and its section.
///
/// The fragments are copied from live roadmap task titles: S1 from the
/// `ProcessBuffer` annotation task, S2 from the continuation annotation task, S3
/// from the conventions-only baseline task, and S4 from the return-shape task.
/// Each resolves to exactly one line of `docs/roadmap.md`.
pub(crate) fn gate_table() -> String {
    [
        GATES_HEADING,
        "",
        "<!-- gate-table:begin -->",
        "| Gate | Roadmap task title fragment |",
        "| --- | --- |",
        "| S1 | Annotate `mdtablefix` `ProcessBuffer` |",
        "| S2 | Annotate `mdtablefix` continuation |",
        "| S3 | Apply the conventions-only baseline |",
        "| S4 | Finalize the `StateName` return shape |",
        "<!-- gate-table:end -->",
        "",
    ]
    .join("\n")
}

/// The status register without one field's rows.
///
/// For controls that remove a whole field rather than mutate one cell. Removing
/// a single row would leave the field's other statuses in place, and the check
/// under test would not fire.
pub(crate) fn status_register_without(field: &str) -> String {
    let prefix = format!("| {field} |");
    status_register()
        .lines()
        .filter(|line| !line.starts_with(&prefix))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// The `state-display-name` row of the note fixture.
pub(crate) const STATE_DISPLAY_NAME_ROW: &str =
    "| state-display-name | Enumerated | `mdtablefix@abc1234:src/process.rs` lists LineMode |";

/// The `identifier-need` row of the note fixture.
///
/// The cell names both a consumer and an *unmet* property while the status
/// records `None`, which is a valid note: the property clause sits inside a
/// participle phrase negated by "no unmet property", the shape a real "no
/// consumer asked for anything" observation takes. A cell naming a property
/// without that negation is the disagreement a rejecting control covers.
#[rustfmt::skip]
pub(crate) const IDENTIFIER_NEED_ROW: &str =
    "| identifier-need | None | subscriber, metrics, model checker and generated documentation considered; `mdtablefix@abc1234:src/process.rs` records no unmet property |";

/// The `metrics-cardinality` row of the note fixture.
pub(crate) const METRICS_CARDINALITY_ROW: &str =
    "| metrics-cardinality | Bounded | `mdtablefix@abc1234:src/process.rs` yields three names |";

/// The `tracing-use` row of the note fixture.
#[rustfmt::skip]
pub(crate) const TRACING_USE_ROW: &str =
    "| tracing-use | Full | emits `transition.state.before`; `mdtablefix@abc1234:src/process.rs` |";

/// The note fixture's four register rows, in register order.
pub(crate) const NOTE_ROWS: [&str; 4] = [
    STATE_DISPLAY_NAME_ROW,
    IDENTIFIER_NEED_ROW,
    METRICS_CARDINALITY_ROW,
    TRACING_USE_ROW,
];

/// A complete `StateName` note carrying exactly the rows it is given.
///
/// Controls select rows through this rather than editing the assembled note,
/// so a control cannot quietly stop applying when the fixture is rewrapped.
pub(crate) fn note_with_rows(rows: &[&str]) -> String {
    let mut lines = vec![
        "<!-- state-name-note -->",
        "# Validation note: StateName consumption in `mdtablefix`",
        "",
        "Roadmap task: 2.2.1.",
        "",
        "## Note register",
        "",
        "<!-- note-register:begin -->",
        "| Field | Status | Evidence |",
        "| --- | --- | --- |",
    ];
    lines.extend_from_slice(rows);
    lines.extend_from_slice(&["<!-- note-register:end -->", ""]);
    lines.join("\n")
}

/// A complete, valid `StateName` note declaring the marker.
///
/// Every cell carries a citation, as ADR 004 requires of every evidence cell
/// including one recording `None`. The `identifier-need` cell carries its
/// citation beside the search set rather than instead of it, and the
/// `tracing-use` cell likewise beside the label it observed.
pub(crate) fn state_name_note() -> String { note_with_rows(&NOTE_ROWS) }

/// A note shaped like roadmap task 1.2.3's benchmark note: the marker belongs to
/// another roadmap task, and the fields are not the register's. A scan that
/// globbed the directory instead of reading the marker would reject it.
pub(crate) fn benchmark_note() -> String {
    [
        "<!-- benchmark-note -->",
        "# Validation note: `mdtablefix` benchmark",
        "",
        "| Field | Status | Evidence |",
        "| --- | --- | --- |",
        "| throughput | Unchanged | `mdtablefix@abc1234:benches` |",
        "",
    ]
    .join("\n")
}

/// The blank form a Phase 2 engineer copies into `docs/validation-notes/`.
///
/// Read from the live document rather than restated, so that the schema check
/// fails when the form and the status register disagree.
pub(crate) const TEMPLATE: &str = include_str!("../../docs/phase-2-validation-note-template.md");
