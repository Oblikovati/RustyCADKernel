//! Native face subdivision tools, results and aggregate status.
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{Face, Shape},
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    geom::{GeomCylindricalSurface, GeomSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis3, Plane3};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{
        FaceDivider, ShapeUpgradeFaceDivide, ShapeUpgradeSplitSurface,
        ShapeUpgradeSplitSurfaceAngle, SplitSurface, SplitSurfaceData,
    },
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

fn fixture_face(mode: usize) -> Face {
    if mode == 6 {
        return Face::default();
    }
    if mode == 5 {
        let surface: Handle<dyn GeomSurface> =
            Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast();
        return BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0., 3., 0., 2.), 1e-7)
            .unwrap()
            .face()
            .unwrap();
    }
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., 2., 0., 2.))
        .unwrap()
        .face()
        .unwrap()
}

fn configured_divider(face: &Face, mode: usize) -> ShapeUpgradeFaceDivide {
    let divider = ShapeUpgradeFaceDivide::new(face.clone());
    if mode == 1 || mode == 3 {
        divider.set_wire_divide_tool(&Handle::default());
    }
    if mode == 2 || mode == 3 {
        divider.set_split_surface_tool(&Handle::default());
    }
    if mode == 4 {
        divider.set_surface_segment_mode(false);
    }
    if mode == 5 {
        divider
            .set_split_surface_tool(&Handle::new(ShapeUpgradeSplitSurfaceAngle::new(1.5)).cast());
    }
    if mode == 7 {
        divider.set_split_surface_tool(
            &Handle::new(MarkModifiedSurface {
                parent: ShapeUpgradeSplitSurface::new(),
            })
            .cast(),
        );
    }
    if mode == 8 {
        divider.set_context(&Handle::new(
            rustycad_tkshhealing::shape_build::ShapeBuildReShape::new(),
        ));
        divider
            .context()
            .remove(&face.clone().into_shape())
            .unwrap();
    }
    divider
}

fn count_faces(shape: &Shape) -> i32 {
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}

fn observe(mode: usize) -> Vec<i32> {
    let face = fixture_face(mode);
    let divider = configured_divider(&face, mode);
    let dynamic: &dyn FaceDivider = &divider;
    let changed = dynamic.perform(0.).unwrap();
    let result = divider.result();
    let mut row = vec![
        i32::from(changed),
        i32::from(result.is_null()),
        i32::from(result.is_same(&face.clone().into_shape())),
        count_faces(&result),
        i32::from(divider.context().is_null()),
    ];
    for flag in [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
    ] {
        row.push(i32::from(divider.status_flag(flag)));
    }
    row.push(recorded_vertices(&divider, &face));
    row
}

#[test]
fn configured_face_subdivision_matches_native_results_and_flags() {
    for (mode, line) in include_str!("../../../tools/oracle/tkshhealing_face_divide.expected")
        .lines()
        .enumerate()
    {
        let expected: Vec<i32> = line
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(observe(mode), expected, "native case {line}");
    }
}

struct MarkModifiedSurface {
    parent: ShapeUpgradeSplitSurface,
}
impl_transient!(MarkModifiedSurface, "MarkModifiedSurface", ShapeUpgradeSplitSurface, parent, interfaces:[dyn SplitSurface]);
impl SplitSurface for MarkModifiedSurface {
    fn split_surface_data(&self) -> &SplitSurfaceData {
        self.parent.split_surface_data()
    }
    fn compute(&self, _segment: bool) -> Result<(), Failure> {
        self.split_surface_data().with_mut(|fields| {
            fields.status = rustycad_tkshhealing::shape_extend::ShapeExtend::encode_status(
                ShapeExtendStatus::Done3,
            )
        });
        Ok(())
    }
}
fn recorded_vertices(divider: &ShapeUpgradeFaceDivide, face: &Face) -> i32 {
    if divider.context().is_null() {
        return 0;
    }
    let mut explorer = Explorer::new(face, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += i32::from(divider.context().is_recorded(explorer.current()));
        explorer.next().unwrap();
    }
    count
}
