//! Native const handle slots are read anew across valid application callbacks.
use super::StdPrsIsolineSurfaceSource;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, SharedState};
pub(super) enum IsolineSurfaceReference<'a, S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized> {
    Borrowed(&'a Handle<S>),
    Live(&'a SharedState<Handle<S>>),
}
impl<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized> IsolineSurfaceReference<'_, S> {
    pub(super) fn snapshot(&self) -> Handle<S> {
        // A method callback may replace a live slot; never retain a lock during virtual dispatch.
        match self {
            Self::Borrowed(source) => (*source).clone(),
            Self::Live(source) => source.snapshot(),
        }
    }
}
