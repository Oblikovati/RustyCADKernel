//! Safe borrowed or shared curve ownership without resetting projector caches.
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use std::sync::Arc;
pub(super) trait CurveOwner<C: ?Sized> {
    fn curve(&self) -> &C;
}
impl<C: ?Sized + RuntimeTyped> CurveOwner<C> for Handle<C> {
    fn curve(&self) -> &C {
        self.get()
            .expect("null curve owner; expected validated live handle")
    }
}
pub(super) enum CurveReference<'a, C: ?Sized> {
    Borrowed(&'a C),
    Owned(Arc<dyn CurveOwner<C> + Send + Sync>),
}
impl<C: ?Sized> Clone for CurveReference<'_, C> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(curve) => Self::Borrowed(curve),
            Self::Owned(owner) => Self::Owned(owner.clone()),
        }
    }
}
impl<C: ?Sized> CurveReference<'_, C> {
    pub(super) fn get(&self) -> &C {
        match self {
            Self::Borrowed(curve) => curve,
            Self::Owned(owner) => owner.curve(),
        }
    }
}
impl<C: ?Sized + RuntimeTyped + Send + Sync> CurveReference<'_, C> {
    pub(super) fn owned(curve: Handle<C>) -> Result<Self, Failure> {
        if curve.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "null curve; expected live projector adaptor",
            ));
        }
        Ok(Self::Owned(Arc::new(curve)))
    }
}
