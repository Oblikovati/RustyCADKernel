// Rust translation of ChFi2d_AnaFilletAlgo::Perform and Cut (ChFi2d_AnaFilletAlgo.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytical fillet: the case of the neighbours, the arc of the fillet, and the
//! neighbours shrunk to it.
use super::ChFi2dAnaFilletAlgo;
use super::cases::FilletDef;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_api::ExtremaCurveCurve;
use rustycad_tkgeombase::proj_lib::project_point_on_plane;
use rustycad_tkmath::elslib::plane_value;
use rustycad_tkmath::gp::{Axis2, Circle3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisWire;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

impl ChFi2dAnaFilletAlgo {
    /// `Perform(radius)`: the fillet of radius `radius` (cut first where arcs intersect);
    /// a neighbour almost disappearing gets a radius 1e-5 smaller; false when no fillet
    /// fits. `algo.perform(1.0)?`.
    pub fn perform(&mut self, radius: f64) -> Result<bool, Failure> {
        if self.e1.is_null() || self.e2.is_null() || radius < precision::confusion() {
            return Ok(false);
        }
        // Analytical algorithm works for non-intersecting arcs only.
        let is_cut = (!self.n1.segment || !self.n2.segment) && self.self_intersects()?;
        if is_cut && !self.cut()? {
            return Ok(false);
        }
        let (ac1, ac2) = (
            BRepAdaptorCurve::with_edge(&self.e1)?,
            BRepAdaptorCurve::with_edge(&self.e2)?,
        );
        let mut def = FilletDef::default();
        if !self.compute_case(radius, &mut def)? {
            return Ok(false);
        }
        // Invert the fillet for left-handed plane.
        def.cw ^= !self.plane.position().direct()?;
        let mut circ = self.fillet_circle(&def, radius)?;
        // Fillet may only shrink a neighbour edge, it can't prolongate it.
        let delta1 = ac1.last_parameter()? - ac1.first_parameter()?;
        let delta2 = ac2.last_parameter()? - ac2.first_parameter()?;
        if !is_cut && (def.start > delta1 || def.end > delta2) {
            let Some(reduced) = self.reduce_radius(radius, (delta1, delta2), &mut def)? else {
                return Ok(false);
            };
            circ = reduced.with_axis(circ)?;
        }
        self.build_fillet(&def, circ, (&ac1, &ac2))
    }

    /// Whether the two edges of the wire intersect themselves (ShapeAnalysis_Wire on
    /// the face of the plane).
    fn self_intersects(&self) -> Result<bool, Failure> {
        let mk_wire = BRepBuilderAPIMakeWire::from_two_edges(&self.e1, &self.e2)?;
        if !mk_wire.is_done() {
            return Ok(false);
        }
        let mk_face = BRepBuilderAPIMakeFace::from_plane(self.plane)?;
        if !mk_face.is_done() {
            return Ok(false);
        }
        let analyzer = ShapeAnalysisWire::from_wire(
            &mk_wire.wire()?,
            &mk_face.face()?,
            precision::confusion(),
        )?;
        analyzer.check_self_intersection()
    }

    /// Choose the case.
    fn compute_case(&self, radius: f64, def: &mut FilletDef) -> Result<bool, Failure> {
        match (self.n1.segment, self.n2.segment) {
            (true, true) => self.segment_fillet_segment(radius, def),
            (true, false) => self.segment_fillet_arc(radius, def),
            (false, true) => self.arc_fillet_segment(radius, def),
            (false, false) => self.arc_fillet_arc(radius, def),
        }
    }

    /// The circle of the fillet: its center in the plane, its axis the normal of the
    /// plane (reversed when clockwise).
    fn fillet_circle(&self, def: &FilletDef, radius: f64) -> Result<Circle3, Failure> {
        let center = plane_value(def.center.0, def.center.1, self.plane.position());
        let normal = self.plane.position().direction();
        let axis = Axis2::new(center, if def.cw { normal.reversed() } else { normal })?;
        Circle3::new(axis, radius)
    }

    /// Check a case when a neighbour edge almost disappears: try to reduce the fillet
    /// radius for a little (1.e-5 mm). The new center and radius, or `None`.
    fn reduce_radius(
        &self,
        radius: f64,
        (delta1, delta2): (f64, f64),
        def: &mut FilletDef,
    ) -> Result<Option<ReducedFillet>, Failure> {
        let little = 100.0 * precision::confusion();
        let (d1, d2) = ((def.start - delta1).abs(), (def.end - delta2).abs());
        if d1 >= little && d2 >= little {
            return Ok(None);
        }
        if !self.compute_case(radius - little, def)? {
            return Ok(None);
        }
        // Invert the fillet for left-handed planes (the circle keeps its axis).
        def.cw ^= !self.plane.position().direct()?;
        let center = plane_value(def.center.0, def.center.1, self.plane.position());
        Ok(Some(ReducedFillet {
            center,
            radius: radius - little,
        }))
    }

    /// The fillet arc between its points on the neighbours and the neighbours limited
    /// there; false when an edge cannot be built.
    fn build_fillet(
        &mut self,
        def: &FilletDef,
        circ: Circle3,
        (ac1, ac2): (&BRepAdaptorCurve, &BRepAdaptorCurve),
    ) -> Result<bool, Failure> {
        let pstart = self.start_point(def, ac1)?;
        let pend = self.end_point(def, ac2)?;
        // Make arc.
        let mk_edge = BRepBuilderAPIMakeEdge::from_circ_points(circ, pstart, pend)?;
        if !mk_edge.is_done() {
            return Ok(false);
        }
        self.fillet = mk_edge.edge()?;
        // Left neighbour, then right neighbour (both by the orientation of the first).
        let forward = self.e1.orientation() == Orientation::Forward;
        let (p1, p2) = if forward {
            (ac1.value(ac1.first_parameter()?)?, pstart)
        } else {
            (pstart, ac1.value(ac1.last_parameter()?)?)
        };
        self.shrinke1 = limited(ac1, p1, p2)?;
        let (p1, p2) = if forward {
            (pend, ac2.value(ac2.last_parameter()?)?)
        } else {
            (ac2.value(ac2.first_parameter()?)?, pend)
        };
        self.shrinke2 = limited(ac2, p1, p2)?;
        Ok(!self.shrinke1.is_null() && !self.shrinke2.is_null())
    }

    /// The start of the fillet: its point on an arc neighbour, else at the shrinking
    /// length from the common end of the first neighbour.
    fn start_point(&self, def: &FilletDef, ac1: &BRepAdaptorCurve) -> Result<Point3, Failure> {
        if def.start_point.0 != f64::MAX {
            return Ok(plane_value(
                def.start_point.0,
                def.start_point.1,
                self.plane.position(),
            ));
        }
        if self.e1.orientation() == Orientation::Forward {
            ac1.value(ac1.last_parameter()? - def.start)
        } else {
            ac1.value(ac1.first_parameter()? + def.start)
        }
    }

    /// The end of the fillet: its point on an arc neighbour, else at the shrinking length
    /// from the common end of the second neighbour.
    fn end_point(&self, def: &FilletDef, ac2: &BRepAdaptorCurve) -> Result<Point3, Failure> {
        if def.end_point.0 != f64::MAX {
            return Ok(plane_value(
                def.end_point.0,
                def.end_point.1,
                self.plane.position(),
            ));
        }
        if self.e2.orientation() == Orientation::Forward {
            ac2.value(ac2.first_parameter()? + def.end)
        } else {
            ac2.value(ac2.last_parameter()? - def.end)
        }
    }

    /// `Cut(plane, e1, e2)`: the edges cut at their first intersection (not at the end
    /// of the first one nor at the start of the second one); the common point of the
    /// neighbours moves there.
    fn cut(&mut self) -> Result<bool, Failure> {
        let (c1, f1, l1) = BRepTool::curve(&self.e1)?;
        let (c2, f2, l2) = BRepTool::curve(&self.e2)?;
        let extrema = ExtremaCurveCurve::with_ranges(&c1, &c2, (f1, l1), (f2, l2))?;
        let Some((param1, param2, p)) = first_crossing(&extrema, l1, f2)? else {
            return Ok(false);
        };
        let mk_edge1 = BRepBuilderAPIMakeEdge::from_curve_params(&c1, f1, param1)?;
        if !mk_edge1.is_done() {
            return Ok(false);
        }
        self.e1 = mk_edge1.edge()?;
        let mk_edge2 = BRepBuilderAPIMakeEdge::from_curve_params(&c2, param2, l2)?;
        if !mk_edge2.is_done() {
            return Ok(false);
        }
        self.e2 = mk_edge2.edge()?;
        let p2d = project_point_on_plane(&self.plane, p);
        self.n1.p2 = (p2d.x(), p2d.y());
        self.n2.p1 = self.n1.p2;
        Ok(true)
    }
}

/// The center and radius of a fillet reduced for a disappearing neighbour.
struct ReducedFillet {
    center: Point3,
    radius: f64,
}

impl ReducedFillet {
    /// The circle again: the old axis direction at the new center and radius.
    fn with_axis(&self, mut circ: Circle3) -> Result<Circle3, Failure> {
        circ.set_location(self.center);
        circ.set_radius(self.radius)?;
        Ok(circ)
    }
}

/// The first extremum of the curves at a distance below Precision::Confusion that is
/// not at the end `l1` of the first one nor at the start `f2` of the second one.
fn first_crossing(
    extrema: &ExtremaCurveCurve,
    l1: f64,
    f2: f64,
) -> Result<Option<(f64, f64, Point3)>, Failure> {
    let tol = precision::confusion();
    for i in 1..=extrema.nb_extrema() {
        if extrema.distance(i)? >= tol {
            continue;
        }
        let (param1, param2) = extrema.parameters(i)?;
        if (l1 - param1).abs() > tol && (f2 - param2).abs() > tol {
            let (p, _) = extrema.points(i)?;
            return Ok(Some((param1, param2, p)));
        }
    }
    Ok(None)
}

/// The edge on the curve of the neighbour between the points (the null edge when it
/// cannot be built).
fn limited(ac: &BRepAdaptorCurve, p1: Point3, p2: Point3) -> Result<Edge, Failure> {
    let curve = ac.transformed_curve().geom_curve();
    let mut mk = BRepBuilderAPIMakeEdge::new();
    mk.init_curve_points(&curve, p1, p2)?;
    if mk.is_done() {
        mk.edge()
    } else {
        Ok(Edge::default())
    }
}
