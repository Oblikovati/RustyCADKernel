//! BRepLib_MakeFace, BRepLib_FindSurface, BRepLib::SameParameter and BRepLib::ExtendFace
//! probe operations and the shape records 5 to 8, the Rust twin of
//! tools/oracle/tktopalgo_probe_breplib_face.cxx (the request layout is documented there).
use super::brep_lib_common::{geom_curve, geom2d_curve, shape, type_name};
use super::brep_lib_package::{guarded, sub_shapes};
use super::brep_lib_shapes::shape_record;
use super::common::{Reader, attempt, flag, int, pnt, real};
use super::face::surface;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomOffsetSurface, GeomPlane, GeomSphericalSurface,
    GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Axis1;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::{self, BRepLibFindSurface, BRepLibMakeFace, BRepLibMakePolygon};

/// `tp_surface2`.
pub fn surface2(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(match r.integer() {
        0 => return surface(r),
        1 => {
            let v = r.real();
            Handle::new(GeomOffsetSurface::new(&surface2(r)?, v, false)?).cast()
        }
        2 => {
            let d = r.dir();
            Handle::new(GeomSurfaceOfLinearExtrusion::new(&geom_curve(r)?, d)?).cast()
        }
        _ => {
            let (p, d) = (r.pnt(), r.dir());
            Handle::new(GeomSurfaceOfRevolution::new(
                &geom_curve(r)?,
                Axis1::new(p, d),
            )?)
            .cast()
        }
    })
}

/// `tp_shape_record_more`: the shape records 5 to 9.
pub fn shape_record_more(r: &mut Reader<'_>, kind: i32) -> Result<Shape, Failure> {
    match kind {
        5 => polygon_record(r),
        6 => face_wire_record(r),
        7 => edge_on_surface_record(r),
        9 => made_face_record(r),
        10.. => super::brep_lib_fuse::shape_record_fuse(r, kind),
        _ => unflagged_record(r),
    }
}

/// Record 5.
fn polygon_record(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let n = r.integer();
    let mut m = BRepLibMakePolygon::new();
    for _ in 0..n {
        m.add_point(r.pnt())?;
    }
    if r.flag() {
        m.close()?;
    }
    if !m.is_done() {
        return Ok(Shape::default());
    }
    Ok(m.wire()?.into_shape())
}

/// Record 9.
fn made_face_record(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let s = surface2(r)?;
    let b = bounds(r);
    let m = BRepLibMakeFace::from_surface_bounds(&s, b, r.real())?;
    if !m.is_done() {
        return Ok(Shape::default());
    }
    Ok(m.face()?.into_shape())
}

/// Record 6.
fn face_wire_record(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let s = surface2(r)?;
    let bounds = (r.real(), r.real(), r.real(), r.real());
    let reversed = r.flag();
    let m = BRepLibMakeFace::from_surface_bounds(&s, bounds, precision::confusion())?;
    let face = m.face()?;
    let it = ShapeIterator::new(&face, true, true);
    let Ok(wire) = it.value() else {
        return Ok(Shape::default());
    };
    Ok(if reversed {
        wire.reversed()
    } else {
        wire.clone()
    })
}

/// Record 7.
fn edge_on_surface_record(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let c = geom_curve(r)?;
    let (u1, u2) = (r.real(), r.real());
    let pc = geom2d_curve(r)?;
    let (p1, p2) = (r.real(), r.real());
    let s = surface(r)?;
    let tol = r.real();
    let (sp, sr) = (r.flag(), r.flag());
    let mut e = Edge::default();
    builder.make_edge_with_curve(&mut e, &c, tol)?;
    builder.update_edge_pcurve(&e, &pc, &s, &Location::default(), tol, None)?;
    builder.range(&e, u1, u2, true)?;
    builder.range_on_surface(&e, &s, &Location::default(), p1, p2)?;
    let curve = c.get().expect("a curve");
    let (mut v1, mut v2) = (Vertex::default(), Vertex::default());
    builder.make_vertex_at(&mut v1, curve.value(u1)?, tol)?;
    builder.make_vertex_at(&mut v2, curve.value(u2)?, tol)?;
    builder.add(&mut e, &v1.oriented(Orientation::Forward))?;
    builder.add(&mut e, &v2.oriented(Orientation::Reversed))?;
    builder.same_parameter(&e, sp)?;
    builder.same_range(&e, sr)?;
    Ok(e.into_shape())
}

/// Record 8.
fn unflagged_record(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let sh = shape_record(r)?;
    let (build, unflag) = (r.flag(), r.flag());
    if build {
        brep_lib::build_curves3d_default(&sh)?;
    }
    if unflag {
        for e in Explorer::new(&sh, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let e = Edge::unchecked(e);
            builder.same_parameter(&e, false)?;
            builder.same_range(&e, false)?;
        }
    }
    Ok(sh)
}

/// The wire of a shape record (Standard_TypeMismatch otherwise).
fn wire_record(r: &mut Reader<'_>) -> Result<Wire, Failure> {
    topods::wire(&shape_record(r)?)
}

/// `tp_gp_face`: None for a surface that is not elementary.
fn gp_face(
    s: &Handle<dyn GeomSurface>,
    mode: i32,
    r: &mut Reader<'_>,
) -> Result<Option<BRepLibMakeFace>, Failure> {
    let mut bounds = (0., 0., 0., 0.);
    let mut wire = Wire::default();
    let mut inside = true;
    if mode == 1 {
        bounds = (r.real(), r.real(), r.real(), r.real());
    }
    if mode == 4 {
        wire = wire_record(r)?;
        inside = r.flag();
    }
    macro_rules! make {
        ($gp:expr, $plain:ident, $bounds:ident, $wire:ident) => {
            Some(match mode {
                0 => BRepLibMakeFace::$plain($gp)?,
                1 => BRepLibMakeFace::$bounds($gp, bounds)?,
                _ => BRepLibMakeFace::$wire($gp, &wire, inside)?,
            })
        };
    }
    Ok(if let Some(p) = s.cast::<GeomPlane>().get() {
        make!(p.pln(), from_plane, from_plane_bounds, from_plane_wire)
    } else if let Some(c) = s.cast::<GeomCylindricalSurface>().get() {
        make!(
            c.cylinder()?,
            from_cylinder,
            from_cylinder_bounds,
            from_cylinder_wire
        )
    } else if let Some(k) = s.cast::<GeomConicalSurface>().get() {
        make!(k.cone()?, from_cone, from_cone_bounds, from_cone_wire)
    } else if let Some(sp) = s.cast::<GeomSphericalSurface>().get() {
        make!(
            sp.sphere()?,
            from_sphere,
            from_sphere_bounds,
            from_sphere_wire
        )
    } else if let Some(t) = s.cast::<GeomToroidalSurface>().get() {
        make!(t.torus()?, from_torus, from_torus_bounds, from_torus_wire)
    } else {
        None
    })
}

/// `tp_face_maker`.
fn face_maker(m: &BRepLibMakeFace) {
    flag(m.is_done());
    int(m.error() as i32);
    attempt(|| shape(&m.face()?.into_shape()));
}

/// The four bounds of a request.
fn bounds(r: &mut Reader<'_>) -> (f64, f64, f64, f64) {
    (r.real(), r.real(), r.real(), r.real())
}

/// The maker of `blm_face`, None for a gp mode on another surface.
fn face_request(r: &mut Reader<'_>) -> Result<Option<BRepLibMakeFace>, Failure> {
    let mode = r.integer();
    Ok(Some(match mode {
        0 | 1 | 4 => {
            let s = surface2(r)?;
            return gp_face(&s, mode, r);
        }
        2 => BRepLibMakeFace::from_surface(&surface2(r)?, r.real())?,
        3 | 10 => {
            let s = surface2(r)?;
            let b = bounds(r);
            let mut m = BRepLibMakeFace::from_surface_bounds(&s, b, r.real())?;
            if mode == 10 {
                face_maker(&m);
                let s2 = surface2(r)?;
                let b2 = bounds(r);
                m.init_surface_bounds(&s2, b2, r.real())?;
            }
            m
        }
        5 => {
            let s = surface2(r)?;
            let w = wire_record(r)?;
            BRepLibMakeFace::from_surface_wire(&s, &w, r.flag())?
        }
        6 => {
            let w = wire_record(r)?;
            BRepLibMakeFace::from_wire(&w, r.flag())?
        }
        7 => BRepLibMakeFace::from_face(&topods::face(&shape_record(r)?)?)?,
        8 => {
            let f: Face = topods::face(&shape_record(r)?)?;
            BRepLibMakeFace::from_face_wire(&f, &wire_record(r)?)?
        }
        _ => {
            let s = surface2(r)?;
            let bound = r.flag();
            let mut m = BRepLibMakeFace::new();
            m.init_surface(&s, bound, r.real())?;
            for _ in 0..r.integer() {
                m.add(&wire_record(r)?)?;
            }
            m
        }
    }))
}

/// `blm_face`.
pub fn face_case(values: &[f64]) {
    guarded(values, |r| {
        match face_request(r)? {
            Some(m) => face_maker(&m),
            None => int(-2),
        }
        Ok(())
    });
}

/// `blm_facedegen`.
pub fn facedegen_case(values: &[f64]) {
    guarded(values, |r| {
        let c = geom_curve(r)?;
        let (degenerated, tol) = BRepLibMakeFace::is_degenerated(&c, r.real())?;
        flag(degenerated);
        real(tol);
        Ok(())
    });
}

/// `blp_findsurface`.
pub fn findsurface_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let tol = r.real();
        let (plane, closed) = (r.flag(), r.flag());
        let fs = BRepLibFindSurface::new(&s, tol, plane, closed)?;
        flag(fs.found());
        flag(fs.existed());
        real(fs.tolerance());
        real(fs.tolerance_reached());
        flag(fs.location().is_identity());
        let surf = fs.surface();
        type_name(&surf);
        if let Some(surf) = surf.get() {
            for (u, v) in [(0., 0.), (1., 0.), (0., 1.)] {
                attempt(|| {
                    pnt(surf.value(u, v)?);
                    Ok(())
                });
            }
        }
        Ok(())
    });
}

/// The per-edge modes 2 to 4 of `blp_sameparam`.
fn sameparam_edges(s: &Shape, mode: i32, tol: f64) -> Result<(), Failure> {
    let m = sub_shapes(s, ShapeEnum::Edge)?;
    for i in 1..=m.extent() {
        let e = Edge::unchecked(m.find_key(i)?.clone());
        if mode == 2 {
            attempt(|| brep_lib::same_parameter_edge(&e, tol));
            continue;
        }
        attempt(|| {
            let mut nt = -1.;
            let e2 = brep_lib::same_parameter_edge_with_tolerance(&e, tol, &mut nt, mode == 4)?;
            real(nt);
            if mode == 3 {
                shape(&e2.into_shape())?;
            }
            Ok(())
        });
    }
    Ok(())
}

/// `blp_sameparam`.
pub fn sameparam_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let (mode, tol, forced) = (r.integer(), r.real(), r.flag());
        match mode {
            0 => brep_lib::same_parameter_shape(&s, tol, forced)?,
            1 => {
                let rs = BRepToolsReShape::new();
                brep_lib::same_parameter_shape_with_reshape(&s, &rs, tol, forced)?;
                shape(&s)?;
                shape(&rs.apply(&s, ShapeEnum::Shape)?)?;
            }
            _ => sameparam_edges(&s, mode, tol)?,
        }
        shape(&s)
    });
}

/// `blp_extendface`.
pub fn extendface_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let ext = r.real();
        let d = [r.flag(), r.flag(), r.flag(), r.flag()];
        let m = sub_shapes(&s, ShapeEnum::Face)?;
        for i in 1..=m.extent() {
            let f = Face::unchecked(m.find_key(i)?.clone());
            attempt(|| shape(&brep_lib::extend_face(&f, ext, d)?.into_shape()));
        }
        Ok(())
    });
}
