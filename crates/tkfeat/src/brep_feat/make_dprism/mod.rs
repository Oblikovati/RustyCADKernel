// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeDPrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_MakeDPrism`: a drafted prism (tapered boss or pocket) of a planar face
//! along its normal, of a given height, up to a shape, between two shapes, up to the end
//! or through all of the basis shape; the top and lateral edges of a boss.
mod boss_edges;
mod limited;
mod perform;
mod statics;

use super::common::check_basis_face;
use super::form::impl_form_make_shape;
use super::{Form, FormFeature};
use crate::errors::construction_error;
use crate::loc_ope::DPrism;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `BRepFeat_MakeDPrism`: the drafted prism of the face `pbase` sketched on `skface`,
/// its sides inclined by `angle`, fused (boss) or cut (pocket).
pub struct BRepFeatMakeDPrism {
    pub(crate) form: Form,
    pub(crate) pbase: Face,
    /// The sliding faces of the basis with the profile edges lying in them.
    pub(crate) slface: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(crate) angle: f64,
    pub(crate) curves: Sequence<Handle<dyn GeomCurve>>,
    pub(crate) b_curve: Handle<dyn GeomCurve>,
    pub(crate) top_edges: List<Shape>,
    pub(crate) lat_edges: List<Shape>,
}

impl_form_make_shape!(BRepFeatMakeDPrism);

impl FormFeature for BRepFeatMakeDPrism {
    fn form(&self) -> &Form {
        &self.form
    }

    fn form_mut(&mut self) -> &mut Form {
        &mut self.form
    }

    /// `Curves(scur)`: the generating lines of the drafted prism.
    fn curves(&mut self, s: &mut Sequence<Handle<dyn GeomCurve>>) -> Result<(), Failure> {
        *s = self.curves.clone();
        Ok(())
    }

    /// `BarycCurve()`: the line of the drafted prism through the barycentre of its face.
    fn baryc_curve(&mut self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(self.b_curve.clone())
    }
}

impl Default for BRepFeatMakeDPrism {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepFeatMakeDPrism {
    /// `BRepFeat_MakeDPrism()`: not initialized; `BRepFeatMakeDPrism::new()`.
    pub fn new() -> Self {
        Self {
            form: Form::default(),
            pbase: Face::default(),
            slface: DataMap::default(),
            angle: 0.,
            curves: Sequence::default(),
            b_curve: Handle::default(),
            top_edges: List::default(),
            lat_edges: List::default(),
        }
    }

    /// `BRepFeat_MakeDPrism(Sbase, Pbase, Skface, Angle, Fuse, Modify)`: initialized by
    /// `init`; `BRepFeatMakeDPrism::with_shapes(&sbase, &face, &skface, 0.1, 1, true)?`.
    pub fn with_shapes(
        sbase: &Shape,
        pbase: &Face,
        skface: &Face,
        angle: f64,
        mode: i32,
        modify: bool,
    ) -> Result<Self, Failure> {
        let mut dprism = Self::new();
        dprism.init(sbase, pbase, skface, angle, mode, modify)?;
        Ok(dprism)
    }

    /// `Init(Sbase, Pbase, Skface, Angle, Mode, Modify)`: the basis, the face sketched on
    /// `skface`, the draft angle and the mode (0 cut, 1 fuse, 2 the drafted prism alone);
    /// the history of the basis faces is reset (the new edges kept, as natively);
    /// `dprism.init(&sbase, &face, &skface, 0.1, 0, true)?`.
    pub fn init(
        &mut self,
        sbase: &Shape,
        pbase: &Face,
        skface: &Face,
        angle: f64,
        mode: i32,
        modify: bool,
    ) -> Result<(), Failure> {
        self.form.skface = skface.clone();
        self.form.sketch_face_valid();
        self.form.sbase = sbase.clone();
        self.form.basis_shape_valid();
        self.pbase = pbase.clone();
        self.slface.clear(false);
        self.form.set_mode(mode);
        self.form.modify = modify;
        self.form.just_gluer = false;
        self.top_edges.clear();
        self.lat_edges.clear();
        self.form.reset_map()?;
        self.angle = angle;
        Ok(())
    }

    /// `Add(E, F)`: the edge `e` of the face slides on the face `f` of the basis
    /// (Standard_ConstructionError when either is not one); `dprism.add(&e, &f)?`.
    pub fn add(&mut self, e: &Edge, f: &Face) -> Result<(), Failure> {
        check_basis_face(&self.form.sbase, f)?;
        if !Explorer::new(&self.pbase, ShapeEnum::Edge, ShapeEnum::Shape)?.any(|x| x.is_same(e)) {
            return Err(construction_error(
                "BRepFeat_MakeDPrism::Add",
                "the edge is not an edge of the profile",
            ));
        }
        let edges = self.slface.try_bound((**f).clone(), List::default());
        if !edges.iter().any(|x| x.is_same(e)) {
            edges.append((**e).clone());
        }
        Ok(())
    }

    /// The history of the drafted prism mapped (`MajMap`); its shape returned.
    fn map_dprism(&mut self, the_dprism: &DPrism) -> Result<Shape, Failure> {
        let f = &mut self.form;
        statics::maj_map(
            &self.pbase,
            the_dprism,
            &mut f.map,
            &mut f.f_shape,
            &mut f.l_shape,
        )?;
        Ok(the_dprism.shape()?.clone())
    }

    /// The drafted prism made the generated shape of the feature, its history mapped.
    fn take_dprism(&mut self, the_dprism: &DPrism) -> Result<Shape, Failure> {
        let vrai = self.map_dprism(the_dprism)?;
        self.form.g_shape = vrai.clone();
        self.form.generated_shape_valid();
        Ok(vrai)
    }

    /// The curves of the drafted prism kept, then the topological reconstruction.
    fn global_with(&mut self, the_dprism: &DPrism) -> Result<(), Failure> {
        the_dprism.curves(&mut self.curves)?;
        self.b_curve = the_dprism.baryc_curve()?;
        super::form::global_perform(self)
    }

    /// InvFirstShape when the base of the drafted prism has more than one face (true).
    fn invalid_first_shape(&mut self, the_dprism: &DPrism) -> Result<bool, Failure> {
        let mut exp = Explorer::new(the_dprism.first_shape(), ShapeEnum::Face, ShapeEnum::Shape)?;
        // theBase: natively cast (Standard_TypeMismatch without face) and unused
        rustycad_tkbrep::topods::face(exp.current())?;
        exp.next()?;
        if exp.more() {
            self.form.fail(super::StatusError::InvFirstShape);
            return Ok(true);
        }
        Ok(false)
    }
}
