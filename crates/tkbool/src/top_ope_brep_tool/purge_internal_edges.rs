// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_PurgeInternalEdges.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_PurgeInternalEdges`: the INTERNAL and EXTERNAL edges of the faces of
//! a shape (edges no face holds as a boundary) and the shape rebuilt without them.
use crate::top_ope_brep_tool::tool::ShapeListMap;
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The purge of the internal edges of a shape;
/// `let mut p = TopOpeBRepToolPurgeInternalEdges::new(&s, true)?;`.
pub struct TopOpeBRepToolPurgeInternalEdges {
    my_shape: Shape,
    /// `myMapEdgLstFac`: the faces of each edge.
    pub my_map_edg_lst_fac: IndexedDataMapOfShapeListOfShape,
    my_map_fac_lst_edg: ShapeListMap,
    my_is_done: bool,
}

/// Standard_NullObject of a null shape.
fn null_shape(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        member,
        "TopOpeBRepTool_PurgeInternalEdges",
    )
}

/// Whether an orientation is INTERNAL or EXTERNAL.
fn non_oriented(o: Orientation) -> bool {
    matches!(o, Orientation::Internal | Orientation::External)
}

impl TopOpeBRepToolPurgeInternalEdges {
    /// `TopOpeBRepTool_PurgeInternalEdges(theShape, PerformNow)` (PerformNow natively
    /// defaulted to true): Standard_NullObject for a null shape;
    /// `TopOpeBRepToolPurgeInternalEdges::new(&s, true)?`.
    pub fn new(the_shape: &Shape, perform_now: bool) -> Result<Self, Failure> {
        if the_shape.is_null() {
            return Err(null_shape("PurgeInternalEdges"));
        }
        let mut p = Self {
            my_shape: the_shape.clone(),
            my_map_edg_lst_fac: IndexedDataMapOfShapeListOfShape::default(),
            my_map_fac_lst_edg: ShapeListMap::default(),
            my_is_done: false,
        };
        if perform_now {
            p.perform()?;
        }
        Ok(p)
    }

    /// `Faces(theMapFacLstEdg)`: the internal edges of each face (computed first when
    /// needed); `p.faces()?`.
    pub fn faces(&mut self) -> Result<ShapeListMap, Failure> {
        if !self.my_is_done {
            self.build_list()?;
        }
        Ok(self.my_map_fac_lst_edg.clone())
    }

    /// `NbEdges()`: the count of the internal edges recorded per face; `p.nb_edges()?`.
    pub fn nb_edges(&self) -> Result<i32, Failure> {
        if self.my_shape.is_null() {
            return Err(null_shape("PurgeInternalEdges : No Shape"));
        }
        Ok(self
            .my_map_fac_lst_edg
            .items()
            .map(|(_, l)| l.extent())
            .sum())
    }

    /// `Shape()`: the shape (purged after `Perform`); `p.shape()?`.
    pub fn shape(&mut self) -> Result<&mut Shape, Failure> {
        if self.my_shape.is_null() {
            return Err(null_shape("PurgeInternalEdges : No Shape"));
        }
        Ok(&mut self.my_shape)
    }

    /// `IsDone()`; `p.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// The faces to treat for an edge: those holding it INTERNAL or EXTERNAL before a
    /// face holding it as a boundary (none then).
    fn faces_to_treat(
        &self,
        edgecur: &Shape,
        lmap_fac: &List<Shape>,
    ) -> Result<List<Shape>, Failure> {
        let mut lst: List<Shape> = List::default();
        if lmap_fac.extent() <= 1 {
            if non_oriented(edgecur.orientation())
                && let Some(f) = lmap_fac.first()
            {
                lst.append(f.clone());
            }
            return Ok(lst);
        }
        for facecur in lmap_fac.iter() {
            let found = Explorer::new(facecur, ShapeEnum::Edge, ShapeEnum::Shape)?
                .find(|e| e.is_same(edgecur));
            let Some(edge) = found else {
                continue;
            };
            if !non_oriented(edge.orientation()) {
                lst.clear();
                break;
            }
            lst.append(facecur.clone());
        }
        Ok(lst)
    }

    /// `BuildList()` (protected): the internal edges of each face.
    fn build_list(&mut self) -> Result<(), Failure> {
        self.my_map_edg_lst_fac.clear(false);
        topexp::map_shapes_and_ancestors(
            &self.my_shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut self.my_map_edg_lst_fac,
        )?;
        for i_edg in 1..=self.my_map_edg_lst_fac.extent() {
            let edgecur = self.my_map_edg_lst_fac.find_key(i_edg)?.clone();
            let lmap_fac = self.my_map_edg_lst_fac.find_from_index(i_edg)?.clone();
            let lst = self.faces_to_treat(&edgecur, &lmap_fac)?;
            let mut map_uniq_edg: Map<Shape, ShapeMapHasher> = Map::default();
            for face in lst.iter() {
                if !map_uniq_edg.add(edgecur.clone()) {
                    continue;
                }
                match self.my_map_fac_lst_edg.change_seek(face) {
                    Some(l) => {
                        l.append(edgecur.clone());
                    }
                    None => {
                        let mut l: List<Shape> = List::default();
                        l.append(edgecur.clone());
                        self.my_map_fac_lst_edg.bind(face.clone(), l);
                    }
                }
            }
        }
        self.my_is_done = true;
        Ok(())
    }

    /// `Perform()`: the internal edges removed from the shape (BRepTools_Substitution);
    /// `p.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if !self.my_is_done {
            self.build_list()?;
        }
        if self.my_map_fac_lst_edg.extent() == 0 {
            return Ok(());
        }
        let mut bsub = Substitution::new();
        for (_, lmap_edg) in self.my_map_fac_lst_edg.items() {
            for e in lmap_edg.iter() {
                bsub.substitute(e, &[])?;
            }
        }
        bsub.build(&self.my_shape)?;
        if bsub.is_copied(&self.my_shape)
            && let Some(first) = bsub.copy(&self.my_shape)?.first()
        {
            self.my_shape = first.clone();
        }
        Ok(())
    }
}
