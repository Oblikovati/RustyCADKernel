// Rust translation of the face part of DBRep_DrawableShape::DrawOn
// (Draw/TKDraw/DBRep/DBRep_DrawableShape.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The isos and triangulations of the faces of a drawable shape.
use super::DBRepDrawableShape;
use super::plot::{Plot, draw_and_check};
use crate::dbrep::edge_face::{DBRepFace, FaceIso};
use crate::draw::DrawDisplay;
use crate::draw::{DrawColor, DrawColorKind};
use crate::draw_tr_surf::TriangulationEdges;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::geom_abs::{CurveType, IsoType, Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};

/// The parameter span below which a range counts as empty in the restriction test.
const ZERO_SPAN: f64 = 1.0e-9;
/// The parameter span above which a face range counts as infinite.
const HUGE_SPAN: f64 = 4.1e100;

/// The walk along one iso: the current point parameters, the end parameters and the
/// steps (`U1`, `V1`, `U2`, `V2`, `stepU`, `stepV`).
#[derive(Debug, Clone, Copy, Default)]
struct IsoWalk {
    u1: f64,
    v1: f64,
    u2: f64,
    v2: f64,
    step_u: f64,
    step_v: f64,
}

/// The surface of a face as the iso drawing reads it.
struct IsoSurface {
    adaptor: BRepAdaptorSurface,
    kind: SurfaceType,
    /// The continuity interval bounds along U and along V.
    u_bounds: Array1<'static, f64>,
    v_bounds: Array1<'static, f64>,
}

impl DBRepDrawableShape {
    /// The face part of `DrawOn`: each face's isos in its color, and its
    /// triangulation when it has no surface or triangulations are shown.
    pub(super) fn draw_faces(
        &self,
        display: &mut DrawDisplay<'_>,
        halt: &mut bool,
    ) -> Result<(), Failure> {
        let faces = self.faces.borrow().clone();
        for face in &faces {
            if *halt {
                break;
            }
            display.set_color(face.color());
            let has_surface = BRepTool::surface_with_location(face.face())?.0.is_some();
            if has_surface {
                self.draw_face_isos(face, display, halt)?;
            }
            if !has_surface || self.triangulations.get() {
                draw_face_triangulation(face.face(), display)?;
            }
        }
        Ok(())
    }

    /// The isos of one face on its (possibly restricted) adaptor.
    fn draw_face_isos(
        &self,
        face: &DBRepFace,
        display: &mut DrawDisplay<'_>,
        halt: &mut bool,
    ) -> Result<(), Failure> {
        let adaptor = BRepAdaptorSurface::with_face(face.face(), restriction(face.face())?)?;
        let surface = IsoSurface {
            kind: adaptor.get_type()?,
            u_bounds: bounds(adaptor.nb_u_intervals(Continuity::CN)?, |b| {
                adaptor.u_intervals(b, Continuity::CN)
            })?,
            v_bounds: bounds(adaptor.nb_v_intervals(Continuity::CN)?, |b| {
                adaptor.v_intervals(b, Continuity::CN)
            })?,
            adaptor,
        };
        for index in 1..=face.nb_isos() {
            let iso = face.iso(index)?;
            self.draw_iso(&surface, iso, (face, display, halt))?;
        }
        Ok(())
    }

    /// One iso: a move to its start, each continuity interval it crosses, then a line
    /// to its end.
    fn draw_iso(
        &self,
        surface: &IsoSurface,
        iso: FaceIso,
        (face, display, halt): (&DBRepFace, &mut DrawDisplay<'_>, &mut bool),
    ) -> Result<(), Failure> {
        let along_v = iso.kind == IsoType::IsoU;
        let ti = if along_v {
            &surface.v_bounds
        } else {
            &surface.u_bounds
        };
        let (start, end) = (
            std_max(iso.first, at(ti, 1)?),
            std_min(iso.last, at(ti, 2)?),
        );
        let mut walk = if along_v {
            IsoWalk {
                u1: iso.parameter,
                v1: start,
                u2: iso.parameter,
                v2: end,
                ..IsoWalk::default()
            }
        } else {
            IsoWalk {
                u1: start,
                v1: iso.parameter,
                u2: end,
                v2: iso.parameter,
                ..IsoWalk::default()
            }
        };
        display.move_to_3d(surface.adaptor.d0(walk.u1, walk.v1)?);
        let nb_intervals = ti.len() as i32 - 1;
        for interval in 1..=nb_intervals {
            let (t1, t2) = (at(ti, interval)?, at(ti, interval + 1)?);
            if (t1 <= iso.first && t2 <= iso.first) || (t1 >= iso.last && t2 >= iso.last) {
                continue;
            }
            let (from, to) = (std_max(iso.first, t1), std_min(iso.last, t2));
            let discret = f64::from(self.discret.get());
            if along_v {
                (walk.v1, walk.v2, walk.step_v) = (from, to, (to - from) / discret);
            } else {
                (walk.u1, walk.u2, walk.step_u) = (from, to, (to - from) / discret);
            }
            self.walk_interval(surface, iso.kind, &mut walk, (face, display, halt))?;
        }
        let end = surface.adaptor.d0(walk.u2, walk.v2)?;
        let face_shape = face.face().as_ref();
        draw_and_check(display, end, (face_shape, walk.u2, walk.v2), halt);
        Ok(())
    }

    /// The points of one interval, by surface type: none on a plane, `discret` steps on
    /// quadrics along circles, adaptive double steps on splines and swept curves.
    fn walk_interval(
        &self,
        surface: &IsoSurface,
        kind: IsoType,
        walk: &mut IsoWalk,
        (face, display, halt): (&DBRepFace, &mut DrawDisplay<'_>, &mut bool),
    ) -> Result<(), Failure> {
        let steps = match surface.kind {
            SurfaceType::Plane => IntervalSteps::None,
            SurfaceType::Cylinder | SurfaceType::Cone => {
                if kind == IsoType::IsoV {
                    IntervalSteps::Uniform
                } else {
                    IntervalSteps::None
                }
            }
            SurfaceType::Sphere
            | SurfaceType::Torus
            | SurfaceType::OffsetSurface
            | SurfaceType::OtherSurface => IntervalSteps::Uniform,
            SurfaceType::BezierSurface | SurfaceType::BSplineSurface => IntervalSteps::Adaptive,
            SurfaceType::SurfaceOfExtrusion | SurfaceType::SurfaceOfRevolution => {
                swept_steps(surface, kind)?
            }
        };
        match steps {
            IntervalSteps::None => Ok(()),
            IntervalSteps::Uniform => self.uniform_steps(surface, walk, (face, display, halt)),
            IntervalSteps::Adaptive => {
                self.adaptive_steps(surface, kind, walk, (face, display, halt))
            }
        }
    }

    /// `discret - 1` points at equal steps.
    fn uniform_steps(
        &self,
        surface: &IsoSurface,
        walk: &mut IsoWalk,
        (face, display, halt): (&DBRepFace, &mut DrawDisplay<'_>, &mut bool),
    ) -> Result<(), Failure> {
        for _ in 1..self.discret.get() {
            walk.u1 += walk.step_u;
            walk.v1 += walk.step_v;
            let point = surface.adaptor.d0(walk.u1, walk.v1)?;
            draw_and_check(
                display,
                point,
                (face.face().as_ref(), walk.u1, walk.v1),
                halt,
            );
        }
        Ok(())
    }

    /// `discret / 2` double steps, each plotted with `PlotIso`.
    fn adaptive_steps(
        &self,
        surface: &IsoSurface,
        kind: IsoType,
        walk: &mut IsoWalk,
        (face, display, halt): (&DBRepFace, &mut DrawDisplay<'_>, &mut bool),
    ) -> Result<(), Failure> {
        let step = if kind == IsoType::IsoV {
            walk.step_u * 2.
        } else {
            walk.step_v * 2.
        };
        for _ in 1..=self.discret.get() / 2 {
            let shape = face.face().as_ref();
            Plot::new(display, shape, halt).iso(
                &surface.adaptor,
                kind,
                (walk.u1, walk.v1),
                step,
            )?;
            walk.u1 += walk.step_u * 2.;
            walk.v1 += walk.step_v * 2.;
        }
        Ok(())
    }
}

/// How the points inside one continuity interval are generated.
enum IntervalSteps {
    None,
    Uniform,
    Adaptive,
}

/// The swept surfaces: along the sweep (V isos of a revolution) by uniform steps,
/// nothing along an extrusion direction, else by the type of the basis curve.
fn swept_steps(surface: &IsoSurface, kind: IsoType) -> Result<IntervalSteps, Failure> {
    let revolution = surface.kind == SurfaceType::SurfaceOfRevolution;
    let along_sweep =
        (kind == IsoType::IsoV && revolution) || (kind == IsoType::IsoU && !revolution);
    if along_sweep {
        return Ok(if revolution {
            IntervalSteps::Uniform
        } else {
            IntervalSteps::None
        });
    }
    let basis = surface.adaptor.basis_curve()?;
    let curve_type = basis
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "basis curve"))?
        .get_type()?;
    Ok(match curve_type {
        CurveType::Line => IntervalSteps::None,
        CurveType::Circle | CurveType::Ellipse => IntervalSteps::Uniform,
        _ => IntervalSteps::Adaptive,
    })
}

/// The restriction flag of the face adaptor: a face of a periodic surface whose UV
/// bounds leave the surface bounds is restricted to its wires, unless one of the
/// ranges is empty, reversed or huge.
fn restriction(face: &Face) -> Result<bool, Failure> {
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let Some(surface) = surface.get() else {
        return Ok(false);
    };
    let (u_periodic, v_periodic) = (surface.is_u_periodic(), surface.is_v_periodic());
    if !u_periodic && !v_periodic {
        return Ok(false);
    }
    let (su1, su2, sv1, sv2) = surface.bounds();
    let (fu1, fu2, fv1, fv2) = uv_bounds(face)?;
    let outside = |f: f64, s1: f64, s2: f64| f < s1 || f > s2;
    let mut restricted = u_periodic && (outside(fu1, su1, su2) || outside(fu2, su1, su2));
    if !restricted && v_periodic {
        restricted = outside(fv1, sv1, sv2) || outside(fv2, sv1, sv2);
    }
    let zero_surface = (su2 - su1).abs() <= ZERO_SPAN || (sv2 - sv1).abs() <= ZERO_SPAN;
    let zero_face = (fu2 - fu1).abs() <= ZERO_SPAN || (fv2 - fv1).abs() <= ZERO_SPAN;
    let reversed = fu1 >= fu2 || fv1 >= fv2;
    let huge = (fu2 - fu1).abs() > HUGE_SPAN || (fv2 - fv1).abs() > HUGE_SPAN;
    Ok(restricted && !(zero_surface || zero_face) && !reversed && !huge)
}

/// `NCollection_Array1<double> TI(1, nb + 1)` filled by `fill`.
fn bounds(
    nb: i32,
    fill: impl FnOnce(&mut Array1<'static, f64>) -> Result<(), Failure>,
) -> Result<Array1<'static, f64>, Failure> {
    crate::draw_tr_surf::interval_bounds(nb, fill)
}

/// `TI(index)`.
fn at(bounds: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    crate::draw_tr_surf::bound(bounds, index)
}

/// The triangulation of a face (located): free edges in red, internal ones in blue.
fn draw_face_triangulation(face: &Face, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
    let (triangulation, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    if triangulation.is_null() {
        return Ok(());
    }
    draw_triangulation(&triangulation, location.transformation(), display)
}

/// `DBRep_DrawableShape::display(T, tr, dis)`.
pub(super) fn draw_triangulation(
    triangulation: &Handle<Triangulation>,
    transform: Transform3,
    display: &mut DrawDisplay<'_>,
) -> Result<(), Failure> {
    let Some(mesh) = triangulation.get() else {
        return Ok(());
    };
    let edges = TriangulationEdges::compute(triangulation)?;
    let sets = [
        (DrawColorKind::Rouge, &edges.free),
        (DrawColorKind::Bleu, &edges.internal),
    ];
    for (color, pairs) in sets {
        display.set_color(DrawColor::new(color));
        for (n1, n2) in pairs {
            let (p1, p2) = (mesh.node(*n1)?, mesh.node(*n2)?);
            display.draw_3d(p1.transformed(transform), p2.transformed(transform));
        }
    }
    Ok(())
}
