//! Graphic3d_SequenceOfHClipPlane, Graphic3d_ViewAffinity and the Graphic3d exceptions.
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::graphic3d::{
    Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane, Graphic3dSequenceOfHClipPlaneIterator,
    Graphic3dViewAffinity, group_definition_error, priority_definition_error,
    structure_definition_error,
};

fn planes(count: usize) -> Vec<Handle<Graphic3dClipPlane>> {
    (0..count)
        .map(|_| Handle::new(Graphic3dClipPlane::default()))
        .collect()
}

#[test]
fn append_keeps_each_plane_once() {
    let sequence = Graphic3dSequenceOfHClipPlane::default();
    let [a, b] = <[_; 2]>::try_from(planes(2)).unwrap();
    assert!(sequence.append(a.clone()));
    assert!(!sequence.append(a.clone()));
    assert!(sequence.append(b.clone()));
    assert_eq!(sequence.size(), 2);
    assert_eq!(sequence.first().unwrap(), a);
    assert!(sequence.remove(&a));
    assert!(!sequence.remove(&a));
    assert_eq!(sequence.first().unwrap(), b);
}

#[test]
fn first_of_empty_sequence_raises() {
    let sequence = Graphic3dSequenceOfHClipPlane::default();
    assert!(sequence.is_empty());
    let error = sequence.first().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
}

#[test]
fn iterator_remove_moves_to_the_next_plane() {
    let sequence = Graphic3dSequenceOfHClipPlane::default();
    let all = planes(3);
    all.iter().for_each(|plane| {
        sequence.append(plane.clone());
    });
    let mut it = Graphic3dSequenceOfHClipPlaneIterator::new(&sequence);
    it.next();
    sequence.remove_at(&mut it).unwrap();
    assert_eq!(it.value(), all[2]);
    assert_eq!(sequence.items(), vec![all[0].clone(), all[2].clone()]);
    it.next();
    assert!(!it.more());
    sequence.remove_at(&mut it).unwrap();
    let null = Graphic3dSequenceOfHClipPlaneIterator::from_handle(&Handle::<
        Graphic3dSequenceOfHClipPlane,
    >::default());
    assert!(!null.more());
    sequence.clear();
    assert!(sequence.is_empty());
}

#[test]
fn removing_a_default_or_null_initialized_iterator_is_a_native_noop() {
    let sequence = Graphic3dSequenceOfHClipPlane::default();
    let plane = planes(1).pop().unwrap();
    sequence.append(plane.clone());
    let mut iterator = Graphic3dSequenceOfHClipPlaneIterator::default();
    sequence.remove_at(&mut iterator).unwrap();
    iterator.init_handle(&Handle::<Graphic3dSequenceOfHClipPlane>::default());
    sequence.remove_at(&mut iterator).unwrap();
    assert_eq!(sequence.size(), 1);
    assert_eq!(plane.ref_count(), 2);
    assert!(!iterator.more());
    assert_native_end_cursor(&sequence, &iterator);
}

fn assert_native_end_cursor(
    sequence: &Graphic3dSequenceOfHClipPlane,
    iterator: &Graphic3dSequenceOfHClipPlaneIterator,
) {
    let actual = format!(
        "remove-end {} {}",
        i32::from(iterator.more()),
        sequence.size()
    );
    let expected = include_str!("fixtures/clip-collection-references-native.txt");
    assert_eq!(actual, expected.lines().last().unwrap());
}

#[test]
fn dump_lists_the_flag_then_the_planes() {
    let sequence = Graphic3dSequenceOfHClipPlane::default();
    sequence.set_override_global(true);
    assert!(sequence.to_override_global());
    sequence.append(planes(1).remove(0));
    let mut stream = DumpStream::default();
    sequence.dump_json(&mut stream, -1).unwrap();
    let text = String::from_utf8(stream.bytes().to_vec()).unwrap();
    assert!(text.starts_with("\"ToOverrideGlobal\": 1"), "{text}");
    assert!(
        text.contains("\"className\": \"Graphic3d_ClipPlane\""),
        "{text}"
    );
}

#[test]
fn view_affinity_masks_views() {
    let affinity = Graphic3dViewAffinity::default();
    assert!(affinity.is_visible(0) && affinity.is_visible(31));
    affinity.set_view_visible(3, false);
    assert!(!affinity.is_visible(3) && affinity.is_visible(2));
    affinity.set_visible(false);
    assert!(!affinity.is_visible(0));
    affinity.set_view_visible(31, true);
    assert!(affinity.is_visible(31));
    let mut stream = DumpStream::default();
    affinity.dump_json(&mut stream, -1).unwrap();
    let text = String::from_utf8(stream.bytes().to_vec()).unwrap();
    assert!(text.ends_with("\"Mask\": 2147483648"), "{text}");
}

#[test]
fn exceptions_keep_their_class_names() {
    assert_eq!(
        group_definition_error("x").exception_type(),
        "Graphic3d_GroupDefinitionError"
    );
    assert_eq!(
        priority_definition_error("x").exception_type(),
        "Graphic3d_PriorityDefinitionError"
    );
    assert_eq!(
        structure_definition_error("x").exception_type(),
        "Graphic3d_StructureDefinitionError"
    );
}
