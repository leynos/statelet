//! Transition tracing field names, published as a semver-relevant contract.
//!
//! A transition boundary — the function or method where a state machine
//! handles one input and decides what happens next — records its work on a
//! `tracing` span. Statelet fixes the names of five fields on that span so
//! dashboards, alerts, and log queries can depend on them. The full contract,
//! including each field's meaning, when it is recorded, and the shape of its
//! value, is [ADR 0008][adr-0008]; this module publishes the names so a
//! downstream crate can use them from code.
//!
//! Naming a field through these constants moves the spelling under the
//! compiler. A misspelt constant fails to compile, while a misspelt dotted
//! name silently creates a different field that nothing reads:
//!
//! ```
//! use statelet::{StateName, fields};
//!
//! enum Mode {
//!     Normalize,
//! }
//!
//! impl StateName for Mode {
//!     fn state_name(&self) -> &'static str { "Normalize" }
//! }
//!
//! #[tracing::instrument(skip_all, fields(
//!     {fields::TRANSITION_NAME} = "handle_line",
//!     {fields::TRANSITION_STATE_BEFORE} = mode.state_name(),
//!     {fields::TRANSITION_EVENT} = "table_row",
//!     {fields::TRANSITION_OUTCOME} = tracing::field::Empty,
//!     {fields::TRANSITION_ERROR} = tracing::field::Empty,
//! ))]
//! fn handle_line(mode: &Mode, fails: bool) -> Result<&'static str, &'static str> {
//!     if fails {
//!         let category = "unterminated_row";
//!         tracing::Span::current().record(fields::TRANSITION_ERROR, category);
//!         return Err(category);
//!     }
//!     let outcome = "stay";
//!     tracing::Span::current().record(fields::TRANSITION_OUTCOME, outcome);
//!     Ok(outcome)
//! }
//!
//! assert_eq!(
//!     handle_line(&Mode::Normalize, false),
//!     Ok("stay"),
//!     "the boundary must return the label it recorded"
//! );
//! assert_eq!(
//!     handle_line(&Mode::Normalize, true),
//!     Err("unterminated_row"),
//!     "the boundary must return its bounded error category"
//! );
//! ```
//!
//! A field recorded after the span is created must be declared when the span
//! is created — with `tracing::field::Empty` or a bare `fields(name)`
//! declaration — because `tracing` silently discards a value recorded for an
//! undeclared field. A declared field that is never recorded stays absent, and
//! absence means only that the boundary did not capture it: it never implies
//! success or failure.
//!
//! A late record written through `tracing::Span::current()` targets the
//! nearest enabled span. When the subscriber's filter disables the boundary's
//! own span, entering it does nothing and the record can reach an enclosing
//! span that declares the same field, so do not rely on `Span::current()` to
//! identify the boundary's own span under filtering. An application that needs
//! the strict location must keep the boundary span enabled.
//!
//! Braced constant field names need both a `tracing` facade whose macros accept
//! braced names and a resolved `tracing-attributes` 0.1.31 or later. In
//! practice that is `tracing` 0.1.43 or later — `tracing` 0.1.42, the first
//! release to require 0.1.31, is yanked — with `tracing-attributes` 0.1.31 or
//! later. A newer `tracing-attributes` with an older facade is not sufficient,
//! because the facade's own `span!` macros must accept the braced name. A
//! dotted field name written in
//! the list, such as `transition.name = "handle_line"`, produces the same field
//! and needs a resolved `tracing-attributes` 0.1.9 or later; `tracing` 0.1.9
//! itself permits an older `tracing-attributes`, so the requirement is on the
//! resolved attributes crate rather than the `tracing` version alone. Dotted
//! names remain an equal alternative to the constants.
//!
//! [adr-0008]: https://github.com/leynos/statelet/blob/main/docs/adrs/adr-0008-publish-the-transition-tracing-field-contract.md

/// The stable label naming the transition boundary.
///
/// The application chooses the label, usually the instrumented function's
/// name. Every boundary span carries this field. The value rules are
/// provisional until roadmap task 3.1.3.
#[doc(alias = "transition.name")]
pub const TRANSITION_NAME: &str = "transition.name";

/// The label of the state observed on entry.
///
/// Recorded by a boundary that observes a state. The contract recommends the
/// label [`crate::StateName::state_name`] returns, but does not require it; a
/// different stable label is permitted.
#[doc(alias = "transition.state.before")]
pub const TRANSITION_STATE_BEFORE: &str = "transition.state.before";

/// The stable label describing the input the boundary handles.
///
/// The contract recommends a low-cardinality semantic class of the input rather
/// than the input itself, so a raw parsed line does not reach the trace, but it
/// does not require that projection. It does not forbid recording the
/// expression form a future macro's `event(...)` argument accepts; that
/// question is deferred in design §14.
#[doc(alias = "transition.event")]
pub const TRANSITION_EVENT: &str = "transition.event";

/// The stable label of the result the boundary produced.
///
/// Chosen by the application, for example the variant label of its own
/// decision enum. The contract does not require it to be absent when the
/// boundary returns an error, and the value rules behind it are provisional
/// until roadmap task 3.1.3.
#[doc(alias = "transition.outcome")]
pub const TRANSITION_OUTCOME: &str = "transition.outcome";

/// The stable category of the error a fallible boundary returned.
///
/// A bounded category such as the variant label of the application's error
/// enum, not arbitrary error text; the contract forbids secrets and raw
/// payloads in any value. Only fallible boundaries declare it. The value rules
/// are provisional until roadmap task 3.1.3.
#[doc(alias = "transition.error")]
pub const TRANSITION_ERROR: &str = "transition.error";
