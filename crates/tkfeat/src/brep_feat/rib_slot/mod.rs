// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_RibSlot.cxx (the
// members of the history) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_RibSlot`: the base of the ribs and slots (linear and revolution forms): the
//! profile built from a wire sketched on the basis, its extreme faces, the gluing or the
//! Boolean operation of the form, and the history of the faces.
mod extreme;
mod extreme_multi;
mod geometry;
mod no_sliding;
mod no_sliding_edges;
mod perform;
mod sliding;

pub use extreme::Extremes;
pub use geometry::{choice_of_faces, int_par};
pub use sliding::ProfileInput;

use super::{DescendantMap, PerfSelection, StatusError};
use crate::loc_ope::Gluer;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIBooleanOperation;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Wire};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase;

/// `BRepFeat_RibSlot`: the data of a rib or slot (the protected members of the native
/// class), embedded in the linear and revolution forms.
pub struct RibSlot {
    pub(crate) base: BRepBuilderAPIMakeShapeBase,
    pub(crate) first_pnt: Point3,
    pub(crate) last_pnt: Point3,
    pub(crate) fuse: bool,
    pub(crate) sliding: bool,
    pub(crate) map: DescendantMap,
    pub(crate) lf_map: DescendantMap,
    pub(crate) f_shape: Shape,
    pub(crate) l_shape: Shape,
    pub(crate) perf_selection: PerfSelection,
    pub(crate) wire: Wire,
    pub(crate) sbase: Shape,
    pub(crate) skface: Face,
    pub(crate) pbase: Face,
    pub(crate) g_shape: Shape,
    pub(crate) s_until: Shape,
    pub(crate) glued_f: DataMap<Shape, Shape, ShapeMapHasher>,
    pub(crate) new_edges: List<Shape>,
    pub(crate) tgt_edges: List<Shape>,
    pub(crate) faces_for_draft: List<Shape>,
    pub(crate) status_error: StatusError,
}

impl Default for RibSlot {
    /// `BRepFeat_RibSlot()`: neither fused nor sliding, status OK.
    fn default() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            first_pnt: Point3::default(),
            last_pnt: Point3::default(),
            fuse: false,
            sliding: false,
            map: DataMap::default(),
            lf_map: DataMap::default(),
            f_shape: Shape::default(),
            l_shape: Shape::default(),
            perf_selection: PerfSelection::NoSelection,
            wire: Wire::default(),
            sbase: Shape::default(),
            skface: Face::default(),
            pbase: Face::default(),
            g_shape: Shape::default(),
            s_until: Shape::default(),
            glued_f: DataMap::default(),
            new_edges: List::default(),
            tgt_edges: List::default(),
            faces_for_draft: List::default(),
            status_error: StatusError::Ok,
        }
    }
}

/// The shapes of `list` other than `s`.
fn others(list: &List<Shape>, s: &Shape) -> List<Shape> {
    let mut l = List::default();
    for sh in list.iter() {
        if !sh.is_same(s) {
            l.append(sh.clone());
        }
    }
    l
}

/// Whether `s` has a face the same as `f`.
fn holds_face(s: &Shape, f: &Shape) -> Result<bool, Failure> {
    Ok(Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?.any(|x| x.is_same(f)))
}

impl RibSlot {
    /// `IsDeleted(F)`: whether `f` has no descendant (Standard_NoSuchObject when unknown);
    /// `rib.rib_is_deleted(&f)?`.
    pub fn rib_is_deleted(&self, f: &Shape) -> Result<bool, Failure> {
        Ok(self.map.find(f)?.is_empty())
    }

    /// `Modified(F)`: the descendants of `f` other than itself (none when unknown);
    /// `rib.rib_modified(&f)?`.
    pub fn rib_modified(&mut self, f: &Shape) -> Result<&List<Shape>, Failure> {
        if self.map.is_bound(f) {
            self.base.my_generated = others(self.map.find(f)?, f);
            return Ok(&self.base.my_generated);
        }
        Ok(&self.base.my_generated)
    }

    /// `Generated(S)`: for an edge or vertex, the faces made of it (through the edges of
    /// the profile it became when it is in the map of the profile); none for a face;
    /// `rib.rib_generated(&e)?`.
    pub fn rib_generated(&mut self, s: &Shape) -> Result<&List<Shape>, Failure> {
        if s.shape_type()? == ShapeEnum::Face {
            return Ok(&self.base.my_generated);
        }
        self.base.my_generated.clear();
        if self.lf_map.is_empty() || !self.lf_map.is_bound(s) {
            if self.map.is_bound(s) {
                // check if filter on face or not
                self.base.my_generated = others(self.map.find(s)?, s);
            }
            return Ok(&self.base.my_generated);
        }
        let mut list = List::default();
        for e in self.lf_map.find(s)?.iter() {
            if self.map.is_bound(e) {
                list.append_list(&mut others(self.map.find(e)?, s));
            }
        }
        self.base.my_generated = list;
        Ok(&self.base.my_generated)
    }

    /// `UpdateDescendants(G)`: the history through the faces the gluer made of them.
    pub(crate) fn update_descendants_glue(&mut self, g: &Gluer) -> Result<(), Failure> {
        let keys: Vec<Shape> = self.map.items().map(|(k, _)| k.clone()).collect();
        for orig in keys {
            let mut newdsc = MapOfShape::default();
            for fdsc in self.map.find(&orig)?.iter() {
                for d in g.descendant_faces(&topods::face(fdsc)?)?.iter() {
                    newdsc.add(d.clone());
                }
            }
            let list = self.map.change_find(&orig)?;
            list.clear();
            for d in newdsc.iter() {
                list.append(d.clone());
            }
        }
        Ok(())
    }

    /// `UpdateDescendants(aBOP, S, SkipFace)`: the history through the faces kept in the
    /// result `s` or modified by the Boolean operation (the faces themselves skipped when
    /// `skip_face`); `rib.update_descendants_bop(&cut, &res, false)?`.
    pub(crate) fn update_descendants_bop(
        &mut self,
        a_bop: &BRepAlgoAPIBooleanOperation,
        s: &Shape,
        skip_face: bool,
    ) -> Result<(), Failure> {
        let keys: Vec<Shape> = self.map.items().map(|(k, _)| k.clone()).collect();
        for orig in keys {
            if skip_face && orig.shape_type()? == ShapeEnum::Face {
                continue;
            }
            let mut newdsc = MapOfShape::default();
            for sh in self.map.find(&orig)?.iter() {
                if sh.shape_type()? != ShapeEnum::Face {
                    continue;
                }
                if holds_face(s, sh)? {
                    // preserved
                    newdsc.add(sh.clone());
                    continue;
                }
                for m in a_bop.modified(sh)? {
                    newdsc.add(m);
                }
            }
            let mut kept = List::default();
            for d in newdsc.iter() {
                // check the belonging to the shape...
                if holds_face(s, d)? {
                    kept.append(d.clone());
                }
            }
            *self.map.change_find(&orig)? = kept;
        }
        Ok(())
    }

    /// `FirstShape()`: the faces at the start of the form (none without);
    /// `rib.first_shape()?`.
    pub fn first_shape(&self) -> Result<List<Shape>, Failure> {
        if self.f_shape.is_null() {
            return Ok(self.base.my_generated.clone());
        }
        self.map.find(&self.f_shape).cloned()
    }

    /// `LastShape()`: the faces at the end of the form (none without);
    /// `rib.last_shape()?`.
    pub fn last_shape(&self) -> Result<List<Shape>, Failure> {
        if self.l_shape.is_null() {
            return Ok(self.base.my_generated.clone());
        }
        self.map.find(&self.l_shape).cloned()
    }

    /// `FacesForDraft()`: the faces of the form to draft; `rib.faces_for_draft()`.
    pub fn faces_for_draft(&self) -> &List<Shape> {
        &self.faces_for_draft
    }

    /// `NewEdges()`: the edges created by the gluing; `rib.new_edges()`.
    pub fn new_edges(&self) -> &List<Shape> {
        &self.new_edges
    }

    /// `TgtEdges()`: the tangent edges created by the gluing; `rib.tgt_edges()`.
    pub fn tgt_edges(&self) -> &List<Shape> {
        &self.tgt_edges
    }

    /// `CurrentStatusError()`: why the form failed; `rib.current_status_error()`.
    pub fn current_status_error(&self) -> StatusError {
        self.status_error
    }
}

/// Implements `BRepBuilderAPIMakeShape` for a form with the `BRepFeat_RibSlot` history;
/// the form holds its data in a field named `rib`.
macro_rules! impl_rib_make_shape {
    ($feature:ty) => {
        impl rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape for $feature {
            fn make_shape_base(
                &self,
            ) -> &rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase {
                &self.rib.base
            }
            fn make_shape_base_mut(
                &mut self,
            ) -> &mut rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase {
                &mut self.rib.base
            }
            fn modified(
                &mut self,
                s: &rustycad_tkbrep::topods::Shape,
            ) -> Result<
                &rustycad_tkernel::ncollection::List<rustycad_tkbrep::topods::Shape>,
                rustycad_tkernel::standard::Failure,
            > {
                self.rib.rib_modified(s)
            }
            fn generated(
                &mut self,
                s: &rustycad_tkbrep::topods::Shape,
            ) -> Result<
                &rustycad_tkernel::ncollection::List<rustycad_tkbrep::topods::Shape>,
                rustycad_tkernel::standard::Failure,
            > {
                self.rib.rib_generated(s)
            }
            fn is_deleted(
                &mut self,
                s: &rustycad_tkbrep::topods::Shape,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.rib.rib_is_deleted(s)
            }
        }
    };
}
pub(crate) use impl_rib_make_shape;
