use super::super::*;
fn values<'a>(items: impl Iterator<Item = &'a PublicCopyItem>) -> String {
    let joined = items
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    if joined.is_empty() {
        "-".to_owned()
    } else {
        joined
    }
}
fn row(
    name: &str,
    world: &CopyWorld,
    array: &mut DynamicArray<PublicCopyItem>,
    failed: bool,
    partial: &[PublicCopyItem],
) -> String {
    let prefix = row_prefix(name, world, array, failed, partial.len());
    format!(
        "{prefix} live={} allocations={},{} values={} incomplete={}\n",
        world.live.get(),
        live_allocations(&world.source),
        live_allocations(&world.target),
        values(array.iter()),
        values(partial.iter())
    )
}
fn row_prefix(
    name: &str,
    world: &CopyWorld,
    array: &mut DynamicArray<PublicCopyItem>,
    failed: bool,
    partial: usize,
) -> String {
    format!(
        "{name} {} size={} blocks={} source={} failed={} partial={}",
        world.events.borrow(),
        array.size(),
        array.storage_extension().block_count(),
        u8::from(array.allocator() == Some(&world.source)),
        u8::from(failed),
        partial
    )
}
fn inspect_failure(error: &DynamicArrayCopyConstructionFailure<PublicCopyItem>, failure: usize) {
    assert_eq!(
        error.published_array().size(),
        if failure == 3 { 2 } else { 0 }
    );
    assert!(error.copy_failure().has_incomplete_block());
    assert!(format!("{error:?}").contains("published_size"));
    assert!(format!("{error}").contains("copy failure"));
    assert!(std::error::Error::source(error).is_some());
}
fn constructor_row(failure: usize) -> String {
    let fixture = CopyFixture::new(failure);
    let result = DynamicArray::from_copied_with(&fixture.source, PublicCopyItem::construct_copy);
    let name = format!("constructor_{failure}");
    let output = match result {
        Ok(mut copied) => row(&name, &fixture.world, &mut copied, false, &[]),
        Err(error) => failed_constructor_row(&name, &fixture.world, error, failure),
    };
    assert_eq!(fixture.world.live.get(), 6);
    output
}
fn failed_constructor_row(
    name: &str,
    world: &CopyWorld,
    error: DynamicArrayCopyConstructionFailure<PublicCopyItem>,
    failure: usize,
) -> String {
    inspect_failure(&error, failure);
    let (mut published, incomplete) = error.into_parts();
    row(
        name,
        world,
        &mut published,
        true,
        incomplete.incomplete_values(),
    )
}
fn assignment_result(
    fixture: &mut CopyFixture,
    mode: usize,
) -> Option<rustycad_tkernel::ncollection::DynamicArrayCopyFailure<PublicCopyItem>> {
    let address = std::ptr::from_ref(&fixture.target);
    let result = apply_assignment(fixture, mode);
    match result {
        Ok(returned) => {
            assert_eq!(std::ptr::from_ref(returned), address);
            None
        }
        Err(error) => Some(error),
    }
}
fn apply_assignment(
    fixture: &mut CopyFixture,
    mode: usize,
) -> Result<
    &mut DynamicArray<PublicCopyItem>,
    rustycad_tkernel::ncollection::DynamicArrayCopyFailure<PublicCopyItem>,
> {
    if mode == 2 {
        fixture
            .target
            .assign_operator_with_constructor(&fixture.source, PublicCopyItem::construct_copy)
    } else {
        fixture.target.assign_with_constructor(
            &fixture.source,
            mode == 0,
            PublicCopyItem::construct_copy,
        )
    }
}
fn assignment_row(mode: usize, failure: usize) -> String {
    let mut fixture = CopyFixture::new(failure);
    let incomplete = assignment_result(&mut fixture, mode);
    row(
        &format!("assignment_{mode}_{failure}"),
        &fixture.world,
        &mut fixture.target,
        incomplete.is_some(),
        incomplete
            .as_ref()
            .map_or(&[], |error| error.incomplete_values()),
    )
}
#[test]
fn public_copy_native_rows_preserve_pre_recovery_lifetime_and_allocator_phases() {
    let mut actual = String::new();
    for failure in 0..=3 {
        actual.push_str(&constructor_row(failure));
    }
    for mode in 0..3 {
        for failure in 0..=3 {
            actual.push_str(&assignment_row(mode, failure));
        }
    }
    assert_eq!(
        actual,
        include_str!("../../../../tools/oracle/tkernel_dynamic_public_copy.expected")
    );
}
