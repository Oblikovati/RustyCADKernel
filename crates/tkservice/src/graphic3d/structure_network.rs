//! The structure network of `Graphic3d_Structure`: ancestors and descendants, `Connect`,
//! `Disconnect`, `Network` and the removal of a structure from it.
use super::structure::Graphic3dStructureFields;
use super::{
    Graphic3dCStructureMembers as _, Graphic3dGraphicDriverMembers as _,
    Graphic3dStructureMembers as _, Graphic3dStructureSource, Graphic3dTypeOfConnection,
};
use rustycad_tkernel::standard::{Handle, WeakHandle};

/// The address of a structure, as the native `Graphic3d_Structure*` identity.
pub(crate) fn address_of<S: Graphic3dStructureSource + ?Sized>(structure: &S) -> *const () {
    std::ptr::from_ref(structure).cast::<()>()
}

/// `NCollection_IndexedMap::Add`: appends a new pointer; false when already present.
fn append_unique(
    list: &mut Vec<WeakHandle<dyn Graphic3dStructureSource>>,
    item: &Handle<dyn Graphic3dStructureSource>,
) -> bool {
    if list
        .iter()
        .any(|known| known.object_address() == item.object_address())
    {
        return false;
    }
    list.push(item.downgrade());
    true
}

/// `FindIndex`, `Swap` with the last and `RemoveLast`: removes a pointer; false when absent.
pub(crate) fn swap_remove(
    list: &mut Vec<WeakHandle<dyn Graphic3dStructureSource>>,
    address: *const (),
) -> bool {
    let Some(index) = list
        .iter()
        .position(|known| known.object_address() == address)
    else {
        return false;
    };
    list.swap_remove(index);
    true
}

/// The `Remove()` steps after the groups are cleared: unlinks the structure from its
/// descendants and ancestors, frees its identification and removes its driver structure.
pub(crate) fn remove_fields(fields: &mut Graphic3dStructureFields, address: *const ()) {
    let Some(cstructure) = fields.cstructure.get() else {
        return;
    };
    let unlink = |list: &[WeakHandle<dyn Graphic3dStructureSource>],
                  connection: Graphic3dTypeOfConnection| {
        for other in list.iter().map(WeakHandle::upgrade) {
            if let Some(other) = other.get() {
                other.remove_link(address, connection);
            }
        }
    };
    unlink(&fields.descendants, Graphic3dTypeOfConnection::Ancestor);
    unlink(&fields.ancestors, Graphic3dTypeOfConnection::Descendant);
    let (id, driver) = (cstructure.identification(), cstructure.graphic_driver());
    if let Some(driver) = driver.get() {
        driver.remove_identification(id);
        let mut removed = fields.cstructure.clone();
        driver.remove_structure(&mut removed);
    }
    fields.cstructure = Handle::default();
}

/// The structure network members of every structure.
pub trait Graphic3dStructureNetwork: Graphic3dStructureSource {
    /// `Remove()`: clears the groups, unlinks the structure and removes its driver
    /// structure; the structure is deleted afterwards; `structure.remove()`.
    fn remove(&self) {
        if self.is_deleted() {
            return;
        }
        for group in self.groups().iter().filter_map(Handle::get) {
            group.clear(false);
        }
        let mut fields = self.structure_base().fields.read().clone();
        remove_fields(&mut fields, address_of(self));
        self.structure_base().fields.write().cstructure = Handle::default();
    }
    /// `Remove(Graphic3d_Structure*, Graphic3d_TypeOfConnection)`: forgets a descendant or an
    /// ancestor by address; `structure.remove_link(other.object_address(), Graphic3dTypeOfConnection::Descendant)`.
    fn remove_link(&self, address: *const (), connection: Graphic3dTypeOfConnection) {
        let mut fields = self.structure_base().fields.write();
        match connection {
            Graphic3dTypeOfConnection::Descendant => swap_remove(&mut fields.descendants, address),
            Graphic3dTypeOfConnection::Ancestor => swap_remove(&mut fields.ancestors, address),
        };
    }
    /// `AppendAncestor`; `structure.append_ancestor(&parent)`.
    fn append_ancestor(&self, ancestor: &Handle<dyn Graphic3dStructureSource>) -> bool {
        append_unique(
            &mut self.structure_base().fields.write().ancestors,
            ancestor,
        )
    }
    /// `AppendDescendant`; `structure.append_descendant(&child)`.
    fn append_descendant(&self, descendant: &Handle<dyn Graphic3dStructureSource>) -> bool {
        append_unique(
            &mut self.structure_base().fields.write().descendants,
            descendant,
        )
    }
    /// `RemoveAncestor`; `structure.remove_ancestor(&parent)`.
    fn remove_ancestor(&self, ancestor: &Handle<dyn Graphic3dStructureSource>) -> bool {
        swap_remove(
            &mut self.structure_base().fields.write().ancestors,
            ancestor.object_address(),
        )
    }
    /// `RemoveDescendant`; `structure.remove_descendant(&child)`.
    fn remove_descendant(&self, descendant: &Handle<dyn Graphic3dStructureSource>) -> bool {
        swap_remove(
            &mut self.structure_base().fields.write().descendants,
            descendant.object_address(),
        )
    }
    /// `Ancestors`: adds the parents to `set`; `structure.ancestors(&mut set)`.
    fn ancestors(&self, set: &mut Vec<Handle<dyn Graphic3dStructureSource>>) {
        let ancestors = self.structure_base().fields.read().ancestors.clone();
        add_all(set, &ancestors);
    }
    /// `Descendants`: adds the children to `set`; `structure.descendants(&mut set)`.
    fn descendants(&self, set: &mut Vec<Handle<dyn Graphic3dStructureSource>>) {
        let descendants = self.structure_base().fields.read().descendants.clone();
        add_all(set, &descendants);
    }
    /// `Connect`: links `other` as a descendant or an ancestor (both ways), recomputes the
    /// box, and for a descendant connects the driver structures and updates the views;
    /// with the check, a connection that would make a cycle is ignored.
    /// Example: `parent.connect(&child, Graphic3dTypeOfConnection::Descendant, false)`.
    fn connect(
        &self,
        other: &Handle<dyn Graphic3dStructureSource>,
        connection: Graphic3dTypeOfConnection,
        with_check: bool,
    ) {
        let this = self.structure_base().this();
        if self.is_deleted()
            || other.is_null()
            || (with_check && !graphic3d_structure_accept_connection(&this, other, connection))
        {
            return;
        }
        let Some(other_ref) = other.get() else { return };
        if connection == Graphic3dTypeOfConnection::Ancestor {
            if self.append_ancestor(other) {
                self.calculate_bound_box();
                other_ref.connect(&this, Graphic3dTypeOfConnection::Descendant, false);
            }
            return;
        }
        if !self.append_descendant(other) {
            return;
        }
        self.calculate_bound_box();
        other_ref.connect(&this, Graphic3dTypeOfConnection::Ancestor, false);
        self.graphic_connect(other);
        if let Some(manager) = self.structure_manager().get() {
            manager.connect(&this, other);
        }
        self.update(true);
    }
    /// `Disconnect`: unlinks `other` (descendant or ancestor) both ways and recomputes the box;
    /// `parent.disconnect(&child)`.
    fn disconnect(&self, other: &Handle<dyn Graphic3dStructureSource>) {
        let this = self.structure_base().this();
        let Some(other_ref) = other.get().filter(|_| !self.is_deleted()) else {
            return;
        };
        if self.remove_descendant(other) {
            other_ref.disconnect(&this);
            self.graphic_disconnect(other);
            if let Some(manager) = self.structure_manager().get() {
                manager.disconnect(&this, other);
            }
            self.calculate_bound_box();
            self.update(true);
        } else if self.remove_ancestor(other) {
            other_ref.disconnect(&this);
            self.calculate_bound_box();
        }
    }
    /// `DisconnectAll`: disconnects every descendant or every ancestor, always the first one
    /// (the native loop runs over the initial count); `structure.disconnect_all(Graphic3dTypeOfConnection::Descendant)`.
    fn disconnect_all(&self, connection: Graphic3dTypeOfConnection) {
        if self.is_deleted() {
            return;
        }
        let this = self.structure_base().this();
        let first = || {
            let fields = self.structure_base().fields.read();
            let list = if connection == Graphic3dTypeOfConnection::Descendant {
                &fields.descendants
            } else {
                &fields.ancestors
            };
            (
                list.len(),
                list.first().map(WeakHandle::upgrade).unwrap_or_default(),
            )
        };
        for _ in 0..first().0 {
            if let Some(other) = first().1.get() {
                other.disconnect(&this);
            }
        }
    }
}
impl<T: Graphic3dStructureSource + ?Sized> Graphic3dStructureNetwork for T {}

/// Adds the live structures of `list` to `set` once each (`NCollection_Map::Add`).
fn add_all(
    set: &mut Vec<Handle<dyn Graphic3dStructureSource>>,
    list: &[WeakHandle<dyn Graphic3dStructureSource>],
) {
    for item in list
        .iter()
        .map(WeakHandle::upgrade)
        .filter(|h| !h.is_null())
    {
        if !set
            .iter()
            .any(|known| known.object_address() == item.object_address())
        {
            set.push(item);
        }
    }
}

/// `Network`: `structure` and every structure reachable through the descendants or the
/// ancestors, depth first; `graphic3d_structure_network(&s, Graphic3dTypeOfConnection::Descendant, &mut set)`.
pub fn graphic3d_structure_network(
    structure: &Handle<dyn Graphic3dStructureSource>,
    connection: Graphic3dTypeOfConnection,
    set: &mut Vec<Handle<dyn Graphic3dStructureSource>>,
) {
    let Some(value) = structure.get() else { return };
    // A structure already in the set is not visited again: the result is the native one,
    // and a cyclic network (where the native recursion never ends) terminates.
    if set
        .iter()
        .any(|known| known.object_address() == structure.object_address())
    {
        return;
    }
    set.push(structure.clone());
    let next = {
        let fields = value.structure_base().fields.read();
        match connection {
            Graphic3dTypeOfConnection::Descendant => fields.descendants.clone(),
            Graphic3dTypeOfConnection::Ancestor => fields.ancestors.clone(),
        }
    };
    for item in next.iter().map(WeakHandle::upgrade) {
        graphic3d_structure_network(&item, connection, set);
    }
}

/// `AcceptConnection`: false when `first` is in the network of `second`;
/// `graphic3d_structure_accept_connection(&a, &b, Graphic3dTypeOfConnection::Descendant)`.
pub fn graphic3d_structure_accept_connection(
    first: &Handle<dyn Graphic3dStructureSource>,
    second: &Handle<dyn Graphic3dStructureSource>,
    connection: Graphic3dTypeOfConnection,
) -> bool {
    let mut set = Vec::new();
    graphic3d_structure_network(second, connection, &mut set);
    !set.iter()
        .any(|known| known.object_address() == first.object_address())
}

/// `PrintNetwork`: the identification lines of the network ("\tIdent <id>\n", in the set
/// order); `graphic3d_structure_print_network(&s, Graphic3dTypeOfConnection::Descendant)`.
pub fn graphic3d_structure_print_network(
    structure: &Handle<dyn Graphic3dStructureSource>,
    connection: Graphic3dTypeOfConnection,
) -> String {
    let mut set = Vec::new();
    graphic3d_structure_network(structure, connection, &mut set);
    set.iter()
        .filter_map(Handle::get)
        .map(|s| format!("\tIdent {}\n", s.identification()))
        .collect()
}
