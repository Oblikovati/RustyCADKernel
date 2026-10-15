// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevolutionForm.cxx
// (the class and Add) and .lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_MakeRevolutionForm`: a rib or groove of revolution: a planar wire on the
//! basis turned around an axis between two parallel planes at the heights `h1` and `h2`
//! of the plane of the wire, sliding on the faces of revolution of the basis when they
//! allow it.
mod bound;
mod init;
mod init_sliding;
mod perform;
mod perform_history;
mod propagate;
mod propagate_walk;

use super::StatusError;
use super::rib_slot::{RibSlot, impl_rib_make_shape};
use crate::errors::construction_error;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Axis1;

/// `BRepFeat_MakeRevolutionForm`: the rib of revolution of the wire in the plane `pln`
/// around `axe`.
pub struct BRepFeatMakeRevolutionForm {
    pub(crate) rib: RibSlot,
    pub(crate) axe: Axis1,
    pub(crate) height1: f64,
    pub(crate) height2: f64,
    /// `mySliding` of the revolution form (it shadows the member of the rib natively).
    pub(crate) sliding: bool,
    pub(crate) pln: Handle<GeomPlane>,
    pub(crate) bnd: f64,
    pub(crate) slface: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(crate) list_of_edges: List<Shape>,
    pub(crate) tol: f64,
    pub(crate) angle1: f64,
    pub(crate) angle2: f64,
}

impl_rib_make_shape!(BRepFeatMakeRevolutionForm);

impl Default for BRepFeatMakeRevolutionForm {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepFeatMakeRevolutionForm {
    /// `BRepFeat_MakeRevolutionForm()`: not initialized, the angles RealLast;
    /// `BRepFeatMakeRevolutionForm::new()`.
    pub fn new() -> Self {
        Self {
            rib: RibSlot::default(),
            axe: Axis1::default(),
            height1: 0.,
            height2: 0.,
            sliding: false,
            pln: Handle::default(),
            bnd: 0.,
            slface: DataMap::default(),
            list_of_edges: List::default(),
            tol: 0.,
            angle1: f64::MAX,
            angle2: f64::MAX,
        }
    }

    /// `BRepFeat_MakeRevolutionForm(Sbase, W, P, Axis, H1, H2, Fuse, Sliding)`:
    /// initialized by `init`;
    /// `BRepFeatMakeRevolutionForm::with_wire(&sbase, &w, &pln, axis, 1., 1., 1, true)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_wire(
        sbase: &Shape,
        w: &Wire,
        p: &Handle<GeomPlane>,
        axis: Axis1,
        h1: f64,
        h2: f64,
        fuse: i32,
        sliding: bool,
    ) -> Result<Self, Failure> {
        let mut form = Self::new();
        form.init(sbase, w, p, axis, h1, h2, fuse, sliding)?;
        Ok(form)
    }

    /// The rib data (`BRepFeat_RibSlot` base): status, first and last shapes, faces for
    /// the draft; `form.rib().current_status_error()`.
    pub fn rib(&self) -> &RibSlot {
        &self.rib
    }

    /// `Add(E, F)`: the edge `e` of the wire slides on the face `f` of the basis, while no
    /// sliding face is known (Standard_ConstructionError when `f` is not a face of the
    /// basis); `form.add(&e, &f)?`.
    pub fn add(&mut self, e: &Edge, f: &Face) -> Result<(), Failure> {
        if !self.slface.is_empty() {
            return Ok(());
        }
        if !Explorer::new(&self.rib.sbase, ShapeEnum::Face, ShapeEnum::Shape)?.any(|x| x.is_same(f))
        {
            return Err(construction_error(
                "BRepFeat_MakeRevolutionForm::Add",
                "the face is not a face of the basis",
            ));
        }
        let edges = self.slface.try_bound((**f).clone(), List::default());
        if !edges.iter().any(|x| x.is_same(e)) {
            edges.append((**e).clone());
        }
        Ok(())
    }

    /// Stops the form with the error `e`.
    pub(crate) fn fail(&mut self, e: StatusError) {
        self.rib.status_error = e;
        self.rib.base.set_not_done();
    }
}
