//! Shape-level ShapeFix probe operation, the Rust twin of
//! tools/oracle/tkshhealing_probe_fix_shape.cxx (which documents the requests).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere,
    BRepPrimAPIMakeTorus,
};
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{
    FixRoot, ShapeFix, ShapeFixFixSmallFace, ShapeFixFixSmallSolid, ShapeFixShape, ShapeFixShell,
    ShapeFixSolid, ShapeFixWireframe,
};

const STATUSES: [ShapeExtendStatus; 11] = [
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
        _ => solid(r),
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
