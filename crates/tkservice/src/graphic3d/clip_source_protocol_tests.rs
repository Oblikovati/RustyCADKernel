//! Named native subclasses prove genuine embedded plane/collection virtual transport.
use super::*;
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, SharedState};
use std::io;

struct NativeClipProtocolPlane {
    base: Graphic3dClipPlane,
    clone_calls: SharedState<usize>,
    dump_calls: SharedState<usize>,
}
rustycad_tkernel::impl_transient!(NativeClipProtocolPlane,
    "Martensite_ClipProtocolPlane", Graphic3dClipPlane, base,
    interfaces: [dyn Graphic3dClipPlaneSource]);
impl Graphic3dClipPlaneSource for NativeClipProtocolPlane {
    fn clip_plane_base(&self) -> &Graphic3dClipPlane {
        &self.base
    }
    fn clone_plane(&self) -> Handle<dyn Graphic3dClipPlaneSource> {
        *self.clone_calls.write() += 1;
        graphic3d_clip_plane_clone(&self.base)
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        *self.dump_calls.write() += 1;
        self.base.dump_json(stream, depth)
    }
}
struct NativeClipProtocolSequence {
    base: Graphic3dSequenceOfHClipPlane,
    dump_calls: SharedState<usize>,
}
rustycad_tkernel::impl_transient!(NativeClipProtocolSequence,
    "Martensite_ClipProtocolSequence", Graphic3dSequenceOfHClipPlane, base,
    interfaces: [dyn Graphic3dClipPlaneSequenceSource]);
impl Graphic3dClipPlaneSequenceSource for NativeClipProtocolSequence {
    fn clip_sequence_base(&self) -> &Graphic3dSequenceOfHClipPlane {
        &self.base
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        *self.dump_calls.write() += 1;
        self.base.dump_json(stream, depth)
    }
}
fn protocol_plane() -> Handle<NativeClipProtocolPlane> {
    let base = Graphic3dClipPlane::from_equation(Vec4::new(1.0, 2.0, 3.0, 4.0)).unwrap();
    base.set_on(false).unwrap();
    base.set_capping(true);
    base.set_capping_color(Color::new(0.2, 0.4, 0.6, ColorSpace::Rgb).unwrap())
        .unwrap();
    Handle::new(NativeClipProtocolPlane {
        base,
        clone_calls: SharedState::new(0),
        dump_calls: SharedState::new(0),
    })
}
#[test]
fn derived_plane_clone_and_dump_dispatch_preserve_real_inherited_fields() {
    let concrete = protocol_plane();
    let source = concrete.observe_as::<dyn Graphic3dClipPlaneSource>();
    let cloned = source.clone_plane();
    source.dump_json(&mut DumpStream::default(), -1).unwrap();
    assert_eq!(*concrete.clone_calls.read(), 1);
    assert_eq!(*concrete.dump_calls.read(), 1);
    assert_eq!((source.ref_count(), cloned.ref_count()), (1, 1));
    assert_eq!(cloned.get_equation(), source.get_equation());
    assert_eq!(cloned.capping_color(), source.capping_color());
    assert!(!cloned.is_on() && cloned.is_capping());
    assert_ne!(cloned.capping_aspect(), source.capping_aspect());
    assert!(cloned.is_instance_name("Graphic3d_ClipPlane"));
}
#[test]
fn collection_source_uses_actual_storage_and_one_native_descriptor() {
    let concrete = Handle::new(NativeClipProtocolSequence {
        base: Graphic3dSequenceOfHClipPlane::default(),
        dump_calls: SharedState::new(0),
    });
    let source = concrete.observe_as::<dyn Graphic3dClipPlaneSequenceSource>();
    source.set_override_global(true);
    source.dump_json(&mut DumpStream::default(), -1).unwrap();
    assert!(concrete.base.to_override_global());
    assert_eq!(*concrete.dump_calls.read(), 1);
    assert_eq!(source.ref_count(), 1);
    assert_clip_source_descriptors();
}
fn assert_clip_source_descriptors() {
    assert!(std::ptr::eq(
        Graphic3dSequenceOfHClipPlane::static_type(),
        <dyn Graphic3dClipPlaneSequenceSource>::static_type()
    ));
    assert!(std::ptr::eq(
        Graphic3dClipPlane::static_type(),
        <dyn Graphic3dClipPlaneSource>::static_type()
    ));
}

fn plane_protocol_row(label: &str, plane: &Handle<dyn Graphic3dClipPlaneSource>) -> String {
    let equation = plane.get_equation();
    let color = plane.capping_color();
    format!(
        "{label} {:.1} {:.1} {:.1} {:.1} {} {} {:.1} {:.1} {:.1} {}\n",
        equation.x,
        equation.y,
        equation.z,
        equation.w,
        i32::from(plane.is_on()),
        i32::from(plane.is_capping()),
        color.red(),
        color.green(),
        color.blue(),
        plane.ref_count()
    )
}
fn sequence_protocol_row() -> String {
    let concrete = Handle::new(NativeClipProtocolSequence {
        base: Graphic3dSequenceOfHClipPlane::default(),
        dump_calls: SharedState::new(0),
    });
    let source = concrete.observe_as::<dyn Graphic3dClipPlaneSequenceSource>();
    source.set_override_global(true);
    source.dump_json(&mut DumpStream::default(), -1).unwrap();
    format!(
        "sequence {} {} {}\n",
        i32::from(source.to_override_global()),
        *concrete.dump_calls.read(),
        source.ref_count()
    )
}
fn material_protocol_row(label: &str, plane: &Handle<dyn Graphic3dClipPlaneSource>) -> String {
    let interior = plane.capping_aspect().interior_color();
    let diffuse = plane.capping_material().diffuse_color();
    format!(
        "{label} {:.1} {:.1} {:.1} {:.1} {:.1} {:.1}\n",
        interior.red(),
        interior.green(),
        interior.blue(),
        diffuse.red(),
        diffuse.green(),
        diffuse.blue()
    )
}
#[test]
fn native_plane_and_collection_protocol_rows_replay_without_occt() {
    let rows = capture_plane_protocol_rows() + &sequence_protocol_row();
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/clip-source-protocol-native.txt")
    );
}
fn capture_plane_protocol_rows() -> String {
    let concrete = protocol_plane();
    let source = concrete.observe_as::<dyn Graphic3dClipPlaneSource>();
    let cloned = source.clone_plane();
    source.dump_json(&mut DumpStream::default(), -1).unwrap();
    let mut rows = plane_protocol_row("original", &source);
    rows.push_str(&plane_protocol_row("clone", &cloned));
    rows.push_str(&material_protocol_row("original-material", &source));
    rows.push_str(&material_protocol_row("clone-material", &cloned));
    rows.push_str(&format!(
        "dispatch {} {} {}\n",
        *concrete.clone_calls.read(),
        *concrete.dump_calls.read(),
        i32::from(source.capping_aspect() != cloned.capping_aspect())
    ));
    rows
}

#[test]
fn derived_sequence_iterator_captures_actual_nodes_without_owners() {
    let concrete = Handle::new(NativeClipProtocolSequence {
        base: Graphic3dSequenceOfHClipPlane::default(),
        dump_calls: SharedState::new(0),
    });
    let plane = protocol_plane().into_cast::<dyn Graphic3dClipPlaneSource>();
    concrete.base.append_reference(&plane);
    let rows = replay_derived_iterator(&concrete, &plane);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/clip-derived-iterator-native.txt")
    );
    concrete.base.clear();
}
fn replay_derived_iterator(
    sequence: &Handle<NativeClipProtocolSequence>,
    plane: &Handle<dyn Graphic3dClipPlaneSource>,
) -> String {
    let mut cursor = Graphic3dSequenceOfHClipPlaneIterator::from_reference::<
        NativeClipProtocolSequence,
    >(sequence);
    let same = cursor.value_ref().unwrap().object_address() == plane.object_address();
    let mut rows = derived_iterator_count_row("initial", sequence, plane, same);
    cursor.next();
    rows += &format!("next {}\n", i32::from(cursor.more()));
    cursor.init_reference::<NativeClipProtocolSequence>(sequence);
    rows += &derived_iterator_count_row("restart", sequence, plane, cursor.more());
    rows
}

fn derived_iterator_count_row(
    label: &str,
    sequence: &Handle<NativeClipProtocolSequence>,
    plane: &Handle<dyn Graphic3dClipPlaneSource>,
    flag: bool,
) -> String {
    // Native constructor and Init borrow collection and current payload without temporary owners.
    format!(
        "{label} {} {} {}\n",
        sequence.ref_count(),
        i32::from(flag),
        plane.ref_count()
    )
}
