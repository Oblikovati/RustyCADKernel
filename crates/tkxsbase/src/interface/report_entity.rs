// Rust port of DataExchange/TKXSBase/Interface/Interface_ReportEntity (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_ReportEntity`: the report of an entity of a model: its check, and the
//! content that replaces it when it could not be loaded normally.
use super::InterfaceCheck;
use rustycad_tkernel::{
    impl_transient,
    standard::{Handle, SharedState, StandardTransient},
};

/// The fields of a report.
#[derive(Clone, Default)]
struct ReportState {
    thecheck: Handle<InterfaceCheck>,
    theconcerned: Handle<StandardTransient>,
    thecontent: Handle<StandardTransient>,
}

/// `Interface_ReportEntity`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceReportEntity;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let unknown = Handle::new(StandardTransient::default());
/// let report = InterfaceReportEntity::unknown(&unknown);
/// assert!(report.has_content() && !report.has_new_content());
/// ```
#[derive(Default)]
pub struct InterfaceReportEntity {
    transient: StandardTransient,
    state: SharedState<ReportState>,
}
impl_transient!(
    InterfaceReportEntity,
    "Interface_ReportEntity",
    StandardTransient,
    transient
);

impl InterfaceReportEntity {
    /// `Interface_ReportEntity(unknown)`: an unknown entity, its own content, no check
    /// (natively a null check, which `IsError` and `IsUnknown` dereference; the port reads
    /// it as empty).
    pub fn unknown(unknown: &Handle<StandardTransient>) -> Self {
        Self::from_state(ReportState {
            thecheck: Handle::default(),
            theconcerned: unknown.clone(),
            thecontent: unknown.clone(),
        })
    }
    /// `Interface_ReportEntity(acheck, concerned)`: the check gets the entity.
    pub fn with_check(
        acheck: &Handle<InterfaceCheck>,
        concerned: &Handle<StandardTransient>,
    ) -> Self {
        if let Some(check) = acheck.get() {
            check.set_entity(concerned);
        }
        Self::from_state(ReportState {
            thecheck: acheck.clone(),
            theconcerned: concerned.clone(),
            thecontent: Handle::default(),
        })
    }
    fn from_state(state: ReportState) -> Self {
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `SetContent(content)`.
    pub fn set_content(&self, content: &Handle<StandardTransient>) {
        self.state.write().thecontent = content.clone();
    }
    /// `Check`.
    pub fn check(&self) -> Handle<InterfaceCheck> {
        self.state.read().thecheck.clone()
    }
    /// `CCheck`: replaces the check (natively a modifiable reference to the handle).
    pub fn set_check(&self, check: &Handle<InterfaceCheck>) {
        self.state.write().thecheck = check.clone();
    }
    /// `Concerned`.
    pub fn concerned(&self) -> Handle<StandardTransient> {
        self.state.read().theconcerned.clone()
    }
    /// `HasContent`.
    pub fn has_content(&self) -> bool {
        self.state.read().thecontent.is_some()
    }
    /// `HasNewContent`: a content other than the concerned entity.
    pub fn has_new_content(&self) -> bool {
        let state = self.state.read();
        state.thecontent.is_some() && state.thecontent != state.theconcerned
    }
    /// `Content`.
    pub fn content(&self) -> Handle<StandardTransient> {
        self.state.read().thecontent.clone()
    }
    /// `IsError`: the check has fails.
    pub fn is_error(&self) -> bool {
        self.check().get().is_some_and(|c| c.nb_fails() > 0)
    }
    /// `IsUnknown`: no fail, no warning, and the content is the concerned entity.
    pub fn is_unknown(&self) -> bool {
        let state = self.state.read();
        let (fails, warnings) = state
            .thecheck
            .get()
            .map_or((0, 0), |c| (c.nb_fails(), c.nb_warnings()));
        fails == 0 && warnings == 0 && state.theconcerned == state.thecontent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_with_checks() {
        let concerned = Handle::new(StandardTransient::default());
        let check = Handle::new(InterfaceCheck::new());
        let report = InterfaceReportEntity::with_check(&check, &concerned);
        assert!(check.entity() == concerned);
        assert!(!report.has_content() && !report.is_unknown());
        check.add_fail_text("bad", "");
        assert!(report.is_error());
        let content = Handle::new(StandardTransient::default());
        report.set_content(&content);
        assert!(report.has_new_content() && report.content() == content);
        assert!(report.concerned() == concerned);
        let other = Handle::new(InterfaceCheck::new());
        report.set_check(&other);
        assert!(report.check() == other && !report.is_error());
        let unknown = InterfaceReportEntity::unknown(&concerned);
        assert!(unknown.is_unknown() && !unknown.is_error());
    }
}
