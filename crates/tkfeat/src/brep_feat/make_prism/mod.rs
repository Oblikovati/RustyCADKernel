// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakePrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_MakePrism`: a prism (boss or pocket) of a planar profile along a direction,
//! of a given length, up to a shape, between two shapes, up to the end or through all of
//! the basis shape.
mod limited;
mod perform;
mod statics;

use super::common::check_basis_face;
use super::form::impl_form_make_shape;
use super::{Form, FormFeature};
use crate::errors::construction_error;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Direction3;

/// `BRepFeat_MakePrism`: the prism of the profile `pbase` sketched on the face `skface`
/// of the basis, fused (boss) or cut (pocket).
pub struct BRepFeatMakePrism {
    pub(crate) form: Form,
    pub(crate) pbase: Shape,
    /// The sliding faces of the basis with the profile edges lying in them.
    pub(crate) slface: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(crate) dir: Direction3,
    pub(crate) curves: Sequence<Handle<dyn GeomCurve>>,
    pub(crate) b_curve: Handle<dyn GeomCurve>,
}

impl_form_make_shape!(BRepFeatMakePrism);

impl FormFeature for BRepFeatMakePrism {
    fn form(&self) -> &Form {
        &self.form
    }

    fn form_mut(&mut self) -> &mut Form {
        &mut self.form
    }

    /// `Curves(scur)`: the generating lines of the prism.
    fn curves(&mut self, s: &mut Sequence<Handle<dyn GeomCurve>>) -> Result<(), Failure> {
        *s = self.curves.clone();
        Ok(())
    }

    /// `BarycCurve()`: the line of the prism through the barycentre of its profile.
    fn baryc_curve(&mut self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(self.b_curve.clone())
    }
}

impl Default for BRepFeatMakePrism {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepFeatMakePrism {
    /// `BRepFeat_MakePrism()`: not initialized; `BRepFeatMakePrism::new()`.
    pub fn new() -> Self {
        Self {
            form: Form::default(),
            pbase: Shape::default(),
            slface: DataMap::default(),
            dir: Direction3::default(),
            curves: Sequence::default(),
            b_curve: Handle::default(),
        }
    }

    /// `BRepFeat_MakePrism(Sbase, Pbase, Skface, Direction, Fuse, Modify)`: initialized
    /// by `init`; `BRepFeatMakePrism::with_shapes(&sbase, &pbase, &skface, dir, 1, true)?`.
    pub fn with_shapes(
        sbase: &Shape,
        pbase: &Shape,
        skface: &Face,
        direc: Direction3,
        mode: i32,
        modify: bool,
    ) -> Result<Self, Failure> {
        let mut prism = Self::new();
        prism.init(sbase, pbase, skface, direc, mode, modify)?;
        Ok(prism)
    }

    /// `Init(Sbase, Pbase, Skface, Direc, Mode, Modify)`: the basis, the profile sketched
    /// on `skface`, the direction and the mode (0 cut, 1 fuse, 2 the prism alone); the
    /// history of the basis faces is reset; `prism.init(&sbase, &pbase, &skface, dir, 0,
    /// true)?`.
    pub fn init(
        &mut self,
        sbase: &Shape,
        pbase: &Shape,
        skface: &Face,
        direc: Direction3,
        mode: i32,
        modify: bool,
    ) -> Result<(), Failure> {
        self.form.skface = skface.clone();
        self.form.sketch_face_valid();
        self.form.sbase = sbase.clone();
        self.form.basis_shape_valid();
        self.pbase = pbase.clone();
        self.slface.clear(false);
        self.dir = direc;
        self.form.set_mode(mode);
        self.form.modify = modify;
        self.form.just_gluer = false;
        self.form.reset_history()
    }

    /// `Add(E, F)`: the edge `e` of the profile slides on the face `f` of the basis
    /// (Standard_ConstructionError when either is not one); `prism.add(&e, &f)?`.
    pub fn add(&mut self, e: &Edge, f: &Face) -> Result<(), Failure> {
        check_basis_face(&self.form.sbase, f)?;
        if !Explorer::new(&self.pbase, ShapeEnum::Edge, ShapeEnum::Shape)?.any(|x| x.is_same(e)) {
            return Err(construction_error(
                "BRepFeat_MakePrism::Add",
                "the edge is not an edge of the profile",
            ));
        }
        let edges = self.slface.try_bound((**f).clone(), List::default());
        if !edges.iter().any(|x| x.is_same(e)) {
            edges.append((**e).clone());
        }
        Ok(())
    }
}
