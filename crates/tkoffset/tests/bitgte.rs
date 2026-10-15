//! Independent checks of BiTgte_Blend: the rolling ball inside a box gives the quarter
//! cylinders and sphere octants of the closed-form fillet, and the OCC29861 blend of two
//! cylindrical faces has the counts and area of its DRAW test.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkoffset::bi_tgte::BiTgteBlend;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;
use std::f64::consts::PI;

fn area(s: &Shape) -> Result<f64, Failure> {
    let mut props = GProps::new();
    BRepGProp::surface_properties(s, &mut props, false, false)?;
    Ok(props.mass())
}

fn count(s: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, kind, &mut m)?;
    Ok(m.extent())
}

/// The ball of radius 0.4 rolling inside the box 2 x 3 x 4 between two adjacent faces
/// spreads to every edge: 12 quarter cylinders of length `L - 2r` and 8 sphere octants.
#[test]
fn ball_inside_a_box_gives_the_closed_form_fillet() -> Result<(), Failure> {
    let a_box = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)?.shape()?.clone();
    let faces: Vec<Shape> = Explorer::new(&a_box, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    let r = 0.4;
    let mut blend = BiTgteBlend::with_shape(&a_box, -r, 1.0e-4, false);
    blend.set_faces(&topods::face(&faces[0])?, &topods::face(&faces[2])?);
    blend.perform(false)?;
    assert!(blend.is_done());
    assert_eq!(blend.nb_surfaces(), 20);
    let mut areas: Vec<f64> = (1..=20)
        .map(|i| area(&blend.face(i).unwrap().into_shape()))
        .collect::<Result<_, _>>()?;
    areas.sort_by(f64::total_cmp);
    let octant = PI * r * r / 2.;
    let mut expected = vec![octant; 8];
    for length in [2., 3., 4.] {
        expected.extend([PI / 2. * r * (length - 2. * r); 4]);
    }
    expected.sort_by(f64::total_cmp);
    for (a, e) in areas.iter().zip(&expected) {
        assert!((a - e).abs() < 1.0e-6, "area {a}, expected {e}");
    }
    assert_eq!(blend.nb_branches()?, 1);
    assert_eq!(blend.indices_of_branche(1)?, (1, 20));
    Ok(())
}

/// OCC29861: `brollingball result s 30 @ f1 f2` gives a shell of 6 faces, 7 wires, 17
/// edges and 10 vertices of area 246256 (tests/bugs/modalg_7/bug29861).
#[test]
fn occ29861_blend_of_two_cylinders() -> Result<(), Failure> {
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

    let mut blend = BiTgteBlend::with_shape(&c, 30., 1.0e-4, false);
    blend.set_faces(&f1, &f2);
    blend.perform(true)?;
    assert!(blend.is_done());
    let result = blend.shape().clone();
    assert_eq!(count(&result, ShapeEnum::Vertex)?, 10);
    assert_eq!(count(&result, ShapeEnum::Edge)?, 17);
    assert_eq!(count(&result, ShapeEnum::Wire)?, 7);
    assert_eq!(count(&result, ShapeEnum::Face)?, 6);
    assert_eq!(count(&result, ShapeEnum::Shell)?, 1);
    let a = area(&result)?;
    assert!((a - 246256.).abs() < 1.0e-4 * 246256., "area {a}");
    Ok(())
}
