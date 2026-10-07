// Rust port of the TDF_Attribute virtual members of TNaming_NamedShape
// (ApplicationFramework/TKCAF/TNaming/TNaming_NamedShape.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The framework side of `TNaming_NamedShape`: backup, restore, deltas, copy and undo.
use super::named_shape::{NamedShapeState, used_shapes_of};
use super::node::same_pointer;
use super::node_chains::remove_nodes;
use super::{
    TNamingBuilder, TNamingCopyShape, TNamingDeltaOnModification, TNamingDeltaOnRemoval,
    TNamingEvolution, TNamingIterator, TNamingNamedShape, TNamingNodePtr,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkcdf::json_dump::{class_begin, field_integer, field_string};
use rustycad_tkernel::standard::{
    Failure, Guid, Handle, RuntimeTyped, WeakHandle, dump::DumpStream, exceptions,
};
use rustycad_tklcaf::json::{dump_attribute_json_base, dump_label_entry};
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdata_std::support::cast_attribute;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfAttributeState, TdfDataSet,
    TdfDeltaOnAddition, TdfDeltaOnModification, TdfDeltaOnRemoval, TdfRelocationTable,
};
use std::io;

impl TNamingNamedShape {
    /// Points every node of the chain at `owner` (the `myAtt` update of `BackupCopy` and
    /// `Restore`).
    fn adopt_nodes(first: &TNamingNodePtr, owner: &WeakHandle<TNamingNamedShape>) {
        let mut current = first.clone();
        while let Some(node) = current {
            node.edit(|links| links.att = owner.clone());
            current = node.next_same_attribute();
        }
    }
    /// The body of `AfterUndo` on an undone addition: the evolutions leave the shape
    /// chains and are deleted, as `Clear` does, with the root of the delta's label.
    fn forget_evolutions(&self, delta: &dyn TdfAttributeDelta) -> Result<(), Failure> {
        let used = used_shapes_of(&delta.label())?;
        let first = self.first_node();
        remove_nodes(used.get(), &first);
        self.state.write().node = None;
        Ok(())
    }
}

impl TdfAttribute for TNamingNamedShape {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        Self::get_id()
    }
    /// `BackupCopy`: the copy takes the nodes (so the shape chains still reach them, now
    /// through the copy) and this attribute keeps none.
    fn backup_copy(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        let copy = Handle::new(TNamingNamedShape::new());
        let taken = {
            let mut state = self.state.write();
            let taken = state.clone();
            state.node = None;
            taken
        };
        let copy_ref = live(&copy, "TNaming_NamedShape::BackupCopy")?;
        Self::adopt_nodes(&taken.node, &copy.downgrade());
        *copy_ref.state.write() = taken;
        Ok(copy.cast())
    }
    /// `Restore`: clears this attribute, then takes the nodes of `anAttribute` (a node is
    /// in one attribute only).
    fn restore(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        self.clear()?;
        let other = cast_attribute::<TNamingNamedShape>(attribute, "TNaming_NamedShape::Restore")?;
        let other = live(&other, "TNaming_NamedShape::Restore")?;
        let taken: NamedShapeState = {
            let mut state = other.state.write();
            let taken = state.clone();
            state.node = None;
            taken
        };
        Self::adopt_nodes(&taken.node, &self.this_handle()?.downgrade());
        *self.state.write() = taken;
        Ok(())
    }
    /// `DeltaOnModification(anOldAttribute)`: a `TNaming_DeltaOnModification` that saves the
    /// old attribute's evolutions.
    fn delta_on_modification(
        &self,
        old_attribute: &Handle<dyn TdfAttribute>,
    ) -> Result<Handle<dyn TdfDeltaOnModification>, Failure> {
        let old = old_attribute.cast::<TNamingNamedShape>();
        Ok(TNamingDeltaOnModification::new(&old)?.cast())
    }
    /// `DeltaOnModification(aDelta)`: the delta applies (no backup, unlike the default).
    fn apply_delta_on_modification(
        &self,
        delta: &Handle<dyn TdfDeltaOnModification>,
    ) -> Result<(), Failure> {
        live(delta, "TNaming_NamedShape::DeltaOnModification")?.apply()
    }
    /// `DeltaOnRemoval`: a `TNaming_DeltaOnRemoval` on this attribute.
    fn delta_on_removal(&self) -> Result<Handle<dyn TdfDeltaOnRemoval>, Failure> {
        Ok(TNamingDeltaOnRemoval::new(&self.this_handle()?)?.cast())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(TNamingNamedShape::new()).cast()
    }
    /// `Paste`: rebuilds the evolutions on the target label with copies of the shapes
    /// (through the relocation table's transient map).
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let label = live(into, "TNaming_NamedShape::Paste")?.label();
        if label.is_null() {
            return Err(exceptions::null_object(
                "TNaming_NamedShape::Paste: the target attribute has no label; expected an attached attribute",
            ));
        }
        let table = live(relocation_table, "TNaming_NamedShape::Paste")?;
        let builder = TNamingBuilder::new(&label)?;
        let mut iterator = TNamingIterator::of(self);
        while iterator.more() {
            let evolution = iterator.evolution()?;
            let old = copy_unless(
                &iterator.old_shape()?,
                evolution == TNamingEvolution::Primitive,
                table,
            )?;
            let new = copy_unless(
                &iterator.new_shape()?,
                evolution == TNamingEvolution::Delete,
                table,
            )?;
            paste_evolution(&builder, evolution, &old, &new)?;
            iterator.next()?;
        }
        Ok(())
    }
    /// `References`: the labels that created each old shape (a non-selection evolution
    /// whose new shape it is).
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        let Some(data_set) = data_set.get() else {
            return;
        };
        let mut current = self.first_node();
        while let Some(node) = current {
            if let Some(old) = node.old() {
                add_creators(data_set, &Some(old));
            }
            current = node.next_same_attribute();
        }
    }
    /// `BeforeRemoval`: `Clear()`.
    fn before_removal(&self) -> Result<(), Failure> {
        self.clear()
    }
    /// `BeforeUndo`: nothing to do.
    fn before_undo(
        &self,
        _delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `AfterUndo`: an undone addition removes the evolutions from the shape chains.
    fn after_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        if delta.is_kind(TdfDeltaOnAddition::static_type()) {
            self.forget_evolutions(live(delta, "TNaming_NamedShape::AfterUndo")?)?;
        }
        Ok(true)
    }
    /// `Dump`: writes nothing.
    fn dump(&self, _stream: &mut dyn io::Write) -> io::Result<()> {
        Ok(())
    }
    /// `DumpJson`: the base attribute, the label entry of the first node, the evolution
    /// (as its integer) and the version.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TNaming_NamedShape")?;
        dump_attribute_json_base(self, stream, depth)?;
        if let Some(node) = self.first_node() {
            let label = node.label().map_err(io::Error::other)?;
            dump_label_entry(stream, "aLabel", &label)?;
        }
        let evolution = self.evolution() as i32;
        field_string(stream, "myEvolution", &evolution.to_string())?;
        field_integer(stream, "myVersion", self.version())
    }
}

/// The copy of `Paste`: a null shape when `skip`, else `TNaming_CopyShape::CopyTool`.
fn copy_unless(shape: &Shape, skip: bool, table: &TdfRelocationTable) -> Result<Shape, Failure> {
    let mut copy = Shape::default();
    if !skip {
        TNamingCopyShape::copy_tool(shape, &mut table.transient_table(), &mut copy)?;
    }
    Ok(copy)
}

/// The `switch` of `Paste` (`REPLACE` is not pasted).
fn paste_evolution(
    builder: &TNamingBuilder,
    evolution: TNamingEvolution,
    old: &Shape,
    new: &Shape,
) -> Result<(), Failure> {
    match evolution {
        TNamingEvolution::Primitive => builder.generated(new),
        TNamingEvolution::Generated => builder.generated_from(old, new),
        TNamingEvolution::Modify => builder.modify(old, new),
        TNamingEvolution::Delete => builder.delete(old),
        TNamingEvolution::Selected => builder.select(new, old),
        TNamingEvolution::Replace => Ok(()),
    }
}

/// The inner loop of `References`: every node of the shape's chain that made it as a new
/// shape, outside a selection, adds its label.
fn add_creators(data_set: &TdfDataSet, shape: &super::TNamingRefShapePtr) {
    let Some(reference) = shape else { return };
    let mut current = reference.first_use();
    while let Some(node) = current {
        let creates = same_pointer(&node.new_shape(), shape)
            && node
                .attribute()
                .get()
                .is_some_and(|named| named.evolution() != TNamingEvolution::Selected);
        if creates && let Ok(label) = node.label() {
            data_set.add_label(&label);
        }
        current = node.next_same_shape(shape);
    }
}
