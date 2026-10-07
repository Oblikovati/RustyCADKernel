//! The ShapeConstruct package functions, the Rust twin of
//! tools/oracle/tkshhealing_probe_construct_package.cxx (which documents the requests).
use super::brep_lib_common::{geom_curve, geom2d_curve};
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::face::{surface as tp_surface, tp_face};
use super::sh_common::{curve, curve2d, surface};
use super::shape_construct::{construct_spline_2d, construct_spline_3d};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Axis1;
use rustycad_tkshhealing::shape_construct::{
    ShapeConstruct, ShapeConstructApproxSettings, ShapeConstructSurfaceBounds,
};

const CONTINUITIES: [Continuity; 7] = [
    Continuity::C0,
    Continuity::G1,
    Continuity::C1,
    Continuity::G2,
    Continuity::C2,
    Continuity::C3,
    Continuity::CN,
];

fn settings(r: &mut Reader<'_>, tol: f64) -> ShapeConstructApproxSettings {
    ShapeConstructApproxSettings {
        tol3d: tol,
        continuity: CONTINUITIES[r.integer() as usize],
        max_segments: r.integer(),
        max_degree: r.integer(),
    }
}

/// `scp_bspline_surface`: the B-spline data of a surface.
pub fn bspline_surface(handle: &Handle<GeomBSplineSurface>) {
    let Some(s) = handle.get() else { return };
    [s.u_degree(), s.v_degree(), s.nb_u_poles(), s.nb_v_poles()]
        .into_iter()
        .for_each(int);
    [
        s.is_u_periodic(),
        s.is_v_periodic(),
        s.is_u_rational(),
        s.is_v_rational(),
    ]
    .into_iter()
    .for_each(flag);
    for i in 1..=s.nb_u_knots() {
        real(s.u_knot(i).unwrap());
        int(s.u_multiplicity(i).unwrap());
    }
    for i in 1..=s.nb_v_knots() {
        real(s.v_knot(i).unwrap());
        int(s.v_multiplicity(i).unwrap());
    }
    for i in 1..=s.nb_u_poles() {
        for j in 1..=s.nb_v_poles() {
            pnt(s.pole(i, j).unwrap());
            real(s.weight(i, j).unwrap());
        }
    }
}

/// `scp_surface`: the surface of one record.
pub fn probe_surface(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    match kind {
        0 => tp_surface(r),
        1 => {
            let c = geom_curve(r)?;
            Ok(Handle::new(GeomSurfaceOfLinearExtrusion::new(&c, r.dir())?).cast())
        }
        2 => {
            let c = geom_curve(r)?;
            let p = r.pnt();
            Ok(Handle::new(GeomSurfaceOfRevolution::new(&c, Axis1::new(p, r.dir()))?).cast())
        }
        _ => {
            let basis = probe_surface(r)?;
            let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
            let trimmed: Handle<dyn GeomSurface> = Handle::new(GeomRectangularTrimmedSurface::new(
                &basis,
                (u1, u2),
                (v1, v2),
                (true, true),
            )?)
            .cast();
            if kind == 3 {
                return Ok(trimmed);
            }
            Ok(rustycad_tkgeombase::geom_convert::surface_to_bspline_surface(&trimmed)?.cast())
        }
    }
}

fn curve_case(r: &mut Reader<'_>) -> Result<(), Failure> {
    let c = geom_curve(r)?;
    let (first, last, tol) = (r.real(), r.real(), r.real());
    let s = settings(r, tol);
    attempt(|| {
        let b = ShapeConstruct::convert_curve_to_bspline(
            &c,
            first,
            last,
            s.tol3d,
            s.continuity,
            s.max_segments,
            s.max_degree,
        )?;
        flag(b == c);
        curve(&b.cast());
        construct_spline_3d(&b.cast());
        Ok(())
    });
    Ok(())
}

fn curve2d_case(r: &mut Reader<'_>) -> Result<(), Failure> {
    let c = geom2d_curve(r)?;
    let (first, last, tol) = (r.real(), r.real(), r.real());
    let s = settings(r, tol);
    attempt(|| {
        let b = ShapeConstruct::convert_curve2d_to_bspline(
            &c,
            first,
            last,
            s.tol3d,
            s.continuity,
            s.max_segments,
            s.max_degree,
        )?;
        flag(b == c);
        curve2d(&b.cast());
        construct_spline_2d(&b.cast());
        Ok(())
    });
    Ok(())
}

fn surface_case(r: &mut Reader<'_>) {
    let mut s: Handle<dyn GeomSurface> = Handle::default();
    attempt(|| {
        s = probe_surface(r)?;
        Ok(())
    });
    let bounds = ShapeConstructSurfaceBounds {
        uf: r.real(),
        ul: r.real(),
        vf: r.real(),
        vl: r.real(),
    };
    let tol = r.real();
    let settings = settings(r, tol);
    if s.is_null() {
        return;
    }
    attempt(|| {
        let b = ShapeConstruct::convert_surface_to_bspline(&s, bounds, settings)?;
        surface(&b.clone().cast());
        bspline_surface(&b);
        Ok(())
    });
}

fn pcurve(edge: &Edge, face: &Face) -> Result<(), Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    real(found.first);
    real(found.last);
    curve2d(&found.pcurve);
    construct_spline_2d(&found.pcurve);
    Ok(())
}

fn join_case(r: &mut Reader<'_>) -> Result<(), Failure> {
    let t = tp_face(r)?;
    let mut chain: Sequence<Shape> = Sequence::default();
    for _ in 0..r.integer() {
        let index = r.integer() as usize;
        let orientation = super::face::orientation(r);
        chain.append(t.edges[index].oriented(orientation));
    }
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge)?;
    flag(ShapeConstruct::join_pcurves(&chain, &t.face, &mut edge));
    flag(BRepTool::same_range(&edge)?);
    flag(BRepTool::same_parameter(&edge)?);
    flag(BRepTool::is_closed_on_face(&edge, &t.face)?);
    attempt(|| pcurve(&edge, &t.face));
    let mut reversed = edge.clone();
    reversed.reverse();
    attempt(|| pcurve(&reversed, &t.face));
    Ok(())
}

/// `sh_construct_pkg op ...`.
pub fn construct_package_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => curve_case(&mut r).expect("probe curve record"),
        1 => curve2d_case(&mut r).expect("probe curve record"),
        2 => surface_case(&mut r),
        _ => join_case(&mut r).expect("probe face record"),
    }
    end();
}
