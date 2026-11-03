//! Domain wrappers for the macro-cost contract's borrowed text.
//!
//! The contract reads raw Markdown, so most values begin as `&str`. Wrapping
//! each role — task identifier, document path, revision, source line, field
//! name and table cell — stops a signature from accepting any nearby text and
//! names the meaning of each argument at its call site.
//!
//! Every wrapper borrows its content. Construct one with a tuple literal where
//! the text is first read; a constructor function taking `&str` would
//! reintroduce the string-typed argument this module removes.

use std::{fmt, ops::Deref};

/// Declares a borrowed string wrapper that behaves like `str`.
///
/// `Deref` lets predicates such as `starts_with` run directly on the wrapper,
/// while `Display` lets diagnostics interpolate it. Both implementations are
/// identical for every role, so one declaration keeps the vocabulary
/// reviewable.
macro_rules! text_newtype {
    ($(#[$attribute:meta])* $name:ident) => {
        $(#[$attribute])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) struct $name<'a>(pub(crate) &'a str);

        impl<'a> Deref for $name<'a> {
            type Target = str;

            fn deref(&self) -> &str { self.0 }
        }

        impl fmt::Display for $name<'_> {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.0)
            }
        }
    };
}

text_newtype!(
    /// A roadmap task identifier such as `4.2.2`.
    TaskId
);

text_newtype!(
    /// A repository-relative path used to open or report a document.
    DocumentPath
);

text_newtype!(
    /// A full lowercase Git revision named by a roadmap or note claim.
    Revision
);

text_newtype!(
    /// One raw line of Markdown source.
    SourceLine
);

text_newtype!(
    /// A macro-cost register field name.
    FieldName
);

text_newtype!(
    /// A trimmed cell from a `Field | Value | Evidence` register row.
    CellText
);
