// Rust translation of the post-processing of IntPatch_ImpImpIntersection::Perform of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The restrictions of both domains against the other quadric, the tangent faces, and
//! the vertices of the lines.
use super::super::IntPatchIType;
use super::super::aline::IntPatchALine;
use super::super::gline::IntPatchGLine;
use super::super::line::IntPatchLine;
use super::super::path_point::{
    IntPatchThePathPointOfTheSOnBounds, IntPatchTheSegmentOfTheSOnBounds,
};
use super::super::point::IntPatchPoint;
use super::super::rline::IntPatchRLine;
use super::perform::{PairFlags, PerformInput, arc_function};
use super::put_points::{PutPointsInput, put_points_on_line};
use super::rline::process_rline;
use super::segments::{SegmentInput, process_segments};
use super::{IntPatchImpImpIntersection, IntPatchImpImpStatus};
use crate::int_start::TheTopolTool;
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};
use std::f64::consts::PI;

/// The solutions on the restrictions of one domain.
#[derive(Default)]
struct DomainSolutions {
    pnt: Vec<IntPatchThePathPointOfTheSOnBounds>,
    edg: Vec<IntPatchTheSegmentOfTheSOnBounds>,
    /// All the arcs are solutions (with surfaces of one type).
    all: bool,
    /// No point nor segment on the restrictions.
    nosol: bool,
}

impl IntPatchImpImpIntersection {
    /// The restrictions of the domain `d` of `s` against `quad`: false when the search
    /// fails.
    fn domain_solutions(
        &mut self,
        (quad, s, d): (
            &IntSurfQuadric,
            &Adaptor3dSurfaceHandle,
            &(dyn TheTopolTool + 'static),
        ),
        (tol_arc, tol_tang): (f64, f64),
        same_type: bool,
    ) -> Result<Option<DomainSolutions>, Failure> {
        let mut afunc = arc_function(quad, s);
        self.solrst
            .perform(&mut afunc, d, tol_arc, tol_tang, false)?;
        if !self.solrst.is_done() {
            return Ok(None);
        }
        let mut sol = DomainSolutions {
            all: self.solrst.all_arc_solution()? && same_type,
            ..DomainSolutions::default()
        };
        let (nbpt, nbseg) = (self.solrst.nb_points()?, self.solrst.nb_segments()?);
        for i in 1..=nbpt {
            sol.pnt.push(self.solrst.point(i)?.clone());
        }
        for i in 1..=nbseg {
            sol.edg.push(self.solrst.segment(i)?.clone());
        }
        sol.nosol = nbpt == 0 && nbseg == 0;
        if sol.nosol && sol.all {
            // A face without restrictions.
            sol.all = false;
        }
        Ok(Some(sol))
    }

    /// The post-processing on the restrictions; false when the computation stops there
    /// (a failed search, or tangent faces).
    pub(super) fn post_process(
        &mut self,
        input: &PerformInput<'_>,
        (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
        (typs1, typs2): (SurfaceType, SurfaceType),
        flags: &PairFlags,
    ) -> Result<bool, Failure> {
        let tols = (input.tol_arc, input.tol_tang);
        let same: bool = typs1 == typs2;
        let nosol_same = DomainSolutions {
            nosol: true,
            ..DomainSolutions::default()
        };
        let sol1 = if flags.same_surf {
            nosol_same
        } else {
            match self.domain_solutions((quad2, input.s1, input.d1), tols, same)? {
                Some(sol) => sol,
                None => return Ok(false),
            }
        };
        let sol2 = if flags.same_surf {
            DomainSolutions {
                nosol: true,
                ..DomainSolutions::default()
            }
        } else {
            match self.domain_solutions((quad1, input.s2, input.d2), tols, same)? {
                Some(sol) => sol,
                None => return Ok(false),
            }
        };
        if flags.same_surf || (sol1.all && sol2.all) {
            self.tangent_faces_result(input.s1, typs1, (quad1, quad2))?;
            return Ok(false);
        }
        if sol1.nosol && sol2.nosol {
            self.empt = self.slin.is_empty() && self.spnt.is_empty();
            return Ok(true);
        }
        self.empt = false;
        self.on_restrictions(input, (quad1, quad2), flags.multpoint, (&sol1, &sol2))?;
        Ok(true)
    }

    /// Perfectly "parallel" faces: tangent, and opposite when the normals at a
    /// reference point of the first surface are opposite.
    fn tangent_faces_result(
        &mut self,
        s1: &Adaptor3dSurfaceHandle,
        typs1: SurfaceType,
        (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
    ) -> Result<(), Failure> {
        self.empt = false;
        self.tgte = true;
        self.slin.clear();
        self.spnt.clear();
        let ptreference: Point3 = match typs1 {
            SurfaceType::Plane => s1.plane()?.location(),
            SurfaceType::Cylinder => elslib::value(0.0, 0.0, &s1.cylinder()?),
            SurfaceType::Sphere => elslib::value(PI / 4.0, PI / 4.0, &s1.sphere()?),
            SurfaceType::Cone => elslib::value(0.0, 10.0, &s1.cone()?),
            SurfaceType::Torus => elslib::value(0.0, 0.0, &s1.torus()?),
            _ => Point3::default(),
        };
        self.oppo = quad1
            .normale_at(ptreference)?
            .dot(quad2.normale_at(ptreference)?)
            < 0.0;
        self.my_done = IntPatchImpImpStatus::Ok;
        Ok(())
    }

    /// The points and segments of both domains put on the lines, then the restriction
    /// lines processed.
    fn on_restrictions(
        &mut self,
        input: &PerformInput<'_>,
        (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
        multpoint: bool,
        (sol1, sol2): (&DomainSolutions, &DomainSolutions),
    ) -> Result<(), Failure> {
        let tolarc: f64 = input.tol_arc;
        let (s1, s2) = (input.s1, input.s2);
        let put1 = PutPointsInput {
            s1,
            s2,
            listpnt: &sol1.pnt,
            on_first: true,
            domain: input.d1,
            quad_surf: quad1,
            other_quad: quad2,
            multpoint,
            tolarc,
        };
        put_points_on_line(&put1, &self.slin)?;
        let put2 = PutPointsInput {
            s1,
            s2,
            listpnt: &sol2.pnt,
            on_first: false,
            domain: input.d2,
            quad_surf: quad2,
            other_quad: quad1,
            multpoint,
            tolarc,
        };
        put_points_on_line(&put2, &self.slin)?;
        if !sol1.edg.is_empty() {
            let seg = SegmentInput {
                quad1,
                quad2,
                on_first: true,
                tol_arc: tolarc,
            };
            process_segments(&sol1.edg, &mut self.slin, &seg)?;
        }
        if !sol2.edg.is_empty() {
            let seg = SegmentInput {
                quad1,
                quad2,
                on_first: false,
                tol_arc: tolarc,
            };
            process_segments(&sol2.edg, &mut self.slin, &seg)?;
        }
        if !sol1.edg.is_empty() || !sol2.edg.is_empty() {
            process_rline(&mut self.slin, (quad1, quad2), tolarc, input.keep_rline)?;
        }
        Ok(())
    }

    /// The isolated points inside both domains, the vertex parameters of the lines, two
    /// vertices on the closed conics without one; the computation is done.
    pub(super) fn finish(
        &mut self,
        input: &PerformInput<'_>,
        quads: (&IntSurfQuadric, &IntSurfQuadric),
    ) -> Result<(), Failure> {
        let nblin: usize = self.slin.len();
        if !self.spnt.is_empty() {
            let mut kept: Vec<IntPatchPoint> = Vec::new();
            for ip in &self.spnt {
                let (u1, v1, u2, v2) = ip.parameters();
                let state1: State = input
                    .d1
                    .classify(Point2::new(u1, v1), input.tol_arc, true)?;
                let state2: State = input
                    .d2
                    .classify(Point2::new(u2, v2), input.tol_arc, true)?;
                if state1 != State::Out && state2 != State::Out {
                    kept.push(ip.clone());
                }
            }
            self.spnt = kept;
        }
        for line in &self.slin[..nblin] {
            compute_vertex_parameters(line, input.tol_arc)?;
        }
        for line in &self.slin[..nblin] {
            closed_conic_vertices(line, quads, input.tol_arc)?;
        }
        self.my_done = IntPatchImpImpStatus::Ok;
        Ok(())
    }
}

/// `ComputeVertexParameters(TolArc)` of a conic, analytic or restriction line.
fn compute_vertex_parameters(line: &Handle<dyn IntPatchLine>, tol_arc: f64) -> Result<(), Failure> {
    match line.arc_type() {
        IntPatchIType::Ellipse
        | IntPatchIType::Circle
        | IntPatchIType::Lin
        | IntPatchIType::Parabola
        | IntPatchIType::Hyperbola => line
            .cast::<IntPatchGLine>()
            .compute_vertex_parameters(tol_arc),
        IntPatchIType::Analytic => line
            .cast::<IntPatchALine>()
            .compute_vertex_parameters(tol_arc),
        IntPatchIType::Restriction => line
            .cast::<IntPatchRLine>()
            .compute_vertex_parameters(tol_arc),
        IntPatchIType::Walking => Ok(()),
    }
}

/// Two vertices (at 0 and 2 PI, on the point of parameter 0) on a circle or an ellipse
/// without vertices.
fn closed_conic_vertices(
    line: &Handle<dyn IntPatchLine>,
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
    tol_arc: f64,
) -> Result<(), Failure> {
    let typ: IntPatchIType = line.arc_type();
    if typ != IntPatchIType::Circle && typ != IntPatchIType::Ellipse {
        return Ok(());
    }
    let glin: Handle<IntPatchGLine> = line.cast();
    if glin.nb_vertex() != 0 {
        return Ok(());
    }
    let p: Point3 = if typ == IntPatchIType::Circle {
        elclib::value(0.0, &glin.circle()?)
    } else {
        elclib::value(0.0, &glin.ellipse()?)
    };
    let (u1, v1) = quad1.parameters(p).unwrap_or((0.0, 0.0));
    let (u2, v2) = quad2.parameters(p).unwrap_or((0.0, 0.0));
    let mut point = IntPatchPoint::new();
    for param in [0.0, PI + PI] {
        point.set_value_tangent(p, tol_arc, false);
        point.set_parameters(u1, v1, u2, v2);
        point.set_parameter(param);
        glin.add_vertex(&point)?;
    }
    Ok(())
}
