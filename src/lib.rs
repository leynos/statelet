//! Transition-boundary support for ordinary Rust state machines.

/// A state that reports a stable, deliberately chosen label.
///
/// Implement this trait for an application-owned enum when its states need to
/// be observable without implementing [`Debug`](std::fmt::Debug). The trait has
/// no supertraits and no default method, so an ordinary downstream crate can
/// implement it directly:
///
/// ```
/// use statelet::StateName;
///
/// enum ContinuationMode {
///     Normalize,
///     TightCodeSpan,
///     VerbatimFlush,
/// }
///
/// impl StateName for ContinuationMode {
///     fn state_name(&self) -> &'static str {
///         match self {
///             Self::Normalize => "Normalize",
///             Self::TightCodeSpan => "TightCodeSpan",
///             Self::VerbatimFlush => "VerbatimFlush",
///         }
///     }
/// }
///
/// assert_eq!(
///     ContinuationMode::TightCodeSpan.state_name(),
///     "TightCodeSpan"
/// );
/// ```
///
/// Implementors choose the labels and own their stability. The trait fixes the
/// return lifetime and type, but it cannot enforce that a label survives a
/// variant rename or that an implementation is free of side effects, so use
/// explicit literals rather than deriving labels from formatting traits, type
/// names, or numeric discriminants.
pub trait StateName {
    /// Returns the state's deliberately chosen static label.
    ///
    /// The label is chosen by the implementor, not derived by this crate. It is
    /// independent of any payload the state carries, so equal variants with
    /// different payloads report the same label:
    ///
    /// ```
    /// use statelet::StateName;
    ///
    /// enum Scan {
    ///     Idle,
    ///     Reading(u64),
    /// }
    ///
    /// impl StateName for Scan {
    ///     fn state_name(&self) -> &'static str {
    ///         match self {
    ///             Self::Idle => "Idle",
    ///             Self::Reading(_) => "Reading",
    ///         }
    ///     }
    /// }
    ///
    /// let reading = Scan::Reading(42);
    /// assert_eq!(reading.state_name(), "Reading");
    /// ```
    #[must_use]
    fn state_name(&self) -> &'static str;
}
