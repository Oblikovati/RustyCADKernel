//! Actual generic action slots remain live during overridable session callbacks.
use super::{
    AspectXRAction, AspectXRGenericAction, AspectXRSession, AspectXRSessionFields,
    AspectXRTrackedDeviceRole,
};
use rustycad_tkernel::standard::{
    Failure, Handle, HandleObserver, HandleReference, SharedState, raise_native_failure,
};
/// Native action-reference adapter; `AspectXrActionRef::borrowed(&action)` wraps caller storage.
pub enum AspectXrActionRef<'a> {
    /// Ordinary caller-owned handle slot.
    Borrowed(&'a dyn HandleReference<AspectXRAction>),
    /// Actual session role/action field, without an extra action owner or retained lock.
    Generic(
        &'a SharedState<AspectXRSessionFields>,
        AspectXRTrackedDeviceRole,
        usize,
        usize,
    ),
}
impl<'a> AspectXrActionRef<'a> {
    /// Retain actual argument identity; `AspectXrActionRef::borrowed(&action)`.
    pub fn borrowed(action: &'a dyn HandleReference<AspectXRAction>) -> Self {
        // Caller ownership keeps this ordinary native handle reference valid.
        Self::Borrowed(action)
    }
    /// Borrow the generic slot itself; `AspectXrActionRef::generic(&session,role,action)`.
    pub fn generic(
        session: &'a dyn AspectXRSession,
        role: AspectXRTrackedDeviceRole,
        action: AspectXRGenericAction,
    ) -> Result<Self, Failure> {
        let fields = &session.xr_session_base().fields;
        Self::from_fields(fields, role, action)
    }
    pub(super) fn from_fields(
        fields: &'a SharedState<AspectXRSessionFields>,
        role: AspectXRTrackedDeviceRole,
        action: AspectXRGenericAction,
    ) -> Result<Self, Failure> {
        let actual = fields.read();
        let actions = &actual.role_actions[role as usize];
        let slot = actions
            .value(action as i32)
            .map_err(|error| error.into_failure("Aspect_XRSession::GenericAction"))?;
        let offset = (i64::from(action as i32) - i64::from(actions.lower())) as usize;
        Ok(Self::Generic(
            fields,
            role,
            offset,
            std::ptr::from_ref(slot) as usize,
        ))
    }
    /// Observe current assignment with unchanged native refcount; `reference.observe()`.
    pub fn observe(&self) -> HandleObserver<AspectXRAction> {
        match self {
            Self::Borrowed(action) => action.observe(),
            Self::Generic(fields, role, offset, address) => {
                let actual = fields.read();
                raise_native_failure(retained_generic_action(&actual, *role, *offset, *address))
                    .observe()
            }
        }
    }
    /// Inspect current null state; `reference.is_null()`.
    pub fn is_null(&self) -> bool {
        let observed = self.observe();
        observed.is_null()
    }
    /// Observe target identity after valid callback replacement; `reference.object_address()`.
    pub fn object_address(&self) -> *const () {
        let observed = self.observe();
        observed.object_address()
    }
    /// Compare actual handle-slot references; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &AspectXrActionRef<'_>) -> bool {
        let left = self.reference_address();
        let right = other.reference_address();
        left == right
    }
    /// Identify the actual argument handle slot; `reference.reference_address()` remains stable across valid rebasing.
    pub fn reference_address(&self) -> *const Handle<AspectXRAction> {
        match self {
            Self::Borrowed(action) => action.reference_address().cast(),
            Self::Generic(fields, role, offset, address) => {
                let actual = fields.read();
                std::ptr::from_ref(raise_native_failure(retained_generic_action(
                    &actual, *role, *offset, *address,
                )))
            }
        }
    }
}

fn retained_generic_action(
    fields: &AspectXRSessionFields,
    role: AspectXRTrackedDeviceRole,
    offset: usize,
    address: usize,
) -> Result<&Handle<AspectXRAction>, Failure> {
    let slot = fields.role_actions[role as usize]
        .at(offset)
        .map_err(|error| error.into_failure("GenericAction retained offset"))?;
    let current = std::ptr::from_ref(slot) as usize;
    if current == address {
        return Ok(slot);
    }
    Err(Failure::new(&format!(
        "GenericAction role={role:?} offset={offset} address={current:#x}; expected retained slot={address:#x} without reallocation"
    )))
}
