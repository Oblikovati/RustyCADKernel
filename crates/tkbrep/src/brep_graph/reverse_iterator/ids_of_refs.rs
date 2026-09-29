// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of BRepGraph_ReverseIterator::IdsOfRefs and the *FromRefTraits
// (ModelingData/TKBRep/BRepGraph/BRepGraph_ReverseIterator.hxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The node each reference of a reverse relation list names.
use crate::brep_graph::iterator::NodeTraits;
use crate::brep_graph::refs_iterator::RefTraits;
use crate::brep_graph::{
    BRepGraph, ChildRefTag, CompSolidTag, CompoundTag, FaceRefTag, FaceTag, OccurrenceRefTag,
    OccurrenceTag, ProductTag, ShellRefTag, ShellTag, SolidRefTag, SolidTag, TypedId, WireRefTag,
};
use crate::brep_graph_inc::{NodeTag, RefTag};

/// The traits of [`IdsOfRefs`]: the node a reference names.
pub trait IdsOfRefsTraits {
    /// `IdType`: the node kind yielded.
    type Id: NodeTag;
    /// `RefIdType`: the reference kind of the list.
    type Ref: RefTag;
    /// `Id(theGraph, theRefId)`: the node of a reference in range.
    fn id(graph: &BRepGraph, reference: TypedId<Self::Ref>) -> TypedId<Self::Id>;
}

macro_rules! ids_of_refs_traits {
    ($($(#[$doc:meta])* $name:ident: $ref:ty => $id:ty, $field:ident;)*) => {$(
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name;

        impl IdsOfRefsTraits for $name {
            type Id = $id;
            type Ref = $ref;
            fn id(graph: &BRepGraph, reference: TypedId<$ref>) -> TypedId<$id> {
                <$ref>::get(graph, reference).$field
            }
        }
    )*};
}

ids_of_refs_traits! {
    /// `FaceFromWireRefTraits`: the parent face of a wire reference.
    FaceFromWireRefTraits: WireRefTag => FaceTag, parent_face_id;
    /// `ShellFromFaceRefTraits`: the parent shell of a face reference.
    ShellFromFaceRefTraits: FaceRefTag => ShellTag, parent_shell_id;
    /// `SolidFromShellRefTraits`: the parent solid of a shell reference.
    SolidFromShellRefTraits: ShellRefTag => SolidTag, parent_solid_id;
    /// `CompSolidFromSolidRefTraits`: the parent compsolid of a solid reference.
    CompSolidFromSolidRefTraits: SolidRefTag => CompSolidTag, parent_comp_solid_id;
    /// `CompoundFromChildRefTraits`: the parent compound of a child reference.
    CompoundFromChildRefTraits: ChildRefTag => CompoundTag, parent_compound_id;
    /// `OccurrenceFromOccurrenceRefTraits`: the occurrence an occurrence reference names.
    OccurrenceFromOccurrenceRefTraits: OccurrenceRefTag => OccurrenceTag, child_occurrence_id;
    /// `ProductFromOccurrenceRefTraits`: the parent product of an occurrence reference.
    ProductFromOccurrenceRefTraits: OccurrenceRefTag => ProductTag, parent_product_id;
}

/// `IdsOfRefs<TraitsT>`: the active nodes named by the active references of a list,
/// with the reference that names each.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, WireRefId};
/// use rustycad_tkbrep::brep_graph::reverse_iterator::FacesOfWire;
/// let refs: Vec<WireRefId> = Vec::new();
/// assert!(!FacesOfWire::new(&BRepGraph::new(), &refs).more());
/// ```
pub struct IdsOfRefs<'g, I: IdsOfRefsTraits> {
    graph: &'g BRepGraph,
    refs: &'g [TypedId<I::Ref>],
    current: TypedId<I::Id>,
    current_ref: TypedId<I::Ref>,
    nb_refs: u32,
    nb_ids: u32,
    index: u32,
    has_current: bool,
}

impl<'g, I: IdsOfRefsTraits> IdsOfRefs<'g, I> {
    /// `IdsOfRefs(theGraph, theRefs)`.
    pub fn new(graph: &'g BRepGraph, refs: &'g [TypedId<I::Ref>]) -> Self {
        Self::starting_at(graph, refs, 0)
    }

    /// `IdsOfRefs(theGraph, theRefs, theStartIndex)`.
    pub fn starting_at(
        graph: &'g BRepGraph,
        refs: &'g [TypedId<I::Ref>],
        start_index: u32,
    ) -> Self {
        let mut iterator = Self {
            graph,
            refs,
            current: TypedId::default(),
            current_ref: TypedId::default(),
            nb_refs: I::Ref::count(graph),
            nb_ids: I::Id::count(graph),
            index: start_index,
            has_current: false,
        };
        iterator.advance();
        iterator
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.has_current
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.index += 1;
        self.advance();
    }

    /// `CurrentId()`: the current node (the previous one after the end).
    pub fn current_id(&self) -> TypedId<I::Id> {
        self.current
    }

    /// `CurrentParentId()`: the same as [`current_id`](Self::current_id).
    pub fn current_parent_id(&self) -> TypedId<I::Id> {
        self.current
    }

    /// `CurrentRefId()`: the reference naming the current node.
    pub fn current_ref_id(&self) -> TypedId<I::Ref> {
        self.current_ref
    }

    /// `Current()`: the same as [`current_id`](Self::current_id).
    pub fn current(&self) -> TypedId<I::Id> {
        self.current
    }

    /// `Definition()`: the current node's definition.
    pub fn definition(&self) -> &'g <I::Id as NodeTag>::Def {
        I::Id::get(self.graph, self.current)
    }

    /// `Index()`: the position in the reference list.
    pub fn index(&self) -> u32 {
        self.index
    }

    fn advance(&mut self) {
        self.has_current = false;
        while let Some(reference) = self.refs.get(self.index as usize).copied() {
            if reference.is_valid_for(self.nb_refs) && !reference.is_removed(self.graph) {
                let id = I::id(self.graph, reference);
                if id.is_valid_for(self.nb_ids) && !id.is_removed(self.graph) {
                    self.current = id;
                    self.current_ref = reference;
                    self.has_current = true;
                    return;
                }
            }
            self.index += 1;
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` id.
impl<I: IdsOfRefsTraits> Iterator for IdsOfRefs<'_, I> {
    type Item = TypedId<I::Id>;
    fn next(&mut self) -> Option<TypedId<I::Id>> {
        if !self.more() {
            return None;
        }
        let current = self.current;
        IdsOfRefs::next(self);
        Some(current)
    }
}
