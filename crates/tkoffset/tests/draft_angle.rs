//! Independent checks of BRepOffsetAPI_DraftAngle: a drafted box face and a drafted
//! cylinder have the closed-form volumes of the prism and the frustum they become.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIDraftAngle;
use rustycad_tkoffset::draft::DraftErrorStatus;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

fn volume(s: &Shape) -> Result<f64, Failure> {
    let mut props = GProps::new();
    BRepGProp::volume_properties(s, &mut props, false, false, false)?;
    Ok(props.mass())
}

fn face(s: &Shape, index: usize) -> Result<topods::Face, Failure> {
    let f = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?
        .nth(index)
        .expect("face");
    topods::face(&f)
}

fn bottom_plane() -> Result<Plane3, Failure> {
    Plane3::new(Point3::new(0., 0., 0.), Direction3::new(0., 0., 1.)?)
}

/// The face x = 0 of the box 2 x 3 x 4 drafted by `angle` about z = 0 turns about its
/// bottom edge: the box gains or loses the wedge 4 * (4 tan a) / 2 * 3.
#[test]
fn drafted_box_face_adds_the_wedge_volume() -> Result<(), Failure> {
    let a_box = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)?.shape()?.clone();
    let angle = 0.1;
    let mut draft = BRepOffsetAPIDraftAngle::with_shape(&a_box)?;
    draft.add(
        &face(&a_box, 0)?,
        Direction3::new(0., 0., 1.)?,
        angle,
        &bottom_plane()?,
        true,
    )?;
    assert!(draft.add_done()?);
    draft.build(&ProgressRange::default())?;
    assert!(draft.is_done());
    let result = draft.shape()?.clone();
    assert!(BRepCheckAnalyzer::new(&result, true, false, false)?.is_valid()?);
    let wedge = 0.5 * 4. * 4. * angle.tan() * 3.;
    let v = volume(&result)?;
    assert!(
        ((v - 24.).abs() - wedge).abs() < 1.0e-9,
        "volume {v}, wedge {wedge}"
    );
    assert_eq!(draft.modified_faces()?.len(), 1);
    Ok(())
}

/// The lateral face of the cylinder of radius 1.5 and height 3 drafted about its bottom
/// becomes the cone of the frustum with radii 1.5 and 1.5 +- 3 tan a.
#[test]
fn drafted_cylinder_becomes_the_frustum() -> Result<(), Failure> {
    let cylinder = BRepPrimAPIMakeCylinder::new(1.5, 3.).shape()?.clone();
    let angle = 0.1;
    let mut draft = BRepOffsetAPIDraftAngle::with_shape(&cylinder)?;
    draft.add(
        &face(&cylinder, 0)?,
        Direction3::new(0., 0., 1.)?,
        angle,
        &bottom_plane()?,
        true,
    )?;
    draft.build(&ProgressRange::default())?;
    assert!(draft.is_done());
    let v = volume(&draft.shape()?.clone())?;
    let frustum = |r2: f64| PI * 3. / 3. * (1.5 * 1.5 + 1.5 * r2 + r2 * r2);
    let (wider, narrower) = (
        frustum(1.5 + 3. * angle.tan()),
        frustum(1.5 - 3. * angle.tan()),
    );
    assert!(
        (v - wider).abs() < 1.0e-6 * wider || (v - narrower).abs() < 1.0e-6 * narrower,
        "volume {v}, frustums {wider} {narrower}"
    );
    Ok(())
}

/// A sphere cannot be drafted: the face is the problematic shape.
#[test]
fn sphere_draft_reports_the_face() -> Result<(), Failure> {
    let sphere = BRepPrimAPIMakeSphere::new(2.)?.shape()?.clone();
    let mut draft = BRepOffsetAPIDraftAngle::with_shape(&sphere)?;
    let f = face(&sphere, 0)?;
    draft.add(
        &f,
        Direction3::new(0., 0., 1.)?,
        0.1,
        &bottom_plane()?,
        true,
    )?;
    assert!(!draft.add_done()?);
    assert_eq!(draft.status()?, DraftErrorStatus::FaceRecomputation);
    assert!(draft.problematic_shape()?.is_same(&f));
    Ok(())
}
