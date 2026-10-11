//! Native JSON preserves exact diagnostic fragments, depth and NaNs; only addresses vary by process.
use super::group_oracle_geometry::group_children;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::dump::DumpStream;
fn group_json_case(kind: i32, depth: i32) -> String {
    let owner = SelectMgrEntityOwner::new(13).cast();
    let children = group_children(if kind == 4 { 1 } else { kind }, &owner);
    let group = if kind == 0 {
        Select3DSensitiveGroup::new(&owner, true)
    } else {
        Select3DSensitiveGroup::with_entities(&owner, &children, true)
    };
    if kind == 4 {
        group.remove(&children[0]);
    }
    let mut text = DumpStream::default();
    group.dump_json(&mut text, depth).unwrap();
    normalize_group_json_pointers(text.text())
}
pub(super) fn normalize_group_json_pointers(text: &str) -> String {
    let mut normalized = String::new();
    let mut remaining = text;
    while let Some(position) = remaining.find("0xx") {
        normalized.push_str(&remaining[..position]);
        normalized.push_str("@pointer");
        let pointer = &remaining[position + 3..];
        let length = pointer.bytes().take_while(u8::is_ascii_hexdigit).count();
        remaining = &pointer[length..];
    }
    normalized.push_str(remaining);
    normalized
}
#[test]
fn native_group_json_depth_nan_and_cached_structural_state_are_preserved() {
    let native = include_str!("../../tests/fixtures/sensitive-group-json.txt");
    for line in native.lines() {
        let mut fields = line.splitn(4, ' ');
        let kind = fields.next().unwrap().parse().unwrap();
        let depth = fields.next().unwrap().parse().unwrap();
        assert_eq!(fields.next().unwrap(), "0");
        assert_eq!(
            group_json_case(kind, depth),
            fields.next().unwrap(),
            "kind {kind} depth {depth}"
        );
    }
    assert_eq!(native.lines().count(), 30);
}
