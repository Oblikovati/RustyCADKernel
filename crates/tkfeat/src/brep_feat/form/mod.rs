// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_Form.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_Form`: the general construction of the form features (prisms, drafted
//! prisms, revolutions, pipes): the generated tool limited by the shapes From and Until,
//! glued on the basis when possible, else combined with it by BRepFeat_Builder keeping
//! the parts of the tool selected along the feature direction; the history of the faces.
mod global_perform;

use super::{BRepFeatBuilder, PerfSelection, StatusError};
use crate::loc_ope::Gluer;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIBooleanOperation;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomPlane, GeomRectangularTrimmedSurface,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};

pub(crate) use global_perform::global_perform;

/// The shapes and their descendants in the result (`myMap`).
pub type DescendantMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The protected and private data of `BRepFeat_Form`, embedded in each form feature.
#[derive(Clone)]
pub struct Form {
    /// The `BRepBuilderAPI_MakeShape` base.
    pub(crate) base: BRepBuilderAPIMakeShapeBase,
    pub(crate) fuse: bool,
    pub(crate) modify: bool,
    pub(crate) map: DescendantMap,
    pub(crate) f_shape: Shape,
    pub(crate) l_shape: Shape,
    pub(crate) new_edges: List<Shape>,
    pub(crate) tgt_edges: List<Shape>,
    pub(crate) perf_selection: PerfSelection,
    pub(crate) just_gluer: bool,
    pub(crate) just_feat: bool,
    pub(crate) sbase: Shape,
    pub(crate) skface: Face,
    pub(crate) g_shape: Shape,
    pub(crate) s_from: Shape,
    pub(crate) s_until: Shape,
    pub(crate) glued_f: DataMap<Shape, Shape, ShapeMapHasher>,
    sb_ok: bool,
    sk_ok: bool,
    gs_ok: bool,
    sf_ok: bool,
    su_ok: bool,
    gf_ok: bool,
    ps_ok: bool,
    pub(crate) status_error: StatusError,
}

impl Default for Form {
    /// `BRepFeat_Form()`: nothing valid yet, not initialized.
    fn default() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            fuse: false,
            modify: false,
            map: DataMap::default(),
            f_shape: Shape::default(),
            l_shape: Shape::default(),
            new_edges: List::default(),
            tgt_edges: List::default(),
            perf_selection: PerfSelection::NoSelection,
            just_gluer: false,
            just_feat: false,
            sbase: Shape::default(),
            skface: Face::default(),
            g_shape: Shape::default(),
            s_from: Shape::default(),
            s_until: Shape::default(),
            glued_f: DataMap::default(),
            sb_ok: false,
            sk_ok: false,
            gs_ok: false,
            sf_ok: false,
            su_ok: false,
            gf_ok: false,
            ps_ok: false,
            status_error: StatusError::NotInitialized,
        }
    }
}

/// `BRepFeat_Form`: a form feature, with the pure virtual members of the native class.
pub trait FormFeature: BRepBuilderAPIMakeShape {
    /// The embedded `BRepFeat_Form` data.
    fn form(&self) -> &Form;

    /// The embedded `BRepFeat_Form` data, to write.
    fn form_mut(&mut self) -> &mut Form;

    /// `Curves(S)`: the curves along which the tool is intersected with the shapes From
    /// and Until.
    fn curves(&mut self, s: &mut Sequence<Handle<dyn GeomCurve>>) -> Result<(), Failure>;

    /// `BarycCurve()`: the curve through the barycentre along which the parts of the tool
    /// are selected.
    fn baryc_curve(&mut self) -> Result<Handle<dyn GeomCurve>, Failure>;
}

/// Implements `BRepBuilderAPIMakeShape` for a form feature with the `BRepFeat_Form`
/// overrides of `Modified`, `Generated` and `IsDeleted`.
macro_rules! impl_form_make_shape {
    ($feature:ty) => {
        impl rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape for $feature {
            fn make_shape_base(
                &self,
            ) -> &rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase {
                &self.form.base
            }
            fn make_shape_base_mut(
                &mut self,
            ) -> &mut rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase {
                &mut self.form.base
            }
            fn modified(
                &mut self,
                s: &rustycad_tkbrep::topods::Shape,
            ) -> Result<
                &rustycad_tkernel::ncollection::List<rustycad_tkbrep::topods::Shape>,
                rustycad_tkernel::standard::Failure,
            > {
                self.form.form_modified(s)
            }
            fn generated(
                &mut self,
                s: &rustycad_tkbrep::topods::Shape,
            ) -> Result<
                &rustycad_tkernel::ncollection::List<rustycad_tkbrep::topods::Shape>,
                rustycad_tkernel::standard::Failure,
            > {
                self.form.form_generated(s)
            }
            fn is_deleted(
                &mut self,
                s: &rustycad_tkbrep::topods::Shape,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                Ok(self.form.form_is_deleted(s))
            }
        }
    };
}
pub(crate) use impl_form_make_shape;

/// `Descendants(S, theFB, mapF)`: the faces the faces of `s` became in the builder.
pub(crate) fn descendants(
    s: &Shape,
    the_fb: &BRepFeatBuilder,
    map_f: &mut MapOfShape,
) -> Result<(), Failure> {
    map_f.clear(false);
    for f in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        for m in the_fb.modified(&f)? {
            map_f.add(m);
        }
    }
    Ok(())
}

/// Whether `s` is a face of `shape`.
fn holds_face(shape: &Shape, s: &Shape) -> Result<bool, Failure> {
    Ok(Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?.any(|x| x.is_same(s)))
}

impl Form {
    /// `IsDone()` of the command; `form.is_done()`.
    pub fn is_done(&self) -> bool {
        self.base.is_done()
    }

    /// `BasisShapeValid()`; `form.basis_shape_valid()`.
    pub fn basis_shape_valid(&mut self) {
        self.sb_ok = true;
    }

    /// `GeneratedShapeValid()`; `form.generated_shape_valid()`.
    pub fn generated_shape_valid(&mut self) {
        self.gs_ok = true;
    }

    /// `ShapeFromValid()`; `form.shape_from_valid()`.
    pub fn shape_from_valid(&mut self) {
        self.sf_ok = true;
    }

    /// `ShapeUntilValid()`; `form.shape_until_valid()`.
    pub fn shape_until_valid(&mut self) {
        self.su_ok = true;
    }

    /// `GluedFacesValid()`; `form.glued_faces_valid()`.
    pub fn glued_faces_valid(&mut self) {
        self.gf_ok = true;
    }

    /// `SketchFaceValid()`; `form.sketch_face_valid()`.
    pub fn sketch_face_valid(&mut self) {
        self.sk_ok = true;
    }

    /// `PerfSelectionValid()`; `form.perf_selection_valid()`.
    pub fn perf_selection_valid(&mut self) {
        self.ps_ok = true;
    }

    /// Whether every part of the construction is valid.
    pub(crate) fn all_valid(&self) -> bool {
        self.sb_ok
            && self.gs_ok
            && self.sf_ok
            && self.su_ok
            && self.gf_ok
            && self.sk_ok
            && self.ps_ok
    }

    /// `CurrentStatusError()`; `form.current_status_error()`.
    pub fn current_status_error(&self) -> StatusError {
        self.status_error
    }

    /// `IsDeleted(F)`: `f` has an empty list of descendants; `form.form_is_deleted(&f)`.
    pub fn form_is_deleted(&self, f: &Shape) -> bool {
        self.map.seek(f).is_some_and(|l| l.is_empty())
    }

    /// `Modified(F)`: the result for the basis, else the descendants of `f` of its type
    /// other than itself (none before the feature is done); `form.form_modified(&f)?`.
    pub fn form_modified(&mut self, f: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated.clear();
        if !self.base.is_done() {
            return Ok(&self.base.my_generated);
        }
        if self.sbase.is_equal(f) {
            let shape = self.base.my_shape.clone();
            self.base.my_generated.append(shape);
            return Ok(&self.base.my_generated);
        }
        if let Some(l) = self.map.seek(f) {
            let kind = f.shape_type()?;
            for sh in l.iter() {
                if !sh.is_same(f) && sh.shape_type()? == kind {
                    self.base.my_generated.append(sh.clone());
                }
            }
        }
        Ok(&self.base.my_generated)
    }

    /// `Generated(S)`: the descendants of a shape other than a face (none for a face);
    /// `form.form_generated(&s)?`.
    pub fn form_generated(&mut self, s: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated.clear();
        if !self.base.is_done() {
            return Ok(&self.base.my_generated);
        }
        if let Some(l) = self.map.seek(s) {
            // check if filter on face or not
            if s.shape_type()? != ShapeEnum::Face {
                for sh in l.iter() {
                    if !sh.is_same(s) {
                        self.base.my_generated.append(sh.clone());
                    }
                }
            }
        }
        Ok(&self.base.my_generated)
    }

    /// `FirstShape()`: the faces created at the bottom of the form (none without);
    /// `form.first_shape()?`.
    pub fn first_shape(&self) -> Result<List<Shape>, Failure> {
        self.end_shape(&self.f_shape)
    }

    /// `LastShape()`: the faces created at the top of the form; `form.last_shape()?`.
    pub fn last_shape(&self) -> Result<List<Shape>, Failure> {
        self.end_shape(&self.l_shape)
    }

    fn end_shape(&self, s: &Shape) -> Result<List<Shape>, Failure> {
        if s.is_null() {
            return Ok(self.base.my_generated.clone());
        }
        self.map.find(s).cloned()
    }

    /// `NewEdges()`: the limiting and gluing edges created by the feature;
    /// `form.new_edges()`.
    pub fn new_edges(&self) -> &List<Shape> {
        &self.new_edges
    }

    /// `TgtEdges()`: those of them between tangent faces; `form.tgt_edges()`.
    pub fn tgt_edges(&self) -> &List<Shape> {
        &self.tgt_edges
    }

    /// `UpdateDescendants(G)`: the descendants become the faces the gluer made of them;
    /// `form.update_descendants_glue(&gluer)?`.
    pub fn update_descendants_glue(&mut self, g: &Gluer) -> Result<(), Failure> {
        let keys: Vec<Shape> = self.map.items().map(|(k, _)| k.clone()).collect();
        for orig in keys {
            let mut newdsc = MapOfShape::default();
            for f in self.map.find(&orig)?.iter() {
                for d in g.descendant_faces(&topods::face(f)?)?.iter() {
                    newdsc.add(d.clone());
                }
            }
            let l = self.map.change_find(&orig)?;
            l.clear();
            for d in newdsc.iter() {
                l.append(d.clone());
            }
        }
        Ok(())
    }

    /// `UpdateDescendants(aBOP, SResult, SkipFace)`: the face descendants kept in
    /// `s_result`, or what the Boolean operation modified them into, restricted to the
    /// faces of `s_result` (the faces themselves skipped when `skip_face`); an empty
    /// list gets its own shape first; `form.update_descendants_bop(&cut, &res, true)?`.
    pub fn update_descendants_bop(
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
            if self.map.find(&orig)?.is_empty() {
                self.map.change_find(&orig)?.append(orig.clone());
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
                // check the appartenance to the shape...
                if holds_face(s, d)? {
                    kept.append(d.clone());
                }
            }
            *self.map.change_find(&orig)? = kept;
        }
        Ok(())
    }

    /// Whether the single face of a From or Until shape is an infinite plane, cylinder
    /// or cone (no wire, or a natural restriction).
    fn is_infinite_face(fac: &Face) -> Result<bool, Failure> {
        let mut s = BRepTool::surface(fac)?;
        let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
        if let Some(t) = trimmed.get() {
            s = t.basis_surface();
        }
        let elementary = !s.cast::<GeomPlane>().is_null()
            || !s.cast::<GeomCylindricalSurface>().is_null()
            || !s.cast::<GeomConicalSurface>().is_null();
        if !elementary {
            return Ok(false);
        }
        if !Explorer::new(fac, ShapeEnum::Wire, ShapeEnum::Shape)?.more() {
            return Ok(true);
        }
        BRepTool::natural_restriction(fac)
    }

    /// `TransformShapeFU(flag)`: the From (0) or Until (1) shape made of one infinite
    /// face is replaced by that face extended beyond the basis (and mapped to it); the
    /// faces of a shape of several faces are mapped to themselves. True when extended;
    /// `form.transform_shape_fu(1)?`.
    pub fn transform_shape_fu(&mut self, flag: i32) -> Result<bool, Failure> {
        let shapefu = match flag {
            0 => self.s_from.clone(),
            1 => self.s_until.clone(),
            _ => return Ok(false),
        };
        let faces: Vec<Shape> =
            Explorer::new(&shapefu, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
        if faces.is_empty() {
            // no faces... It is necessary to return an error
            return Ok(false);
        }
        if faces.len() > 1 {
            for fac in faces {
                let mut thelist = List::default();
                thelist.append(fac.clone());
                self.map.bind(fac, thelist);
            }
            return Ok(false);
        }
        // the only face. Is it infinite?
        let mut fac = topods::face(&faces[0])?;
        let trf = Self::is_infinite_face(&fac)?;
        if trf {
            super::face_until(&self.sbase, &mut fac)?;
        }
        let mut thelist = List::default();
        thelist.append(fac.as_ref().clone());
        self.map.bind(shapefu, thelist);
        if flag == 0 {
            self.s_from = fac.into_shape();
        } else {
            self.s_until = fac.into_shape();
        }
        Ok(trf)
    }
}
