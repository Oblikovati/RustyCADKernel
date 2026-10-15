//! Translation of OCCT's GTests/AIS_ColoredShape_Test.cxx at 3d097a0. The native test
//! accessor exposes the protected `dispatchColors`; Martensite publishes it as
//! `ais_colored_shape_dispatch_colors` with the same parameters.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkv3d::ais::{
    AisColoredDrawer, AisColoredDrawerCompounds, AisColoredOpenedCompounds, AisShapeDrawerMap,
    ais_colored_shape_dispatch_colors,
};
use rustycad_tkv3d::prs3d::Prs3dDrawer;

/// `countDispatchedShapes`: the children of every compound of the map.
fn count_dispatched_shapes(map: &AisColoredDrawerCompounds) -> usize {
    (1..=map.len())
        .map(|index| {
            let compound = map.node_ref(index).unwrap().clone_compound().unwrap();
            let mut iterator = ShapeIterator::new(&compound, true, true);
            let mut count = 0;
            while iterator.more() {
                count += 1;
                iterator.next();
            }
            count
        })
        .sum()
}

fn make_box(dx: f64, dy: f64, dz: f64) -> Shape {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(dx, dy, dz).unwrap();
    maker.solid().unwrap().into_shape()
}

fn faces_of(shape: &Shape) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut faces = Vec::new();
    while explorer.more() {
        faces.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    faces
}

/// The `AIS_ColoredShapeDispatchTest` fixture.
struct DispatchFixture {
    box_shape: Shape,
    faces: Vec<Shape>,
    default_drawer: Handle<Prs3dDrawer>,
}

impl DispatchFixture {
    fn set_up() -> Self {
        let box_shape = make_box(10.0, 20.0, 30.0);
        assert!(!box_shape.is_null());
        let faces = faces_of(&box_shape);
        assert_eq!(faces.len(), 6);
        Self { box_shape, faces, default_drawer: Handle::new(Prs3dDrawer::new()) }
    }

    fn drawer(&self) -> Handle<AisColoredDrawer> {
        Handle::new(AisColoredDrawer::new(&self.default_drawer))
    }

    /// Dispatches the shell child of the box as natively (parent type solid, not closed).
    fn dispatch_shell(&self, styles: &AisShapeDrawerMap, opened: &AisColoredOpenedCompounds, closed: &AisColoredDrawerCompounds) -> bool {
        let mut iterator = ShapeIterator::new(&self.box_shape, true, true);
        while iterator.more() {
            let child = iterator.value().unwrap().clone();
            if child.shape_type().unwrap() == ShapeEnum::Shell {
                return ais_colored_shape_dispatch_colors(&Handle::default(), &child, styles, ShapeEnum::Solid, false, opened, closed).unwrap();
            }
            iterator.next();
        }
        panic!("the box has no shell child");
    }
}

fn opened_maps() -> AisColoredOpenedCompounds {
    std::array::from_fn(|_| AisColoredDrawerCompounds::default())
}

const FACE: usize = ShapeEnum::Face as usize;

#[test]
fn shell_no_overrides() {
    let fixture = DispatchFixture::set_up();
    let styles = AisShapeDrawerMap::default();
    let (opened, closed) = (opened_maps(), AisColoredDrawerCompounds::default());
    assert!(!fixture.dispatch_shell(&styles, &opened, &closed));
    assert_eq!(count_dispatched_shapes(&opened[FACE]), 6);
    assert_eq!(count_dispatched_shapes(&closed), 0);
}

#[test]
fn shell_one_face_overridden() {
    let fixture = DispatchFixture::set_up();
    let styles = AisShapeDrawerMap::default();
    let custom = fixture.drawer();
    custom.set_own_color(Color::named(NamedColor::RED));
    styles.bind(&fixture.faces[0], &custom).unwrap();
    let (opened, closed) = (opened_maps(), AisColoredDrawerCompounds::default());
    assert!(fixture.dispatch_shell(&styles, &opened, &closed));
    assert_eq!(count_dispatched_shapes(&opened[FACE]), 6);
}

#[test]
fn shell_one_face_hidden() {
    let fixture = DispatchFixture::set_up();
    let styles = AisShapeDrawerMap::default();
    let hidden = fixture.drawer();
    hidden.set_hidden(true);
    styles.bind(&fixture.faces[0], &hidden).unwrap();
    let (opened, closed) = (opened_maps(), AisColoredDrawerCompounds::default());
    assert!(fixture.dispatch_shell(&styles, &opened, &closed));
    assert_eq!(count_dispatched_shapes(&opened[FACE]), 5);
}

#[test]
fn shell_all_faces_overridden() {
    let fixture = DispatchFixture::set_up();
    let styles = AisShapeDrawerMap::default();
    for face in &fixture.faces {
        let drawer = fixture.drawer();
        drawer.set_own_color(Color::named(NamedColor::GREEN));
        styles.bind(face, &drawer).unwrap();
    }
    let (opened, closed) = (opened_maps(), AisColoredDrawerCompounds::default());
    assert!(fixture.dispatch_shell(&styles, &opened, &closed));
    assert_eq!(count_dispatched_shapes(&opened[FACE]), 6);
}

#[test]
fn compound_partial_override() {
    let fixture = DispatchFixture::set_up();
    let second_box = make_box(5.0, 5.0, 5.0);
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &fixture.box_shape).unwrap();
    builder.add(&mut compound, &second_box).unwrap();
    let styles = AisShapeDrawerMap::default();
    let custom = fixture.drawer();
    custom.set_own_color(Color::named(NamedColor::BLUE));
    styles.bind(&fixture.faces[0], &custom).unwrap();
    let (opened, closed) = (opened_maps(), AisColoredDrawerCompounds::default());
    let overridden = ais_colored_shape_dispatch_colors(&Handle::default(), &compound, &styles, ShapeEnum::Compound, false, &opened, &closed).unwrap();
    assert!(overridden);
    assert_eq!(count_dispatched_shapes(&opened[FACE]), 12);
}

#[test]
fn shell_all_faces_hidden() {
    let fixture = DispatchFixture::set_up();
    let styles = AisShapeDrawerMap::default();
    for face in &fixture.faces {
        let drawer = fixture.drawer();
        drawer.set_hidden(true);
        styles.bind(face, &drawer).unwrap();
    }
    let (opened, closed) = (opened_maps(), AisColoredDrawerCompounds::default());
    assert!(fixture.dispatch_shell(&styles, &opened, &closed));
    assert_eq!(count_dispatched_shapes(&opened[FACE]), 0);
    assert_eq!(count_dispatched_shapes(&closed), 0);
}
