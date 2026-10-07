//! Shape-level ShapeFix probe operation, the Rust twin of
//! tools/oracle/tkshhealing_probe_fix_shape.cxx (which documents the requests).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag};
use super::fix_shape_checks;
use super::fix_shape_wires;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere,
    BRepPrimAPIMakeTorus,
};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{
    FixRoot, ShapeFix, ShapeFixFixSmallFace, ShapeFixFixSmallSolid, ShapeFixShape, ShapeFixShell,
    ShapeFixSolid, ShapeFixWireframe,
};

pub(super) const STATUSES: [ShapeExtendStatus; 11] = [
    ShapeExtendStatus::Done1,
    ShapeExtendStatus::Done2,
    ShapeExtendStatus::Done3,
    ShapeExtendStatus::Done4,
    ShapeExtendStatus::Done5,
    ShapeExtendStatus::Done6,
    ShapeExtendStatus::Done7,
    ShapeExtendStatus::Done8,
    ShapeExtendStatus::Fail1,
    ShapeExtendStatus::Fail2,
    ShapeExtendStatus::Fail3,
];

fn solid(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let solid = match r.integer() {
        0 => {
            let radius = r.real();
            BRepPrimAPIMakeCylinder::new(radius, r.real()).solid()?
        }
        1 => {
            let (r1, r2) = (r.real(), r.real());
            BRepPrimAPIMakeCone::new(r1, r2, r.real())?.solid()?
        }
        2 => BRepPrimAPIMakeSphere::new(r.real())?.solid()?,
        _ => {
            let r1 = r.real();
            BRepPrimAPIMakeTorus::new(r1, r.real())?.solid()?
        }
    };
    Ok(solid.into_shape())
}

fn input(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    match r.integer() {
        0 => super::unify::input(r),
        1 => Ok(super::fix_face::input(r)?.into_shape()),
        2 => solid(r),
        3 => fix_shape_checks::bezier_face(r),
        4 => fix_shape_checks::raw_face(r),
        5 => fix_shape_checks::sphere_face(r),
        _ => fix_shape_wires::multi_wire_face(r),
    }
}

fn shape_fix(s: &Shape, prec: f64, maxtol: f64) -> Result<(), Failure> {
    let fixer = Handle::new(ShapeFixShape::new(s.clone()));
    FixRoot::set_precision(&*fixer, prec);
    FixRoot::set_max_tolerance(&*fixer, maxtol);
    flag(fixer.perform()?);
    STATUSES.iter().for_each(|&x| flag(fixer.status(x)));
    shape(&fixer.shape())
}

fn shells(s: &Shape) -> Result<(), Failure> {
    for shell in Explorer::new(s, ShapeEnum::Shell, ShapeEnum::Shape)? {
        let fixer = Handle::new(ShapeFixShell::from_shell(&topods::shell(&shell)?));
        attempt(|| {
            flag(fixer.perform()?);
            STATUSES.iter().for_each(|&x| flag(fixer.status_flag(x)));
            shape(&fixer.shape())
        });
    }
    Ok(())
}

fn solids(s: &Shape) -> Result<(), Failure> {
    for solid in Explorer::new(s, ShapeEnum::Solid, ShapeEnum::Shape)? {
        let fixer = Handle::new(ShapeFixSolid::from_solid(&topods::solid(&solid)?));
        attempt(|| {
            flag(fixer.perform()?);
            STATUSES.iter().for_each(|&x| flag(fixer.status_flag(x)));
            shape(&fixer.shape())
        });
    }
    Ok(())
}

fn wireframe(s: &Shape, prec: f64, maxtol: f64, op: i32) -> Result<(), Failure> {
    let fixer = Handle::new(ShapeFixWireframe::from_shape(s));
    FixRoot::set_precision(&*fixer, prec);
    FixRoot::set_max_tolerance(&*fixer, maxtol);
    if op == 11 {
        fixer.set_mode_drop_small_edges(true);
    }
    if op == 12 {
        fixer.set_limit_angle(0.1);
    }
    flag(fixer.fix_wire_gaps()?);
    flag(fixer.fix_small_edges()?);
    for &x in &STATUSES {
        flag(fixer.status_wire_gaps(x));
        flag(fixer.status_small_edges(x));
    }
    shape(&fixer.shape())
}

fn strip_edge(edge: &Edge) -> Result<(), Failure> {
    flag(!edge.is_null());
    if edge.is_null() {
        return Ok(());
    }
    shape(edge)
}

fn small_faces(s: &Shape, prec: f64) -> Result<(), Failure> {
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    let faces: Vec<Shape> = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    for face in &faces {
        let face = topods::face(face)?;
        attempt(|| {
            flag(analyzer.check_spot_face(&face, prec)?);
            Ok(())
        });
    }
    for face in &faces {
        let face = topods::face(face)?;
        let (mut e1, mut e2) = (Edge::default(), Edge::default());
        attempt(|| {
            flag(analyzer.check_strip_face(&face, &mut e1, &mut e2, prec)?);
            strip_edge(&e1)?;
            strip_edge(&e2)
        });
    }
    Ok(())
}

fn strips(s: &Shape, prec: f64, fix: bool) -> Result<(), Failure> {
    let fixer = Handle::new(ShapeFixFixSmallFace::default());
    fixer.init(s.clone());
    FixRoot::set_precision(&*fixer, prec);
    let ctx = fixer.context();
    let ctx = ctx.get().ok_or_else(null_context)?;
    for face in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        let applied = ctx.apply(&face, ShapeEnum::Shape)?;
        if applied.is_null() {
            continue;
        }
        let mut face = topods::face(&applied)?;
        let (mut e1, mut e2) = (Edge::default(), Edge::default());
        if !ShapeAnalysisCheckSmallFace::new().check_strip_face(&face, &mut e1, &mut e2, prec)? {
            continue;
        }
        let ok = fixer.replace_in_case_of_strip(&mut face, &mut e1, &mut e2, prec)?;
        flag(ok);
        if ok {
            fixer.remove_faces_in_case_of_strip(&face)?;
        }
    }
    shape(&ctx.apply(s, ShapeEnum::Shape)?)?;
    if fix {
        fixer.init(ctx.apply(s, ShapeEnum::Shape)?);
        shape(&fixer.fix_shape()?)?;
    }
    Ok(())
}

fn run(mut s: Shape, prec: f64, maxtol: f64, op: i32) -> Result<(), Failure> {
    let ctx = Handle::new(ShapeBuildReShape::new());
    match op {
        0 => shape_fix(&s, prec, maxtol),
        1 | 2 => {
            flag(ShapeFix::same_parameter(
                &s,
                op == 2,
                0.,
                &ProgressRange::default(),
                &Handle::default(),
            )?);
            shape(&s)
        }
        3 => {
            flag(ShapeFix::fix_vertex_position(&mut s, maxtol, &ctx)?);
            shape(
                &ctx.get()
                    .ok_or_else(null_context)?
                    .apply(&s, ShapeEnum::Shape)?,
            )
        }
        4 => shape(&ShapeFix::remove_small_edges(&s, maxtol)?.shape),
        5 | 11 | 12 => wireframe(&s, prec, maxtol, op),
        6 => {
            let fixer = Handle::new(ShapeFixFixSmallFace::default());
            fixer.init(s.clone());
            FixRoot::set_precision(&*fixer, prec);
            fixer.perform()?;
            shape(&fixer.shape())
        }
        7 | 8 => {
            let fixer = ShapeFixFixSmallSolid::new();
            fixer.set_volume_threshold(maxtol);
            let result = if op == 7 {
                fixer.remove(&s, &ctx)?
            } else {
                fixer.merge(&s, &ctx)?
            };
            shape(&result)
        }
        9 => shells(&s),
        13 => small_faces(&s, prec),
        14 | 15 => strips(&s, prec, op == 15),
        16 => fix_shape_checks::small_face_checks(&s, prec),
        17 | 18 => fix_shape_checks::free_properties(&s, maxtol, op == 18),
        32 => fix_shape_checks::split_tool(&s, prec),
        33 => fix_shape_wires::intersecting_wires(&s, prec, maxtol),
        19.. => fix_shape_checks::wire_fixes(&s, prec, maxtol, op),
        _ => solids(&s),
    }
}

fn null_context() -> Failure {
    Failure::with_type("Standard_NullObject", "probe: null context", "")
}

/// `sh_fix_shape family (input) prec maxtol op`.
pub fn fix_shape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut s = Shape::default();
    attempt(|| {
        s = input(&mut r)?;
        Ok(())
    });
    let (prec, maxtol) = (r.real(), r.real());
    let op = r.integer();
    if !s.is_null() {
        attempt(|| run(s.clone(), prec, maxtol, op));
    }
    end();
}
