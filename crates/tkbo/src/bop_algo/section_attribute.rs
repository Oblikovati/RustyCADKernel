//! Intersection flags from BOPAlgo_SectionAttribute.hxx at OCCT 3d097a0.

/// Independent approximation and surface p-curve flags; `SectionAttribute::new()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SectionAttribute {
    approximation: bool,
    pcurve_on_s1: bool,
    pcurve_on_s2: bool,
}

impl Default for SectionAttribute {
    fn default() -> Self {
        Self::new()
    }
}

impl SectionAttribute {
    /// Enables all three native default flags; `SectionAttribute::new()`.
    pub const fn new() -> Self {
        Self::with_flags(true, true, true)
    }

    /// Assigns each flag independently; `SectionAttribute::with_flags(false, true, false)`.
    pub const fn with_flags(approximation: bool, pcurve_on_s1: bool, pcurve_on_s2: bool) -> Self {
        Self {
            approximation,
            pcurve_on_s1,
            pcurve_on_s2,
        }
    }

    /// Reads approximation policy; `SectionAttribute::new().approximation()`.
    pub const fn approximation(&self) -> bool {
        self.approximation
    }

    /// Reads first-surface p-curve policy; `SectionAttribute::new().pcurve_on_s1()`.
    pub const fn pcurve_on_s1(&self) -> bool {
        self.pcurve_on_s1
    }

    /// Reads second-surface p-curve policy; `SectionAttribute::new().pcurve_on_s2()`.
    pub const fn pcurve_on_s2(&self) -> bool {
        self.pcurve_on_s2
    }

    /// Changes only approximation policy; `flags.set_approximation(false)`.
    pub fn set_approximation(&mut self, enabled: bool) {
        self.approximation = enabled;
    }

    /// Changes only first-surface p-curve policy; `flags.set_pcurve_on_s1(false)`.
    pub fn set_pcurve_on_s1(&mut self, enabled: bool) {
        self.pcurve_on_s1 = enabled;
    }

    /// Changes only second-surface p-curve policy; `flags.set_pcurve_on_s2(false)`.
    pub fn set_pcurve_on_s2(&mut self, enabled: bool) {
        self.pcurve_on_s2 = enabled;
    }
}
