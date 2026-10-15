// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevol.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_MakeRevol`: a revolved feature of a planar profile around an axis, of a
//! given angle, up to a shape, between two shapes or through all (a full turn).
mod limited;
mod perform;

use super::common::{check_basis_face, maj_map_swept};
use super::form::impl_form_make_shape;
use super::{Form, FormFeature, StatusError, tool};
use crate::errors::construction_error;
use crate::loc_ope::{CSIntersector, Revol};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Axis1;

/// `BRepFeat_MakeRevol`: the revolution of the profile `pbase` sketched on `skface`
/// around `axis`, fused or cut.
pub struct BRepFeatMakeRevol {
    pub(crate) form: Form,
    pub(crate) axis: Axis1,
    pub(crate) pbase: Shape,
    /// The sliding faces of the basis with the profile edges lying in them.
    pub(crate) slface: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(crate) curves: Sequence<Handle<dyn GeomCurve>>,
    pub(crate) b_curve: Handle<dyn GeomCurve>,
}

impl_form_make_shape!(BRepFeatMakeRevol);

impl FormFeature for BRepFeatMakeRevol {
    fn form(&self) -> &Form {
        &self.form
    }

    fn form_mut(&mut self) -> &mut Form {
        &mut self.form
    }

    /// `Curves(scur)`: the circles of the revolution.
    fn curves(&mut self, s: &mut Sequence<Handle<dyn GeomCurve>>) -> Result<(), Failure> {
        *s = self.curves.clone();
        Ok(())
    }

    /// `BarycCurve()`: the circle of the revolution through the barycentre of the profile.
    fn baryc_curve(&mut self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(self.b_curve.clone())
    }
}

impl Default for BRepFeatMakeRevol {
    fn default() -> Self {
        Self::new()
    }
}

/// `VerifGluedFaces`: whether every solid of the revolution cut by the solid of the
/// sketch face keeps a face of the base (the solid limited where the circle of the
/// barycentre meets the sketch face); true without intersection.
fn glued_faces_hold(
    the_skface: &Face,
    the_pbase: &Shape,
    the_b_curve: &Handle<dyn GeomCurve>,
    vrai: &Shape,
) -> Result<bool, Failure> {
    let mut scur = Sequence::default();
    scur.append(the_b_curve.clone());
    let mut asi = CSIntersector::with_shape(the_skface);
    asi.perform_curves(&scur)?;
    if !asi.is_done() || asi.nb_points(1)? < 1 {
        return Ok(true);
    }
    let p = asi.point(1, 1)?;
    let comp = tool_compound(the_skface, p.face(), p.orientation())?;
    let tr_p = BRepAlgoAPICut::from_shapes(vrai, &comp)?;
    for sol in Explorer::new(tr_p.shape(), ShapeEnum::Solid, ShapeEnum::Shape)? {
        let mut has_base = false;
        for fac1 in Explorer::new(&sol, ShapeEnum::Face, ShapeEnum::Shape)? {
            if Explorer::new(the_pbase, ShapeEnum::Face, ShapeEnum::Shape)?
                .any(|fac2| fac1.is_same(&fac2))
            {
                has_base = true;
                break;
            }
        }
        if !has_base {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The compound of the solid limited by the face `fac` of `sref` (empty when null).
fn tool_compound(sref: &Shape, fac: &Face, or: Orientation) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut comp = Compound::default();
    b.make_compound(&mut comp);
    let mut comp = comp.into_shape();
    let s = tool(sref, fac, or)?;
    if !s.is_null() {
        b.add(&mut comp, &s)?;
    }
    Ok(comp)
}

impl BRepFeatMakeRevol {
    /// `BRepFeat_MakeRevol()`: not initialized; `BRepFeatMakeRevol::new()`.
    pub fn new() -> Self {
        Self {
            form: Form::default(),
            axis: Axis1::default(),
            pbase: Shape::default(),
            slface: DataMap::default(),
            curves: Sequence::default(),
            b_curve: Handle::default(),
        }
    }

    /// `BRepFeat_MakeRevol(Sbase, Pbase, Skface, Axis, Fuse, Modify)`: initialized by
    /// `init`; `BRepFeatMakeRevol::with_shapes(&sbase, &pbase, &skface, axis, 1, true)?`.
    pub fn with_shapes(
        sbase: &Shape,
        pbase: &Shape,
        skface: &Face,
        axis: Axis1,
        mode: i32,
        modify: bool,
    ) -> Result<Self, Failure> {
        let mut revol = Self::new();
        revol.init(sbase, pbase, skface, axis, mode, modify)?;
        Ok(revol)
    }

    /// `Init(Sbase, Pbase, Skface, Axis, Mode, Modify)`: the basis, the profile sketched
    /// on `skface`, the axis and the mode (0 cut, 1 fuse, 2 the revolution alone); the
    /// history of the basis faces is reset (the new edges kept, as natively);
    /// `revol.init(&sbase, &pbase, &skface, axis, 0, true)?`.
    pub fn init(
        &mut self,
        sbase: &Shape,
        pbase: &Shape,
        skface: &Face,
        axis: Axis1,
        mode: i32,
        modify: bool,
    ) -> Result<(), Failure> {
        self.axis = axis;
        self.pbase = pbase.clone();
        self.form.sbase = sbase.clone();
        self.form.basis_shape_valid();
        self.form.skface = skface.clone();
        self.form.sketch_face_valid();
        self.slface.clear(false);
        self.form.set_mode(mode);
        self.form.modify = modify;
        self.form.just_gluer = false;
        self.form.reset_map()
    }

    /// `Add(E, F)`: the edge `e` of the profile slides on the face `f` of the basis
    /// (Standard_ConstructionError when either is not one); `revol.add(&e, &f)?`.
    pub fn add(&mut self, e: &Edge, f: &Face) -> Result<(), Failure> {
        check_basis_face(&self.form.sbase, f)?;
        if !Explorer::new(&self.pbase, ShapeEnum::Edge, ShapeEnum::Shape)?.any(|x| x.is_same(e)) {
            return Err(construction_error(
                "BRepFeat_MakeRevol::Add",
                "the edge is not an edge of the profile",
            ));
        }
        let edges = self.slface.try_bound((**f).clone(), List::default());
        if !edges.iter().any(|x| x.is_same(e)) {
            edges.append((**e).clone());
        }
        Ok(())
    }

    /// The history of the revolution mapped (`MajMap`); its shape returned.
    fn map_revol(&mut self, the_revol: &Revol) -> Result<Shape, Failure> {
        let f = &mut self.form;
        let ends = (the_revol.first_shape(), the_revol.last_shape());
        maj_map_swept(
            &self.pbase,
            ends,
            |e| Ok(the_revol.shapes(e)?.clone()),
            &mut f.map,
            &mut f.f_shape,
            &mut f.l_shape,
        )?;
        Ok(the_revol.shape()?.clone())
    }

    /// The revolution made the generated shape of the feature.
    fn set_generated(&mut self, vrai: &Shape) {
        self.form.g_shape = vrai.clone();
        self.form.generated_shape_valid();
    }

    /// The base face of the revolution (InvFirstShape, None, when its first shape has
    /// more than one face).
    fn first_face(&mut self, the_revol: &Revol) -> Result<Option<Face>, Failure> {
        let mut exp = Explorer::new(the_revol.first_shape(), ShapeEnum::Face, ShapeEnum::Shape)?;
        let the_base = topods::face(exp.current())?;
        exp.next()?;
        if exp.more() {
            self.form.fail(StatusError::InvFirstShape);
            return Ok(None);
        }
        Ok(Some(the_base))
    }

    /// The curves of the revolution kept.
    fn keep_curves(&mut self, the_revol: &Revol) -> Result<(), Failure> {
        the_revol.curves(&mut self.curves)?;
        self.b_curve = the_revol.baryc_curve()?;
        Ok(())
    }

    /// The curves of the revolution kept, then the topological reconstruction.
    fn global_with(&mut self, the_revol: &Revol) -> Result<(), Failure> {
        self.keep_curves(the_revol)?;
        super::form::global_perform(self)
    }

    /// `VerifGluedFaces(mySkface, theBase, myBCurve, myCurves, theRevol, myGluedF)`: the
    /// curves kept, the glued faces forgotten when a piece of the revolution cut at the
    /// sketch face has no face of the base.
    fn verif_glued_faces(&mut self, the_base: &Face, the_revol: &Revol) -> Result<(), Failure> {
        self.keep_curves(the_revol)?;
        let vrai = the_revol.shape()?.clone();
        if !glued_faces_hold(&self.form.skface, the_base, &self.b_curve, &vrai)? {
            self.form.glued_f.clear(false);
        }
        Ok(())
    }
}
