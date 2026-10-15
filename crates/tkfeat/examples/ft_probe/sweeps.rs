//! The BRepFeat sweep probes (`ft_prism`, `ft_dprism`, `ft_revol`, `ft_pipe`); the twin of
//! tools/oracle/tkfeat_probe_sweeps.cxx.
use super::common::{Reader, attempt, end, flag, int};
use super::shapes::{basis, face_at, polygon_face, rect, shape};
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkfeat::brep_feat::{
    BRepFeatMakeDPrism, BRepFeatMakePipe, BRepFeatMakePrism, BRepFeatMakeRevol, FormFeature,
};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};

/// `ft_feature`: done, the status error, then the shape when done.
fn feature<F: FormFeature>(f: &mut F) -> Result<(), Failure> {
    flag(f.is_done());
    int(f.form().current_status_error() as i32);
    if f.is_done() {
        let s = f.shape()?.clone();
        shape(&s)?;
    }
    Ok(())
}

/// `ft_limit`: the square [-5, 15]^2 at the height z.
fn limit(z: f64) -> Result<Shape, Failure> {
    Ok(rect(-5., -5., 15., 15., z)?.into_shape())
}

/// The limit methods of the prisms (see `ft_limits`).
macro_rules! limits {
    ($f:expr, $b:expr, $method:expr, $a:expr, $bb:expr, $length:ident) => {
        match $method {
            0 => $f.$length($a),
            1 => $f.perform_until(&limit($a)?),
            2 => $f.perform_from_until(&limit($a)?, &limit($bb)?),
            3 => $f.perform_until_end(),
            4 => $f.perform_thru_all(),
            5 => $f.perform_until_height(&limit($a)?, $bb),
            6 => $f.perform_from_end(&limit($a)?),
            _ => $f.perform_until(&face_at($b, 0.)?.into_shape()),
        }
    };
}

/// The fields of a prism request after the basis and the mode.
struct Limits {
    method: i32,
    a: f64,
    b: f64,
}

impl Limits {
    fn read(r: &mut Reader) -> Self {
        let method = r.integer();
        let (a, b) = (r.real(), r.real());
        Self { method, a, b }
    }
}

/// `ft_profile`: the rectangle [2, 6] x [3, 5] on the top of the basis.
fn profile() -> Result<Face, Failure> {
    rect(2., 3., 6., 5., 10.)
}

/// `ft_prism basis mode up method a b`.
pub fn prism_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, mode, up) = (r.integer(), r.integer(), r.flag());
    let l = Limits::read(&mut r);
    attempt(|| {
        let b = basis(kind)?;
        let dir = Direction3::new(0., 0., if up { 1. } else { -1. })?;
        let mut f = BRepFeatMakePrism::with_shapes(
            &b,
            &profile()?.into_shape(),
            &face_at(&b, 10.)?,
            dir,
            mode,
            true,
        )?;
        limits!(f, &b, l.method, l.a, l.b, perform_length)?;
        feature(&mut f)
    });
    end();
}

/// `ft_dprism basis mode angle method a b`.
pub fn dprism_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, mode, ang) = (r.integer(), r.integer(), r.real());
    let l = Limits::read(&mut r);
    attempt(|| {
        let b = basis(kind)?;
        let mut f =
            BRepFeatMakeDPrism::with_shapes(&b, &profile()?, &face_at(&b, 10.)?, ang, mode, true)?;
        limits!(f, &b, l.method, l.a, l.b, perform_height)?;
        feature(&mut f)
    });
    end();
}

/// `ft_wall`: the half planes y = 5 beyond x = 5 (0) and before it (1).
fn wall(side: i32) -> Result<Shape, Failure> {
    let (x0, x1) = if side != 0 { (-5., 5.) } else { (5., 15.) };
    Ok(polygon_face(&[(x0, 5., -5.), (x1, 5., -5.), (x1, 5., 15.), (x0, 5., 15.)])?.into_shape())
}

/// `ft_revol mode method a`.
pub fn revol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, method, a) = (r.integer(), r.integer(), r.real());
    attempt(|| {
        let b = basis(0)?;
        let (z0, z1) = if mode != 0 { (10., 12.) } else { (8., 10.) };
        let prof = polygon_face(&[(5., 2., z0), (5., 4., z0), (5., 4., z1), (5., 2., z1)])?;
        let axis = Axis1::new(Point3::new(5., 5., 0.), Direction3::new(0., 0., 1.)?);
        let mut f = BRepFeatMakeRevol::with_shapes(
            &b,
            &prof.into_shape(),
            &face_at(&b, 10.)?,
            axis,
            mode,
            true,
        )?;
        match method {
            0 => f.perform_angle(a)?,
            1 => f.perform_until(&wall(0)?)?,
            2 => f.perform_thru_all()?,
            3 => f.perform_until_angle(&wall(0)?, a)?,
            _ => f.perform_from_until(&wall(1)?, &wall(0)?)?,
        }
        feature(&mut f)
    });
    end();
}

/// `ft_pipe mode z1 method a`.
pub fn pipe_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, z1, method, a) = (r.integer(), r.real(), r.integer(), r.real());
    attempt(|| {
        let b = basis(0)?;
        let top = face_at(&b, 10.)?;
        let e =
            BRepBuilderAPIMakeEdge::from_points(Point3::new(3., 3., 10.), Point3::new(3., 3., z1))?
                .edge()?;
        let spine = BRepBuilderAPIMakeWire::from_edge(&e)?.wire()?;
        let prof = rect(2., 2., 4., 4., 10.)?.into_shape();
        let mut f = BRepFeatMakePipe::with_shapes(&b, &prof, &top, &spine, mode, true)?;
        match method {
            0 => f.perform()?,
            1 => f.perform_until(&limit(a)?)?,
            _ => f.perform_from_until(&top.clone().into_shape(), &limit(a)?)?,
        }
        feature(&mut f)
    });
    end();
}
