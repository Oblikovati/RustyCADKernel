// Rust translation of the topology members of TopOpeBRepDS_BuildTool (MakeEdge(E, C, BDS),
// GetOrientedEdgeVertices, the Add*, Parameter, Range, UpdateEdge, Curve3D, Orientation,
// Closed and UpdateSurface members and the static Project in
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_BuildTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Edges, vertices and faces updated by the build tool.
use super::{TopOpeBRepDSBuildTool, typed};
use crate::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSDataStructure, TopOpeBRepDSSurfaceCurveInterference,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::elclib;
use rustycad_tkmath::toploc::Location;

/// The oriented vertices of an edge with their parameters: `(Vmin, Vmax, Parmin,
/// Parmax)`, the parameters `None` without both vertices.
pub type OrientedEdgeVertices = (Vertex, Vertex, Option<(f64, f64)>);

/// `Project(C, V, p)` (static): the parameter of the first minimum of the extrema of
/// the vertex point on the curve within the vertex tolerance.
fn project(c: &Handle<dyn GeomCurve>, v: &Vertex) -> Result<Option<f64>, Failure> {
    let p = BRepTool::pnt(v)?;
    let tol = BRepTool::vertex_tolerance(v)?;
    let gac = GeomAdaptorCurve::with_curve(c)?;
    let extrema = ExtPC::with_curve(p, &gac, 1.0e-10)?;
    if !extrema.is_done() {
        return Ok(None);
    }
    for i in 1..=extrema.nb_ext()? {
        if extrema.is_min(i)? {
            let epoc = extrema.point(i)?;
            if p.distance(epoc.value()) <= tol {
                return Ok(Some(epoc.parameter()));
            }
        }
    }
    Ok(None)
}

impl TopOpeBRepDSBuildTool {
    /// `MakeEdge(E, C, BDS)`: an edge on the curve, or, without curve, a degenerated
    /// edge with the p-curve of the first surface/curve interference of the curve on
    /// its surface; `bt.make_edge_on_ds(&mut e, &c, &ds)?`.
    pub fn make_edge_on_ds(
        &self,
        e: &mut Shape,
        c: &TopOpeBRepDSCurve,
        bds: &TopOpeBRepDSDataStructure,
    ) -> Result<(), Failure> {
        let mut ee: Edge = typed(e, topods::edge)?;
        if !c.curve().is_null() {
            self.my_builder
                .make_edge_with_curve(&mut ee, c.curve(), c.tolerance())?;
            *e = ee.into_shape();
            return Ok(());
        }
        self.my_builder.make_edge(&mut ee)?;
        self.my_builder.degenerated(&ee, true)?;
        let sci: Handle<TopOpeBRepDSSurfaceCurveInterference> = c.get_sci1().clone().cast();
        let sci = sci.get().ok_or_else(|| Failure::with_type("Standard_NullObject", "TopOpeBRepDS_BuildTool::MakeEdge: the curve has no surface/curve interference; expected a TopOpeBRepDS_SurfaceCurveInterference", "TopOpeBRepDS_BuildTool"))?;
        let dss = bds.surface(sci.support());
        let gs: &Handle<dyn GeomSurface> = dss.surface();
        self.my_builder.update_edge_pcurve(
            &ee,
            &sci.pcurve(),
            gs,
            &Location::default(),
            dss.tolerance(),
            None,
        )?;
        *e = ee.into_shape();
        Ok(())
    }

    /// `GetOrientedEdgeVertices(E, Vmin, Vmax, Parmin, Parmax)`: the vertices in the
    /// edge orientation, with their parameters when both exist;
    /// `let (vmin, vmax, pars) = bt.get_oriented_edge_vertices(&e)?;`.
    pub fn get_oriented_edge_vertices(&self, e: &Edge) -> Result<OrientedEdgeVertices, Failure> {
        let (first, last) = topexp::vertices(e, false);
        let (vmin, vmax) = if e.orientation() == Orientation::Forward {
            (first, last)
        } else {
            (last, first)
        };
        if vmin.as_ref().is_null() || vmax.as_ref().is_null() {
            return Ok((vmin, vmax, None));
        }
        let pars = (
            BRepTool::parameter(&vmin, e)?,
            BRepTool::parameter(&vmax, e)?,
        );
        Ok((vmin, vmax, Some(pars)))
    }

    /// `AddEdgeVertex(Ein, Eou, V)`: the vertex (reversed when `ein` is) added to
    /// `eou`, its point representations transferred from `ein`;
    /// `bt.add_edge_vertex_from(&e, &mut copy, &v)?`.
    pub fn add_edge_vertex_from(
        &self,
        ein: &Shape,
        eou: &mut Shape,
        v: &Shape,
    ) -> Result<(), Failure> {
        let mut e1 = topods::edge(ein)?;
        let mut e2 = topods::edge(eou)?;
        let mut v1 = topods::vertex(v)?;
        if e1.orientation() == Orientation::Reversed {
            v1.reverse();
        }
        e1.set_orientation(Orientation::Forward);
        e2.set_orientation(Orientation::Forward);
        let mut e2s = e2.clone().into_shape();
        self.my_builder.add(&mut e2s, v1.as_ref())?;
        self.my_builder
            .transfert_vertex(&e1, &topods::edge(&e2s)?, &v1, &v1)
    }

    /// `AddEdgeVertex(E, V)`; `bt.add_edge_vertex(&mut e, &v)?`.
    pub fn add_edge_vertex(&self, e: &mut Shape, v: &Shape) -> Result<(), Failure> {
        self.my_builder.add(e, v)
    }

    /// `AddWireEdge(W, E)`; `bt.add_wire_edge(&mut w, &e)?`.
    pub fn add_wire_edge(&self, w: &mut Shape, e: &Shape) -> Result<(), Failure> {
        self.my_builder.add(w, e)
    }

    /// `AddFaceWire(F, W)`; `bt.add_face_wire(&mut f, &w)?`.
    pub fn add_face_wire(&self, f: &mut Shape, w: &Shape) -> Result<(), Failure> {
        self.my_builder.add(f, w)
    }

    /// `AddShellFace(Sh, F)`; `bt.add_shell_face(&mut sh, &f)?`.
    pub fn add_shell_face(&self, sh: &mut Shape, f: &Shape) -> Result<(), Failure> {
        self.my_builder.add(sh, f)
    }

    /// `AddSolidShell(S, Sh)`; `bt.add_solid_shell(&mut s, &sh)?`.
    pub fn add_solid_shell(&self, s: &mut Shape, sh: &Shape) -> Result<(), Failure> {
        self.my_builder.add(s, sh)
    }

    /// `Parameter(E, V, P)`: the vertex parameter on the edge (tolerance 0); on a
    /// periodic curve, the parameter of a REVERSED vertex before the curve start is
    /// brought into the period; `bt.parameter(&e, &v, 1.)?`.
    pub fn parameter(&self, e: &Shape, v: &Shape, p: f64) -> Result<(), Failure> {
        let (edge, vertex) = (topods::edge(e)?, topods::vertex(v)?);
        let mut p = p;
        let found = BRepTool::curve_with_location(&edge)?;
        if let Some(c) = found.curve.get()
            && c.is_periodic()
        {
            let per = c.period()?;
            let ov = Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)?
                .find(|vofe| vofe.is_same(v))
                .map(|vofe| vofe.orientation());
            if ov == Some(Orientation::Reversed) && p < found.first {
                p = elclib::in_period(p, found.first, found.first + per);
            }
        }
        self.my_builder.update_vertex_on_edge(&vertex, p, &edge, 0.)
    }

    /// `Parameter(C, E, V)`: the parameter of the projection of the vertex on the
    /// curve; natively an uninitialized parameter is used when the projection fails,
    /// Standard_ProgramError here; `bt.parameter_on_curve(&c, &mut e, &mut v)?`.
    pub fn parameter_on_curve(
        &self,
        c: &TopOpeBRepDSCurve,
        e: &mut Shape,
        v: &mut Shape,
    ) -> Result<(), Failure> {
        let newparam = project(c.curve(), &topods::vertex(v)?)?.ok_or_else(|| Failure::with_type("Standard_ProgramError", "TopOpeBRepDS_BuildTool::Parameter: the vertex does not project on the curve; expected a vertex within its tolerance of the curve", "TopOpeBRepDS_BuildTool"))?;
        self.parameter(e, v, newparam)
    }

    /// `Range(E, first, last)`; `bt.range(&e, 0., 1.)?`.
    pub fn range(&self, e: &Shape, first: f64, last: f64) -> Result<(), Failure> {
        self.my_builder.range(&topods::edge(e)?, first, last, false)
    }

    /// `UpdateEdge(Ein, Eou)`: on periodic curves, an empty range of `eou` adjusted onto
    /// the range of `ein`; `bt.update_edge(&e, &mut copy)?`.
    pub fn update_edge(&self, ein: &Shape, eou: &mut Shape) -> Result<(), Failure> {
        let cin = BRepTool::curve_with_location(&topods::edge(ein)?)?;
        let cou = BRepTool::curve_with_location(&topods::edge(eou)?)?;
        let Some(c) = cou.curve.get() else {
            return Ok(());
        };
        if cin.curve.is_null() || !c.is_periodic() || cou.last > cou.first {
            return Ok(());
        }
        let (f2n, l2n) = elclib::adjust_periodic(
            cin.first,
            cin.last,
            precision::p_confusion(),
            cou.first,
            cou.last,
        );
        self.range(eou, f2n, l2n)
    }

    /// `Curve3D(E, C, Tol)`: the 3d curve of the edge replaced; `bt.curve3d(&mut e, &c, 1e-7)?`.
    pub fn curve3d(
        &self,
        e: &mut Shape,
        c: &Handle<dyn GeomCurve>,
        tol: f64,
    ) -> Result<(), Failure> {
        self.my_builder.update_edge_curve(&topods::edge(e)?, c, tol)
    }

    /// `Orientation(S, O)`; `bt.set_orientation(&mut s, Orientation::Reversed)`.
    pub fn set_orientation(&self, s: &mut Shape, o: Orientation) {
        s.set_orientation(o);
    }

    /// `Orientation(S)`; `bt.orientation(&s)`.
    pub fn orientation(&self, s: &Shape) -> Orientation {
        s.orientation()
    }

    /// `Closed(S, B)`: the closed flag of the shape; `bt.closed(&mut s, true)?`.
    pub fn closed(&self, s: &mut Shape, b: bool) -> Result<(), Failure> {
        let tshape = s.tshape().get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "TopOpeBRepDS_BuildTool::Closed: null shape; expected a shape",
                "TopOpeBRepDS_BuildTool",
            )
        })?;
        tshape.set_closed(b);
        Ok(())
    }

    /// `UpdateSurface(F, SU)`: the surface of the face replaced (identity location,
    /// face tolerance); `bt.update_surface(&f, &surface)?`.
    pub fn update_surface(&self, f: &Shape, su: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        let face = topods::face(f)?;
        let tol = BRepTool::face_tolerance(&face)?;
        self.my_builder
            .update_face(&face, su, &Location::default(), tol)
    }

    /// `UpdateSurface(E, oldF, newF)`: the p-curve of the edge on `old_f` set on
    /// `new_f` with the tolerance of `old_f`; `bt.update_surface_of_edge(&e, &old, &new)?`.
    pub fn update_surface_of_edge(
        &self,
        e: &Shape,
        old_f: &Shape,
        new_f: &Shape,
    ) -> Result<(), Failure> {
        let edge = topods::edge(e)?;
        let old = topods::face(old_f)?;
        let pc = BRepTool::curve_on_face(&edge, &old)?.pcurve;
        let tol = BRepTool::face_tolerance(&old)?;
        self.my_builder
            .update_edge_pcurve_on_face(&edge, &pc, &topods::face(new_f)?, tol)
    }
}
