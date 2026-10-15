//! BiTgte_Blend probe operations, the Rust twin of tools/oracle/tkoffset_probe_bitgte.cxx
//! (the records are documented there).
use super::common::{Reader, end, flag, int};
use super::make_offset::failed;
use super::maker::{digest, round6};
use super::tool::{primitive, sub};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkoffset::bi_tgte::BiTgteBlend;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;
use std::f64::consts::PI;

/// The shape of a request (`obt_shape`).
fn shape_of(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    if r.integer() == 0 {
        return primitive(r);
    }
    let s1: Handle<dyn GeomSurface> = Handle::new(GeomCylindricalSurface::new(
        Axis3::new(
            Point3::new(0., 360., 0.),
            Direction3::new(0., 0., 1.)?,
            Direction3::new(0., 1., 0.)?,
        )?,
        160.,
    )?)
    .cast();
    let s2: Handle<dyn GeomSurface> = Handle::new(GeomCylindricalSurface::new(
        Axis3::from_normal(Point3::new(0., 0., 0.), Direction3::new(0., 1., 0.)?)?,
        80.,
    )?)
    .cast();
    let f1 =
        BRepLibMakeFace::from_surface_bounds(&s1, (0., 2. * PI, -113.2000012, 113.2000012), 1e-7)?
            .face()?;
    let f2 = BRepLibMakeFace::from_surface_bounds(&s2, (0., 2. * PI, 168.4, 600.), 1e-7)?.face()?;
    let b = BRepBuilder::default();
    let mut c = Compound::default();
    b.make_compound(&mut c);
    b.add(&mut c, &f1)?;
    b.add(&mut c, &f2)?;
    Ok(c.into_shape())
}

/// The sorted records of the blend surfaces (`obt_surfaces`).
fn surfaces(blend: &BiTgteBlend) -> Result<(), Failure> {
    let mut records = Vec::new();
    for i in 1..=blend.nb_surfaces() {
        let f = blend.face(i)?;
        let mut p = GProps::new();
        BRepGProp::surface_properties(&f, &mut p, false, false)?;
        let a = BRepAdaptorSurface::with_face(&f, false)?;
        let mut em = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&f, ShapeEnum::Edge, &mut em)?;
        records.push(vec![
            blend.contact_type(i).map_or(-1, |t| t as i64),
            a.get_type()? as i64,
            round6(p.mass()),
            em.extent() as i64,
        ]);
    }
    records.sort();
    for rec in &records {
        for v in rec {
            print!("{v} ");
        }
    }
    Ok(())
}

/// `obt_blend shape radius tol build npairs (f1 f2)... nedges (e)... nstops (f)...`.
pub fn blend_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if let Err(error) = blend_body(&mut r) {
        failed(error);
        return;
    }
    end();
}

fn blend_body(r: &mut Reader<'_>) -> Result<(), Failure> {
    let s = shape_of(r)?;
    let (radius, tol, build) = (r.real(), r.real(), r.flag());
    let mut blend = BiTgteBlend::new();
    blend.init(&s, radius, tol, false);
    for _ in 0..r.integer() {
        let f1 = topods::face(&sub(&s, ShapeEnum::Face, r.integer())?)?;
        let f2 = topods::face(&sub(&s, ShapeEnum::Face, r.integer())?)?;
        blend.set_faces(&f1, &f2);
    }
    for _ in 0..r.integer() {
        blend.set_edge(&topods::edge(&sub(&s, ShapeEnum::Edge, r.integer())?)?);
    }
    for _ in 0..r.integer() {
        blend.set_stopping_face(&topods::face(&sub(&s, ShapeEnum::Face, r.integer())?)?);
    }
    blend.perform(build)?;
    flag(blend.is_done());
    int(blend.nb_surfaces());
    surfaces(&blend)?;
    let nb = blend.nb_branches()?;
    int(nb);
    let mut sizes = Vec::new();
    for i in 1..=nb {
        let (from, to) = blend.indices_of_branche(i)?;
        sizes.push(to - from + 1);
    }
    sizes.sort();
    for v in sizes {
        int(v);
    }
    digest(blend.shape())
}
