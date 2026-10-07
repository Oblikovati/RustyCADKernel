//! ShapeAlgo_AlgoContainer probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_shape_algo.cxx (which documents the requests).
use super::brep_lib_common::{geom_curve, geom2d_curve, shape};
use super::common::{Reader, attempt, end, flag, int, real};
use super::construct_package::{bspline_surface, probe_surface};
use super::sh_common::{curve, curve2d, shapes, surface};
use super::shape_construct::{construct_spline_2d, construct_spline_3d};
use super::shapes::TpShapes;
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomSurface};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkshhealing::shape_algo::ShapeAlgoAlgoContainer;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisWire;
use rustycad_tkshhealing::shape_construct::{
    ShapeConstructApproxSettings, ShapeConstructSurfaceBounds,
};
use rustycad_tkshhealing::shape_extend::ShapeExtendWireData;

const CONTINUITIES: [Continuity; 7] = [
    Continuity::C0,
    Continuity::G1,
    Continuity::C1,
    Continuity::G2,
    Continuity::C2,
    Continuity::C3,
    Continuity::CN,
];

fn spline3d(c: &Handle<dyn GeomCurve>, input: &Handle<GeomBSplineCurve>) {
    flag(*c == *input);
    curve(c);
    construct_spline_3d(c);
}

fn spline2d(c: &Handle<dyn Geom2dCurve>, input: &Handle<Geom2dBSplineCurve>) {
    flag(*c == *input);
    curve2d(c);
    construct_spline_2d(c);
}

fn approx_case(r: &mut Reader<'_>, a: &ShapeAlgoAlgoContainer) -> Result<(), Failure> {
    let b = geom_curve(r)?.cast::<GeomBSplineCurve>();
    if b.is_null() {
        int(-1);
        return Ok(());
    }
    attempt(|| {
        let seq = a.approx_bspline_curve(&b)?;
        int(seq.length());
        seq.iter().for_each(|c| spline3d(c, &b));
        Ok(())
    });
    Ok(())
}

fn approx2d_case(r: &mut Reader<'_>, a: &ShapeAlgoAlgoContainer) -> Result<(), Failure> {
    let b = geom2d_curve(r)?.cast::<Geom2dBSplineCurve>();
    if b.is_null() {
        int(-1);
        return Ok(());
    }
    attempt(|| {
        let seq = a.approx_bspline_curve_2d(&b)?;
        int(seq.length());
        seq.iter().for_each(|c| spline2d(c, &b));
        Ok(())
    });
    Ok(())
}

fn split_case(r: &mut Reader<'_>, a: &ShapeAlgoAlgoContainer) -> Result<(), Failure> {
    let b = geom_curve(r)?.cast::<GeomBSplineCurve>();
    if b.is_null() {
        int(-1);
        return Ok(());
    }
    attempt(|| {
        let seq = a.c0_bspline_to_sequence_of_c1_bspline_curve(&b)?;
        flag(seq.is_some());
        let Some(seq) = seq else { return Ok(()) };
        int(seq.sequence(|items| items.len()) as i32);
        seq.sequence(|items| items.iter().for_each(|c| spline3d(&c.clone().cast(), &b)));
        Ok(())
    });
    Ok(())
}

fn split2d_case(r: &mut Reader<'_>, a: &ShapeAlgoAlgoContainer) -> Result<(), Failure> {
    let b = geom2d_curve(r)?.cast::<Geom2dBSplineCurve>();
    if b.is_null() {
        int(-1);
        return Ok(());
    }
    attempt(|| {
        let seq = a.c0_bspline2d_to_sequence_of_c1_bspline_curve(&b)?;
        flag(seq.is_some());
        let Some(seq) = seq else { return Ok(()) };
        int(seq.sequence(|items| items.len()) as i32);
        seq.sequence(|items| items.iter().for_each(|c| spline2d(&c.clone().cast(), &b)));
        Ok(())
    });
    Ok(())
}

fn curve_case(r: &mut Reader<'_>, a: &ShapeAlgoAlgoContainer) -> Result<(), Failure> {
    let c = geom_curve(r)?;
    let (first, last, tol) = (r.real(), r.real(), r.real());
    let settings = ShapeConstructApproxSettings {
        tol3d: tol,
        continuity: CONTINUITIES[r.integer() as usize],
        max_segments: r.integer(),
        max_degree: r.integer(),
    };
    attempt(|| {
        let b = a.convert_curve_to_bspline(&c, first, last, settings)?;
        flag(b == c);
        curve(&b.cast());
        construct_spline_3d(&b.cast());
        Ok(())
    });
    Ok(())
}

fn surface_case(r: &mut Reader<'_>, a: &ShapeAlgoAlgoContainer, periodic: bool) {
    let mut s: Handle<dyn GeomSurface> = Handle::default();
    attempt(|| {
        s = probe_surface(r)?;
        Ok(())
    });
    let mut bounds = ShapeConstructSurfaceBounds {
        uf: 0.,
        ul: 0.,
        vf: 0.,
        vl: 0.,
    };
    if !periodic {
        (bounds.uf, bounds.ul, bounds.vf, bounds.vl) = (r.real(), r.real(), r.real(), r.real());
    }
    if s.is_null() {
        return;
    }
    if periodic {
        attempt(|| {
            let p = a.convert_to_periodic(&s)?;
            flag(p == s);
            surface(&p);
            bspline_surface(&p.cast::<GeomBSplineSurface>());
            Ok(())
        });
        return;
    }
    attempt(|| {
        let b = a.convert_surface_to_bspline(&s, bounds)?;
        surface(&b.clone().cast());
        bspline_surface(&b);
        Ok(())
    });
}

/// `sh_algo_geom op ...`.
pub fn geom_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let a = ShapeAlgoAlgoContainer::new();
    let op = r.integer();
    let read = match op {
        0 => approx_case(&mut r, &a),
        1 => approx2d_case(&mut r, &a),
        2 => split_case(&mut r, &a),
        3 => split2d_case(&mut r, &a),
        4 => curve_case(&mut r, &a),
        _ => {
            surface_case(&mut r, &a, op == 6);
            Ok(())
        }
    };
    read.expect("probe curve record");
    end();
}

fn wire_pair(
    r: &mut Reader<'_>,
    t: &TpShapes,
) -> (
    rustycad_tkbrep::topods::Shape,
    rustycad_tkbrep::topods::Shape,
) {
    let w1 = t.shapes[r.integer() as usize].clone();
    (w1, t.shapes[r.integer() as usize].clone())
}

fn homo_case(r: &mut Reader<'_>, t: &TpShapes, a: &ShapeAlgoAlgoContainer) {
    let (w1, w2) = wire_pair(r, t);
    attempt(|| {
        let out = a.homo_wires(&topods::wire(&w1)?, &topods::wire(&w2)?, false)?;
        flag(out.is_some());
        let Some((o1, o2)) = out else { return Ok(()) };
        shape(&o1)?;
        shape(&o2)
    });
}

fn connect_case(r: &mut Reader<'_>, t: &TpShapes, a: &ShapeAlgoAlgoContainer) {
    let (w1, w2) = wire_pair(r, t);
    let (prec, max_tol) = (r.real(), r.real());
    attempt(|| {
        let sewd = Handle::new(ShapeExtendWireData::from_wire(
            &topods::wire(&w1)?,
            true,
            true,
        )?);
        let next = Handle::new(ShapeExtendWireData::from_wire(
            &topods::wire(&w2)?,
            true,
            true,
        )?);
        let saw = Handle::new(ShapeAnalysisWire::new());
        saw.load_wire_data(&sewd);
        saw.set_precision(prec);
        let c = a.connect_next_wire(&saw, &next, max_tol)?;
        flag(c.connected);
        real(c.distance);
        flag(c.reversed_wire);
        flag(c.reversed_next);
        shape(&sewd.wire()?.into_shape())?;
        shape(&next.wire()?.into_shape())
    });
}

fn face_case(r: &mut Reader<'_>, t: &TpShapes, a: &ShapeAlgoAlgoContainer) {
    let face = topods::face(&t.shapes[r.integer() as usize]).expect("probe face index");
    attempt(|| shape(&a.outer_wire(&face)?.into_shape()));
    attempt(|| {
        let (u1, u2, v1, v2) = a.get_face_uv_bounds(&face)?;
        [u1, u2, v1, v2].into_iter().for_each(real);
        Ok(())
    });
}

fn c1_case(r: &mut Reader<'_>, t: &TpShapes, a: &ShapeAlgoAlgoContainer) {
    let s = t.shapes[r.integer() as usize].clone();
    let tol = r.real();
    attempt(|| shape(&a.c0_shape_to_c1_shape(&s, tol)?));
}

/// `sh_algo_shape (prelude) program nops (op)...`.
pub fn shape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = shapes(&mut r).expect("probe shape program");
    let a = ShapeAlgoAlgoContainer::new();
    for _ in 0..r.integer() {
        match r.integer() {
            0 => homo_case(&mut r, &t, &a),
            1 => connect_case(&mut r, &t, &a),
            2 => face_case(&mut r, &t, &a),
            _ => c1_case(&mut r, &t, &a),
        }
    }
    end();
}
