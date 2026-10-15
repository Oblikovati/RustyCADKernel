// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeLinearForm.cxx
// (Add, Perform and the statics) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_MakeLinearForm`: a rib or slot along a planar wire on the basis, swept along
//! a direction (and back along a second one), sliding on the faces of the basis when
//! they allow it.
mod init;
mod init_base;
mod propagate;

use super::common::maj_map_swept;
use super::rib_slot::{RibSlot, impl_rib_make_shape};
use super::{PerfSelection, StatusError};
use crate::errors::construction_error;
use crate::loc_ope::LinearForm;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Vector3};

/// `BRepFeat_MakeLinearForm`: the rib or slot of the wire `wire` in the plane `pln`
/// swept along `dir` (and `dir1`).
pub struct BRepFeatMakeLinearForm {
    pub(crate) rib: RibSlot,
    pub(crate) dir: Vector3,
    pub(crate) dir1: Vector3,
    pub(crate) pln: Handle<GeomPlane>,
    pub(crate) bnd: f64,
    pub(crate) slface: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(crate) list_of_edges: List<Shape>,
    pub(crate) tol: f64,
}

impl_rib_make_shape!(BRepFeatMakeLinearForm);

impl Default for BRepFeatMakeLinearForm {
    fn default() -> Self {
        Self::new()
    }
}

/// The null vector the directions are compared with.
pub(super) fn nulldir() -> Vector3 {
    Vector3::new(0., 0., 0.)
}

/// `SetGluedFaces(theSlmap, thePrism, theMap)`: the face the form sweeps from each
/// sliding edge glued on its sliding face.
fn set_glued_faces(
    the_slmap: &DataMap<Shape, List<Shape>, ShapeMapHasher>,
    the_prism: &LinearForm,
    the_map: &mut DataMap<Shape, Shape, ShapeMapHasher>,
) -> Result<(), Failure> {
    for (fac, ledg) in the_slmap.items() {
        for e in ledg.iter() {
            let gfac = the_prism.shapes(e)?;
            // natively a list of another length is only reported in debug
            let first = gfac.first().cloned().ok_or_else(|| {
                crate::errors::no_such_object("SetGluedFaces", "no face swept from the edge")
            })?;
            the_map.bind(first, fac.clone());
        }
    }
    Ok(())
}

impl BRepFeatMakeLinearForm {
    /// `BRepFeat_MakeLinearForm()`: not initialized; `BRepFeatMakeLinearForm::new()`.
    pub fn new() -> Self {
        Self {
            rib: RibSlot::default(),
            dir: Vector3::new(0., 0., 0.),
            dir1: Vector3::new(0., 0., 0.),
            pln: Handle::default(),
            bnd: 0.,
            slface: DataMap::default(),
            list_of_edges: List::default(),
            tol: 0.,
        }
    }

    /// `BRepFeat_MakeLinearForm(Sbase, W, P, Direc, Direc1, Fuse, Modify)`: initialized
    /// by `init`; `BRepFeatMakeLinearForm::with_wire(&sbase, &w, &pln, d, d1, 1, true)?`.
    pub fn with_wire(
        sbase: &Shape,
        w: &rustycad_tkbrep::topods::Wire,
        p: &Handle<GeomPlane>,
        direc: Vector3,
        direc1: Vector3,
        fuse: i32,
        modify: bool,
    ) -> Result<Self, Failure> {
        let mut form = Self::new();
        form.init(sbase, w, p, direc, direc1, fuse, modify)?;
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
                "BRepFeat_MakeLinearForm::Add",
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

    /// The swept form of the profile (translated back along `dir1` when not null).
    fn sweep(&self, length: f64) -> Result<LinearForm, Failure> {
        let dir = Direction3::from_vector(self.dir)?;
        let v = Vector3::from(dir) * length;
        let r = &self.rib;
        let mut the_form = LinearForm::new();
        if self
            .dir1
            .is_equal(nulldir(), precision::confusion(), precision::confusion())?
        {
            the_form.perform(&r.pbase, v, r.first_pnt, r.last_pnt)?;
        } else {
            the_form.perform_translated(&r.pbase, v, self.dir1, r.first_pnt, r.last_pnt)?;
        }
        Ok(the_form)
    }

    /// `Perform()`: the rib or slot swept, glued on its sliding faces or combined with
    /// the basis; `form.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        let r = &mut self.rib;
        if r.sbase.is_null() || r.skface.is_null() || r.pbase.is_null() {
            self.fail(StatusError::NotInitialized);
            return Ok(());
        }
        let length = self.dir.magnitude() + self.dir1.magnitude();
        r.glued_f.clear(false);
        r.perf_selection = if r.s_until.is_null() {
            PerfSelection::NoSelection
        } else {
            PerfSelection::SelectionU
        };
        let the_form = self.sweep(length)?;
        let vrai_form = the_form.shape()?.clone(); // primitive of the rib
        let r = &mut self.rib;
        r.faces_for_draft.append(the_form.first_shape().clone());
        r.faces_for_draft.append(the_form.last_shape().clone());
        let ends = (the_form.first_shape(), the_form.last_shape());
        maj_map_swept(
            &r.pbase.clone().into_shape(),
            ends,
            |e| Ok(the_form.shapes(e)?.clone()),
            &mut r.map,
            &mut r.f_shape,
            &mut r.l_shape,
        )?;
        for e in Explorer::new(&r.pbase, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !r.map.is_bound(&e) {
                self.fail(StatusError::IncSlidFace);
                return Ok(());
            }
        }
        let r = &mut self.rib;
        r.g_shape = vrai_form;
        set_glued_faces(&self.slface, &the_form, &mut r.glued_f)?; // management of sliding faces
        if !r.glued_f.is_empty() && !r.s_until.is_null() {
            self.fail(StatusError::InvShape);
            return Ok(());
        }
        self.rib.lf_perform()
    }
}
