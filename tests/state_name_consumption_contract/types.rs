//! Register keys, typed rows and parse failures for the consumption contract.
//!
//! Every repair message this contract emits is derived from a `Register` token
//! rather than from a document string, because ADR 004 carries four registers
//! and a failure must say which one it came from.

use std::fmt::{self, Display, Formatter};

/// One delimited register, and everything a repair message needs to name it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Register {
    Status,
    Aggregation,
    Gates,
    Note,
    Evidence,
}

impl Register {
    /// The document this register lives in, as a repository-relative path.
    pub(crate) const fn document(self) -> &'static str {
        match self {
            Self::Status | Self::Aggregation | Self::Gates | Self::Evidence => {
                "docs/adr-004-state-name-consumption-evidence.md"
            }
            Self::Note => "docs/phase-2-validation-note-template.md",
        }
    }

    /// The section heading whose body must contain this register.
    pub(crate) const fn section(self) -> &'static str {
        match self {
            Self::Status => "## Status register",
            Self::Aggregation => "## Aggregation register",
            Self::Gates => "## Gates",
            Self::Evidence => "## Evidence the record preserves",
            Self::Note => "## Note register",
        }
    }

    /// The opening delimiter comment.
    pub(crate) const fn begin(self) -> &'static str {
        match self {
            Self::Status => "<!-- status-register:begin -->",
            Self::Aggregation => "<!-- aggregation-register:begin -->",
            Self::Gates => "<!-- gate-table:begin -->",
            Self::Note => "<!-- note-register:begin -->",
            Self::Evidence => "<!-- evidence:begin -->",
        }
    }

    /// The closing delimiter comment.
    pub(crate) const fn end(self) -> &'static str {
        match self {
            Self::Status => "<!-- status-register:end -->",
            Self::Aggregation => "<!-- aggregation-register:end -->",
            Self::Gates => "<!-- gate-table:end -->",
            Self::Note => "<!-- note-register:end -->",
            Self::Evidence => "<!-- evidence:end -->",
        }
    }

    /// The lower-case name used in prose, such as "gate table".
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Status => "status register",
            Self::Aggregation => "aggregation register",
            Self::Gates => "gate table",
            Self::Note => "note register",
            Self::Evidence => "evidence section",
        }
    }
}

/// Why a register could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ParseError {
    /// The delimiters are absent, or the block holds no data row.
    MissingDelimiters { register: Register },
    /// The section heading holding the register is absent.
    MissingSection { register: Register },
    /// A table row did not split into the expected number of cells.
    MalformedRow { register: Register, row: usize },
    /// A cell held a token the register's own vocabulary does not define.
    UnknownToken {
        register: Register,
        column: &'static str,
        found: String,
    },
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSection { register } => {
                let (document, label, section) =
                    (register.document(), register.label(), register.section());
                write!(
                    formatter,
                    "{document}: the {label} section is absent. Repair: restore the {section} \
                     heading."
                )
            }
            Self::MissingDelimiters { register } => {
                let (document, label) = (register.document(), register.label());
                let (begin, end) = (register.begin(), register.end());
                write!(
                    formatter,
                    "{document}: no {label} found between {begin} and {end}. Repair: add the \
                     {label} to the {} section.",
                    register.section()
                )
            }
            Self::MalformedRow { register, row } => {
                let (document, label) = (register.document(), register.label());
                write!(
                    formatter,
                    "{document}: row {row} of the {label} is malformed. Repair: supply every cell \
                     the {label} header declares."
                )
            }
            Self::UnknownToken {
                register,
                column,
                found,
            } => {
                let (document, label) = (register.document(), register.label());
                write!(
                    formatter,
                    "{document}: unknown {column} {found:?} in the {label}. Repair: use a \
                     {column} the {label} itself defines, or add it to the register first."
                )
            }
        }
    }
}

/// One row of ADR 004's status register.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StatusRow {
    pub(crate) field: String,
    pub(crate) status: String,
    /// Whether a note selecting this status is still usable evidence.
    pub(crate) admissible: bool,
    /// `nothing`, `Sufficient` or `Insufficient`; never empty.
    pub(crate) contributes: String,
}

/// One row of ADR 004's aggregation register.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AggRow {
    pub(crate) admissible_notes: String,
    pub(crate) any_insufficient: String,
    pub(crate) outcome: String,
}

/// One row of ADR 004's gate table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GateRow {
    pub(crate) gate: String,
    /// A roadmap task *title* fragment, not a task number.
    pub(crate) fragment: String,
}

/// One field of a validation note, with its observed status and its citation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NoteRow {
    pub(crate) field: String,
    pub(crate) status: String,
    pub(crate) evidence: String,
}

/// What one note says about the return type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Resolution {
    /// The default holds: no admissible cell recorded a required property.
    Sufficient,
    /// Overturned: an admissible `identifier-need` cell recorded one.
    Insufficient,
    /// No verdict, because a cell the register marks inadmissible blocks it.
    NotResolved { field: String, status: String },
}

impl Display for Resolution {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sufficient => formatter.write_str("Sufficient"),
            Self::Insufficient => formatter.write_str("Insufficient"),
            Self::NotResolved { field, status } => {
                write!(formatter, "Not resolved (blocked by {field}: {status})")
            }
        }
    }
}

/// The distinct field identifiers of a status register, in first-appearance
/// order — the order a Phase 2 engineer fills the template in.
pub(crate) fn field_order(rows: &[StatusRow]) -> Vec<String> {
    let mut ordered: Vec<String> = Vec::new();
    for row in rows {
        if !ordered.iter().any(|field| field == &row.field) {
            ordered.push(row.field.clone());
        }
    }
    ordered
}
