//! The virtual members of `ChFiDS_Spine`, which the fillet and chamfer spines override.
use super::Spine;
use crate::chfi_ds::shared::Shared;
use crate::chfi_ds::{ChamfSpine, ElSpine, FilSpine};
use rustycad_tkernel::standard::{Failure, Handle, Transient};
use std::ops::{Deref, DerefMut};

/// A shared, mutable `occ::handle<ChFiDS_Spine>` whose object may be a [`Spine`], a
/// [`FilSpine`] or a [`ChamfSpine`]; a null native handle is `None`.
pub type SpineHandle = Shared<dyn SpineObject>;

/// The polymorphic interface of a guideline (the virtual members of `ChFiDS_Spine`).
///
/// Through `Deref`, a `dyn SpineObject` also offers every non-virtual [`Spine`] member;
/// the virtual ones are only here, so a call always dispatches to the override.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::{FilSpine, SpineHandle, SpineObject, lock, shared};
/// let spine: SpineHandle = shared(FilSpine::new());
/// assert!(lock(&spine).as_fil_spine().is_some());
/// assert_eq!(lock(&spine).nb_edges(), 0);
/// ```
pub trait SpineObject: Transient {
    /// The `ChFiDS_Spine` part of the object.
    fn spine(&self) -> &Spine;
    /// The mutable `ChFiDS_Spine` part of the object.
    fn spine_mut(&mut self) -> &mut Spine;
    /// `Reset(AllData)`: forgets the split (and, with `all_data`, the extensions);
    /// Standard_NullObject when `all_data` before Load; `spine.reset(false)?`.
    fn reset(&mut self, all_data: bool) -> Result<(), Failure>;
    /// `AppendElSpine(Els)`; `spine.append_el_spine(&el_spine)?`.
    fn append_el_spine(&mut self, el_spine: &Handle<ElSpine>) -> Result<(), Failure>;
    /// `AppendOffsetElSpine(Els)`; `spine.append_offset_el_spine(&el_spine)`.
    fn append_offset_el_spine(&mut self, el_spine: &Handle<ElSpine>) {
        self.spine_mut().offset_el_spines.push(el_spine.clone());
    }
    /// The object as a fillet spine (`occ::down_cast<ChFiDS_FilSpine>`).
    fn as_fil_spine(&self) -> Option<&FilSpine> {
        None
    }
    /// The object as a mutable fillet spine.
    fn as_fil_spine_mut(&mut self) -> Option<&mut FilSpine> {
        None
    }
    /// The object as a chamfer spine (`occ::down_cast<ChFiDS_ChamfSpine>`).
    fn as_chamf_spine(&self) -> Option<&ChamfSpine> {
        None
    }
    /// The object as a mutable chamfer spine.
    fn as_chamf_spine_mut(&mut self) -> Option<&mut ChamfSpine> {
        None
    }
}

impl SpineObject for Spine {
    fn spine(&self) -> &Spine {
        self
    }
    fn spine_mut(&mut self) -> &mut Spine {
        self
    }
    fn reset(&mut self, all_data: bool) -> Result<(), Failure> {
        self.split_done = false;
        self.el_spines.clear();
        if all_data {
            self.ends.first_param = 0.;
            self.ends.last_param = self.total_length()?;
            self.ends.first_prolon = false;
            self.ends.last_prolon = false;
        }
        Ok(())
    }
    fn append_el_spine(&mut self, el_spine: &Handle<ElSpine>) -> Result<(), Failure> {
        self.el_spines.push(el_spine.clone());
        Ok(())
    }
}

impl Deref for dyn SpineObject {
    type Target = Spine;
    fn deref(&self) -> &Spine {
        self.spine()
    }
}

impl DerefMut for dyn SpineObject {
    fn deref_mut(&mut self) -> &mut Spine {
        self.spine_mut()
    }
}
