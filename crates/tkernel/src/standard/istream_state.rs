//! Exception masks and failures for native-protocol `std::istream` reads.

/// Selects stream-state bits that make a native read throw; `IstreamExceptionMask::new(false, true, false)`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IstreamExceptionMask {
    eof: bool,
    fail: bool,
    bad: bool,
}

impl IstreamExceptionMask {
    /// Builds an exception mask in eof/fail/bad order; `IstreamExceptionMask::new(false, true, false)`.
    pub const fn new(eof: bool, fail: bool, bad: bool) -> Self {
        Self { eof, fail, bad }
    }

    /// Returns whether EOF triggers an exception; `mask.eof()`.
    pub const fn eof(&self) -> bool {
        self.eof
    }

    /// Returns whether failbit triggers an exception; `mask.fail()`.
    pub const fn fail(&self) -> bool {
        self.fail
    }

    /// Returns whether badbit triggers an exception; `mask.bad()`.
    pub const fn bad(&self) -> bool {
        self.bad
    }
}

/// Captures the native state and gcount when a masked stream operation throws.
#[derive(Debug)]
pub struct IstreamException {
    pub(super) eof: bool,
    pub(super) fail: bool,
    pub(super) bad: bool,
    pub(super) gcount: usize,
    pub(super) source_error: Option<std::io::Error>,
}

impl IstreamException {
    /// Returns the native EOF bit after the operation; `error.eof()`.
    pub const fn eof(&self) -> bool {
        self.eof
    }

    /// Returns the native fail() value, including badbit; `error.fail()`.
    pub const fn fail(&self) -> bool {
        self.fail || self.bad
    }

    /// Returns the native badbit; `error.bad()`.
    pub const fn bad(&self) -> bool {
        self.bad
    }

    /// Returns the stream's native gcount even when the caller's output was not assigned;
    /// `error.gcount()`.
    pub const fn gcount(&self) -> usize {
        self.gcount
    }

    /// Borrows the original Rust source error rethrown by a masked badbit read;
    /// `error.source_error().and_then(std::io::Error::get_ref)`.
    pub fn source_error(&self) -> Option<&std::io::Error> {
        self.source_error.as_ref()
    }
}

impl std::fmt::Display for IstreamException {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "istream exception: eof={}, fail={}, bad={}, gcount={}",
            self.eof, self.fail, self.bad, self.gcount
        )
    }
}

impl std::error::Error for IstreamException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source_error
            .as_ref()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

/// Result metadata after an unmasked native-protocol read; `result.gcount()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IstreamReadResult {
    pub(super) gcount: usize,
    pub(super) good: bool,
}

impl IstreamReadResult {
    /// Returns the bytes counted by the native read; `result.gcount()`.
    pub const fn gcount(self) -> usize {
        self.gcount
    }

    /// Returns whether badbit stayed clear; `result.is_good()`.
    pub const fn is_good(self) -> bool {
        self.good
    }
}
