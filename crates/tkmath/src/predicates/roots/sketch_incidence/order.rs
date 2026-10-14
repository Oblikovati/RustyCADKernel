// SPDX-License-Identifier: GPL-2.0-only
//! Exact total order of contacts along the source, with proved ties grouped.
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::input::SketchLineDomain;
use super::place::{SourcePlace, compare_places};
use super::report::{SketchContactGroup, SketchTargetIncidences};
use std::cmp::Ordering;

struct OrderedGroup {
    members: Vec<[usize; 2]>,
    representative: SourcePlace,
}
/// Insert every in-domain contact in request order; equal parameters share a group.
pub(super) fn order_contacts(
    targets: &[SketchTargetIncidences],
    domain: SketchLineDomain,
    work: &mut Work<'_>,
) -> Result<Vec<SketchContactGroup>, Cause> {
    let mut groups: Vec<OrderedGroup> = Vec::new();
    for (target_index, target) in targets.iter().enumerate() {
        for (contact_index, contact) in target.contacts.iter().enumerate() {
            if domain == SketchLineDomain::Segment && !contact.source_position.within_closed() {
                continue;
            }
            let mut place = contact.place.clone();
            insert(&mut groups, [target_index, contact_index], &mut place, work)?;
        }
    }
    Ok(groups
        .into_iter()
        .map(|group| SketchContactGroup {
            members: group.members,
        })
        .collect())
}
fn insert(
    groups: &mut Vec<OrderedGroup>,
    member: [usize; 2],
    root: &mut SourcePlace,
    work: &mut Work<'_>,
) -> Result<(), Cause> {
    for index in 0..groups.len() {
        match compare_places(root, &mut groups[index].representative, work)? {
            Ordering::Greater => continue,
            Ordering::Equal => {
                groups[index].members.push(member);
                return Ok(());
            }
            Ordering::Less => {
                groups.insert(index, group(member, root));
                return Ok(());
            }
        }
    }
    groups.push(group(member, root));
    Ok(())
}
fn group(member: [usize; 2], root: &SourcePlace) -> OrderedGroup {
    OrderedGroup {
        members: vec![member],
        representative: root.clone(),
    }
}
