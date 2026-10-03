/// The target lacks OCCT's POSIX thread-locale API, so this scope is descriptive only.
#[derive(Debug)]
pub(super) struct ThreadLocaleScope;

impl ThreadLocaleScope {
    pub(super) fn enter() -> Self {
        Self
    }
}
