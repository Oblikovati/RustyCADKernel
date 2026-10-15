// Rust translation of the one-limit Perform methods of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevol.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The revolutions of an angle, up to a shape, through all (a full turn) and of an angle
//! towards a shape.
use super::{BRepFeatMakeRevol, tool_compound};
use crate::brep_feat::PerfSelection;
use crate::brep_feat::common::detect_top_face;
use crate::errors::construction_error;
use crate::loc_ope::{CSIntersector, Revol};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::f64::consts::PI;

/// Standard_ConstructionError unless `s` has a face.
pub(super) fn require_face(s: &Shape) -> Result<(), Failure> {
    if !Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?.more() {
        return Err(construction_error(
            "BRepFeat_MakeRevol::Perform",
            "the limiting shape has no face",
        ));
    }
    Ok(())
}

/// The last solid of `cutsh` having the face `pbase`, else `vrai`.
fn solid_with_base(cutsh: &Shape, pbase: &Shape, vrai: Shape) -> Result<Shape, Failure> {
    let mut chosen = vrai;
    for sol in Explorer::new(cutsh, ShapeEnum::Solid, ShapeEnum::Shape)? {
        if Explorer::new(&sol, ShapeEnum::Face, ShapeEnum::Shape)?.any(|fac| fac.is_same(pbase)) {
            chosen = sol;
        }
    }
    Ok(chosen)
}

impl BRepFeatMakeRevol {
    /// `Perform(Angle)`: the revolution of `angle` fused with or cut from the basis (a
    /// full turn forgets the sketch face); `revol.perform_angle(PI / 2.)?`.
    pub fn perform_angle(&mut self, angle: f64) -> Result<(), Failure> {
        let f = &mut self.form;
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until.nullify();
        f.shape_until_valid();
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::NoSelection;
        f.perf_selection_valid();
        let revol_comp = 2. * PI - angle.abs() <= precision::angular();
        if revol_comp {
            f.skface.nullify();
        }
        let the_revol = Revol::with_axis(&self.pbase, self.axis, angle)?;
        let vrai = self.map_revol(&the_revol)?;
        self.set_generated(&vrai);
        let Some(the_base) = self.first_face(&the_revol)? else {
            return Ok(());
        };
        detect_top_face(&self.form, !self.slface.is_empty(), &self.pbase, &vrai)?;
        self.form.glued_faces_valid();
        if !self.form.skface.is_null() {
            self.verif_glued_faces(&the_base, &the_revol)?;
        }
        if self.form.glued_f.is_empty() {
            let g_shape = self.form.g_shape.clone();
            return self.form.finish_with(&g_shape, false);
        }
        self.global_with(&the_revol)
    }

    /// The shape Until set (extended when it is an infinite face): whether it was.
    pub(super) fn set_until(
        &mut self,
        until: &Shape,
        selection: PerfSelection,
    ) -> Result<bool, Failure> {
        let f = &mut self.form;
        f.glued_f.clear(false);
        f.perf_selection = selection;
        f.perf_selection_valid();
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until = until.clone();
        let trf = f.transform_shape_fu(1)?;
        f.shape_until_valid();
        Ok(trf)
    }

    /// `Perform(Until)`: the revolution up to the shape `until` (a full turn up to the
    /// sketch face, else almost one); `revol.perform_until(&face)?`.
    pub fn perform_until(&mut self, until: &Shape) -> Result<(), Failure> {
        if until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakeRevol::Perform",
                "null limiting shape",
            ));
        }
        require_face(until)?;
        let tour_complet = !self.form.skface.is_null() && until.is_same(&self.form.skface);
        let trf = self.set_until(until, PerfSelection::SelectionU)?;
        // Do systematically almost complete revolution
        let angle = if tour_complet {
            2. * PI
        } else {
            2. * PI - 3. * PI / 180.
        };
        let the_revol = Revol::with_axis(&self.pbase, self.axis, angle)?;
        let vrai = self.map_revol(&the_revol)?;
        self.glue_or_support(&the_revol, &vrai, !trf)
    }

    /// The end of the revolutions up to a shape: the topological reconstruction when
    /// `glue`, else the revolution cut by the solid of the support, its piece holding the
    /// profile fused with or cut from the basis.
    fn glue_or_support(
        &mut self,
        the_revol: &Revol,
        vrai: &Shape,
        glue: bool,
    ) -> Result<(), Failure> {
        if glue {
            self.set_generated(vrai);
            if self.first_face(the_revol)?.is_none() {
                return Ok(());
            }
            self.form.glued_faces_valid();
            return self.global_with(the_revol);
        }
        self.keep_curves(the_revol)?;
        let mut scur = Sequence::default();
        scur.append(self.b_curve.clone());
        let mut asi = CSIntersector::with_shape(&self.form.s_until);
        asi.perform_curves(&scur)?;
        if !asi.is_done() || asi.nb_points(1)? < 1 {
            return Ok(());
        }
        let p = asi.point(1, 1)?;
        let comp = tool_compound(&self.form.s_until, p.face(), p.orientation())?;
        let tr_p = BRepAlgoAPICut::from_shapes(vrai, &comp)?;
        let vrai = solid_with_base(tr_p.shape(), &self.pbase, vrai.clone())?;
        self.form.finish_with(&vrai, false)
    }

    /// `PerformThruAll()`: the full turn; `revol.perform_thru_all()?`.
    pub fn perform_thru_all(&mut self) -> Result<(), Failure> {
        self.perform_angle(2. * PI)
    }

    /// `PerformUntilAngle(Until, Angle)`: the revolution of `angle` towards the shape
    /// `until` (natively the null and zero cases perform and then go on, and the
    /// reconstruction is chosen for an extended face Until);
    /// `revol.perform_until_angle(&face, PI)?`.
    pub fn perform_until_angle(&mut self, until: &Shape, angle: f64) -> Result<(), Failure> {
        if until.is_null() {
            self.perform_angle(angle)?;
        }
        if angle == 0. {
            self.perform_until(until)?;
        }
        require_face(until)?;
        if !self.form.skface.is_null() && until.is_same(&self.form.skface) {
            return self.perform_angle(angle);
        }
        let trf = self.set_until(until, PerfSelection::NoSelection)?;
        let the_revol = Revol::with_axis(&self.pbase, self.axis, angle)?;
        let vrai = self.map_revol(&the_revol)?;
        self.glue_or_support(&the_revol, &vrai, trf)
    }
}
