//! Native nested diagnostic failures discard only the failing child's buffer and preserve prior output.
use super::group_dump_child_fake::{GroupDumpChildFake, GroupDumpContext};
use super::group_json_tests::normalize_group_json_pointers;
use super::triangulation_failure::catch_triangulation_failure;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use std::sync::{Arc, atomic::Ordering};
struct GroupJsonFailureScenario {
    context: Arc<GroupDumpContext>,
    group: Handle<Select3DSensitiveGroup>,
    stage: i32,
}
impl GroupJsonFailureScenario {
    fn new(stage: i32) -> Self {
        let owner = SelectMgrEntityOwner::new(13).cast();
        let group = Handle::new(Select3DSensitiveGroup::new(&owner, true));
        let context = Arc::new(GroupDumpContext::new(group.clone()));
        insert_dump_children(&group, &owner, &context, stage);
        context.trace.write().clear();
        context.stage.store(stage, Ordering::Relaxed);
        Self {
            context,
            group,
            stage,
        }
    }
    fn dump(&self, depth: i32, output: &mut DumpStream) -> Result<i32, Failure> {
        catch_triangulation_failure(|| match self.group.dump_json(output, depth) {
            Ok(()) => Ok(0),
            Err(error) => {
                assert!(error.to_string().contains("expected successful"));
                Ok(2)
            }
        })
    }
    fn replay(&self, depth: i32, native_trace: &str, native_json: &str) {
        let mut output = DumpStream::default();
        let failure = self.dump(depth, &mut output).unwrap_or(1);
        let trace = dump_trace_text(&self.context, self.stage, depth);
        assert_eq!(trace, native_trace);
        let actual = format!(
            "{} {} {} {}",
            self.stage + 5,
            depth,
            failure,
            normalize_group_json_pointers(output.text())
        );
        assert_eq!(actual, native_json);
    }
}
impl Drop for GroupJsonFailureScenario {
    fn drop(&mut self) {
        // Neither the test seam nor the native context keeps a cycle after its scenario ends.
        *self.context.parent.write() = Handle::default();
    }
}
#[test]
fn native_group_dump_failures_and_policy_mutation_preserve_nested_output_and_callback_order() {
    let mut lines = include_str!("../../tests/fixtures/sensitive-group-json-failures.txt").lines();
    for stage in 0..=4 {
        for depth in [-2, -1, 0, 1, 2, 3] {
            GroupJsonFailureScenario::new(stage).replay(
                depth,
                lines.next().unwrap(),
                lines.next().unwrap(),
            );
        }
    }
    assert!(lines.next().is_none());
}

fn insert_dump_children(
    group: &Handle<Select3DSensitiveGroup>,
    owner: &Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource>,
    context: &Arc<GroupDumpContext>,
    stage: i32,
) {
    for identity in 1..=3 {
        group.add(
            &Handle::new(GroupDumpChildFake::new(
                owner,
                context.clone(),
                identity,
                stage == 4,
            ))
            .cast(),
        );
    }
}
fn dump_trace_text(context: &GroupDumpContext, stage: i32, depth: i32) -> String {
    let events = context.trace.read();
    let callbacks = events
        .iter()
        .map(|event| format!(" {event}"))
        .collect::<String>();
    format!("{} {depth} {}{callbacks}", stage + 5, events.len())
}
