// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakePipe.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_MakePipe`: a pipe feature, a profile swept along a spine wire, fused with
//! or cut from the basis, alone or limited by shapes.
use super::common::{check_basis_face, maj_map_swept};
use super::form::impl_form_make_shape;
use super::{Form, FormFeature, PerfSelection};
use crate::errors::construction_error;
use crate::loc_ope::Pipe;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `BRepFeat_MakePipe`: the pipe of the profile `pbase` sketched on `skface` along the
/// wire `spine`.
pub struct BRepFeatMakePipe {
    pub(crate) form: Form,
    pub(crate) pbase: Shape,
    /// The sliding faces of the basis with the profile edges lying in them.
    pub(crate) slface: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(crate) spine: Wire,
    pub(crate) curves: Sequence<Handle<dyn GeomCurve>>,
    pub(crate) b_curve: Handle<dyn GeomCurve>,
}

impl_form_make_shape!(BRepFeatMakePipe);

impl FormFeature for BRepFeatMakePipe {
    fn form(&self) -> &Form {
        &self.form
    }

    fn form_mut(&mut self) -> &mut Form {
        &mut self.form
    }

    /// `Curves(scur)`: the pipe lines through the sample points of the first shape.
    fn curves(&mut self, s: &mut Sequence<Handle<dyn GeomCurve>>) -> Result<(), Failure> {
        *s = self.curves.clone();
        Ok(())
    }

    /// `BarycCurve()`: the pipe line through the barycentre of the first shape.
    fn baryc_curve(&mut self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(self.b_curve.clone())
    }
}

impl Default for BRepFeatMakePipe {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard_ConstructionError unless `s` has a face.
fn require_face(s: &Shape) -> Result<(), Failure> {
    if !Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?.more() {
        return Err(construction_error(
            "BRepFeat_MakePipe::Perform",
            "the limiting shape has no face",
        ));
    }
    Ok(())
}

impl BRepFeatMakePipe {
    /// `BRepFeat_MakePipe()`: not initialized; `BRepFeatMakePipe::new()`.
    pub fn new() -> Self {
        Self {
            form: Form::default(),
            pbase: Shape::default(),
            slface: DataMap::default(),
            spine: Wire::default(),
            curves: Sequence::default(),
            b_curve: Handle::default(),
        }
    }

    /// `BRepFeat_MakePipe(Sbase, Pbase, Skface, Spine, Fuse, Modify)`: initialized by
    /// `init`; `BRepFeatMakePipe::with_shapes(&sbase, &pbase, &skface, &spine, 1, true)?`.
    pub fn with_shapes(
        sbase: &Shape,
        pbase: &Shape,
        skface: &Face,
        spine: &Wire,
        mode: i32,
        modify: bool,
    ) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        pipe.init(sbase, pbase, skface, spine, mode, modify)?;
        Ok(pipe)
    }

    /// `Init(Sbase, Pbase, Skface, Spine, Mode, Modify)`: the basis, the profile sketched
    /// on `skface`, the spine and the mode (0 cut, 1 fuse, 2 the pipe alone); the history
    /// of the basis faces is reset (the new edges kept, as natively);
    /// `pipe.init(&sbase, &pbase, &skface, &spine, 0, true)?`.
    pub fn init(
        &mut self,
        sbase: &Shape,
        pbase: &Shape,
        skface: &Face,
        spine: &Wire,
        mode: i32,
        modify: bool,
    ) -> Result<(), Failure> {
        self.form.sbase = sbase.clone();
        self.form.basis_shape_valid();
        self.form.skface = skface.clone();
        self.form.sketch_face_valid();
        self.pbase = pbase.clone();
        self.slface.clear(false);
        self.spine = spine.clone();
        self.form.set_mode(mode);
        self.form.modify = modify;
        self.form.just_gluer = false;
        self.form.reset_map()
    }

    /// `Add(E, F)`: the edge `e` of the profile slides on the face `f` of the basis
    /// (Standard_ConstructionError when either is not one); `pipe.add(&e, &f)?`.
    pub fn add(&mut self, e: &Edge, f: &Face) -> Result<(), Failure> {
        check_basis_face(&self.form.sbase, f)?;
        if !Explorer::new(&self.pbase, ShapeEnum::Edge, ShapeEnum::Shape)?.any(|x| x.is_same(e)) {
            return Err(construction_error(
                "BRepFeat_MakePipe::Add",
                "the edge is not an edge of the profile",
            ));
        }
        let edges = self.slface.try_bound((**f).clone(), List::default());
        if !edges.iter().any(|x| x.is_same(e)) {
            edges.append((**e).clone());
        }
        Ok(())
    }

    /// The pipe built, its history mapped and made the generated shape.
    fn make_pipe(&mut self) -> Result<Pipe, Failure> {
        let mut the_pipe = Pipe::new(&self.spine, &self.pbase)?;
        let vrai = the_pipe.shape().clone();
        let ends = (
            the_pipe.first_shape().clone(),
            the_pipe.last_shape().clone(),
        );
        let f = &mut self.form;
        maj_map_swept(
            &self.pbase,
            (&ends.0, &ends.1),
            |e| Ok(the_pipe.shapes(e)?.clone()),
            &mut f.map,
            &mut f.f_shape,
            &mut f.l_shape,
        )?;
        f.g_shape = vrai;
        f.generated_shape_valid();
        f.glued_faces_valid();
        Ok(the_pipe)
    }

    /// The first shape of the pipe kept, the pipe lines through its sample points, then
    /// the topological reconstruction.
    fn global_with(&mut self, the_pipe: &mut Pipe) -> Result<(), Failure> {
        self.form.f_shape = the_pipe.first_shape().clone();
        let mut spt = Sequence::default();
        crate::loc_ope::sample_edges(&self.form.f_shape, &mut spt)?;
        self.curves = the_pipe.curves(&spt)?.clone();
        self.b_curve = the_pipe.baryc_curve()?;
        super::form::global_perform(self)
    }

    /// `Perform()`: the pipe along the whole spine fused with or cut from the basis;
    /// `pipe.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        let f = &mut self.form;
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until.nullify();
        f.shape_until_valid();
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::NoSelection;
        f.perf_selection_valid();
        let mut the_pipe = self.make_pipe()?;
        if self.form.glued_f.is_empty() {
            let g_shape = self.form.g_shape.clone();
            return self.form.finish_with(&g_shape, false);
        }
        self.global_with(&mut the_pipe)
    }

    /// `Perform(Until)`: the pipe up to the shape `until`; `pipe.perform_until(&face)?`.
    pub fn perform_until(&mut self, until: &Shape) -> Result<(), Failure> {
        if until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakePipe::Perform",
                "null limiting shape",
            ));
        }
        require_face(until)?;
        let f = &mut self.form;
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::SelectionU;
        f.perf_selection_valid();
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until = until.clone();
        f.transform_shape_fu(1)?;
        f.shape_until_valid();
        let mut the_pipe = self.make_pipe()?;
        self.global_with(&mut the_pipe)
    }

    /// `Perform(From, Until)`: the pipe between the shapes `from` and `until` (up to the
    /// other one when either is the sketch face); `pipe.perform_from_until(&a, &b)?`.
    pub fn perform_from_until(&mut self, from: &Shape, until: &Shape) -> Result<(), Failure> {
        if from.is_null() || until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakePipe::Perform",
                "null limiting shape",
            ));
        }
        if !self.form.skface.is_null() {
            if from.is_same(&self.form.skface) {
                return self.perform_until(until);
            } else if until.is_same(&self.form.skface) {
                return self.perform_until(from);
            }
        }
        let f = &mut self.form;
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::SelectionFU;
        f.perf_selection_valid();
        require_face(from)?;
        require_face(until)?;
        f.s_from = from.clone();
        f.transform_shape_fu(0)?;
        f.shape_from_valid();
        f.s_until = until.clone();
        f.transform_shape_fu(1)?;
        f.shape_until_valid();
        let mut the_pipe = self.make_pipe()?;
        self.global_with(&mut the_pipe)
    }
}
