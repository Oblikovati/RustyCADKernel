// Rust port of DataExchange/TKXSBase/Interface/Interface_CopyControl (.hxx, .cxx) and
// Interface_CopyMap (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_CopyControl`, the map of the copied entities a copy tool consults, and its
//! default implementation `Interface_CopyMap`, indexed by entity number.
use super::errors::interface_error;
use super::{InterfaceInterfaceModel, InterfaceModelEntities};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient, exceptions},
};

/// `Interface_CopyControl`, abstract.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceCopyControl;
/// fn known(map: &dyn InterfaceCopyControl, ent: &rustycad_tkernel::standard::Handle<rustycad_tkernel::standard::StandardTransient>) -> bool {
///     let mut res = Default::default();
///     map.search(ent, &mut res).unwrap_or(false)
/// }
/// ```
pub trait InterfaceCopyControl: Transient {
    /// `Clear`.
    fn clear(&self);
    /// `Bind(ent, res)`.
    fn bind(
        &self,
        ent: &Handle<StandardTransient>,
        res: &Handle<StandardTransient>,
    ) -> Result<(), Failure>;
    /// `Search(ent, res)`: whether `ent` was copied, and its copy.
    fn search(
        &self,
        ent: &Handle<StandardTransient>,
        res: &mut Handle<StandardTransient>,
    ) -> Result<bool, Failure>;
}
impl_transient_interface!(
    dyn InterfaceCopyControl,
    "Interface_CopyControl",
    StandardTransient
);

/// `Interface_CopyMap`: the copies of the entities of a model, by entity number.
pub struct InterfaceCopyMap {
    transient: StandardTransient,
    themod: Handle<dyn InterfaceInterfaceModel>,
    theres: SharedState<Vec<Handle<StandardTransient>>>,
}
impl_transient!(InterfaceCopyMap, "Interface_CopyMap", dyn InterfaceCopyControl, transient, interfaces: [dyn InterfaceCopyControl]);

impl InterfaceCopyMap {
    /// `Interface_CopyMap(amodel)`: a place per entity (0..NbEntities).
    pub fn new(amodel: &Handle<dyn InterfaceInterfaceModel>) -> Self {
        let nb = amodel.get().map_or(0, |m| m.nb_entities());
        Self {
            transient: StandardTransient::default(),
            themod: amodel.clone(),
            theres: SharedState::new(vec![Handle::default(); (nb + 1) as usize]),
        }
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.themod.clone()
    }
    fn number(&self, ent: &Handle<StandardTransient>) -> i32 {
        self.themod.get().map_or(0, |m| m.number(ent))
    }
}

impl InterfaceCopyControl for InterfaceCopyMap {
    /// `Clear`: forgets every copy.
    fn clear(&self) {
        let mut theres = self.theres.write();
        theres.iter_mut().skip(1).for_each(Handle::nullify);
    }
    /// `Bind(ent, res)`: `Interface_InterfaceError` for an entity out of the model or
    /// already bound.
    fn bind(
        &self,
        ent: &Handle<StandardTransient>,
        res: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let num = self.number(ent);
        let mut theres = self.theres.write();
        let upper = theres.len() as i32 - 1;
        if num == 0 || num > upper {
            return Err(interface_error(
                "CopyMap : Bind, Starting Entity not issued from Starting Model",
            ));
        }
        let slot = &mut theres[num as usize];
        if slot.is_some() {
            return Err(interface_error(
                "CopyMap : Bind, Starting Entity already bound",
            ));
        }
        *slot = res.clone();
        Ok(())
    }
    /// `Search(ent, res)`: `Standard_OutOfRange` for an entity added to the model after
    /// the map was made (beyond the places).
    fn search(
        &self,
        ent: &Handle<StandardTransient>,
        res: &mut Handle<StandardTransient>,
    ) -> Result<bool, Failure> {
        let num = self.number(ent);
        if num == 0 {
            return Ok(false);
        }
        let theres = self.theres.read();
        let found = theres.get(num as usize).ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "Interface_CopyMap::Search: entity {num}; expected [0, {}]",
                theres.len() - 1
            ))
        })?;
        *res = found.clone();
        Ok(res.is_some())
    }
}
