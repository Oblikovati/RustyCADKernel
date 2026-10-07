// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Tool.hxx and .cxx (counting, entries,
// labels and dumps) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_Tool`: label tree utilities. The reference queries are in `tool_references.rs`.
use super::{TdfAttributeIterator, TdfChildIterator, TdfData, TdfIdFilter, TdfLabel};
use crate::json::AttributeIndexMap;
use crate::live;
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use std::io;

/// `TDF_TagSeparator`.
const TAG_SEPARATOR: u8 = b':';

/// `TDF_Tool`.
pub struct TdfTool;
impl TdfTool {
    /// `NbLabels`: the label and its descendants.
    pub fn nb_labels(label: &TdfLabel) -> Result<i32, Failure> {
        let mut count = 1;
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            count += 1;
            children.next();
        }
        Ok(count)
    }
    /// `NbAttributes(aLabel)`: the valid attributes of the label and its descendants.
    pub fn nb_attributes(label: &TdfLabel) -> Result<i32, Failure> {
        let mut count = label.nb_attributes()?;
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            count += children.value().nb_attributes()?;
            children.next();
        }
        Ok(count)
    }
    /// `NbAttributes(aLabel, aFilter)`: only the kept ones.
    pub fn nb_attributes_filtered(label: &TdfLabel, filter: &TdfIdFilter) -> Result<i32, Failure> {
        let mut count = kept_attributes(label, filter)?;
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            count += kept_attributes(&children.value(), filter)?;
            children.next();
        }
        Ok(count)
    }
    /// `Entry`: the `0:1:2` text of a label (`0:` for the root, empty for a null label).
    ///
    /// ```
    /// use rustycad_tklcaf::tdf::{TdfData, TdfTool};
    /// use rustycad_tkernel::tcollection::AsciiString;
    /// let data = TdfData::new();
    /// let label = data.root().find_child(12, true).unwrap().find_child(3, true).unwrap();
    /// let mut entry = AsciiString::default();
    /// TdfTool::entry(&label, &mut entry);
    /// assert_eq!(entry.as_bytes(), b"0:12:3");
    /// TdfTool::entry(&data.root(), &mut entry);
    /// assert_eq!(entry.as_bytes(), b"0:");
    /// ```
    pub fn entry(label: &TdfLabel, entry: &mut AsciiString) {
        if label.is_null() {
            *entry = AsciiString::default();
            return;
        }
        let mut tags = Vec::new();
        let mut current = label.clone();
        while !current.is_root() && !current.is_null() {
            tags.push(current.tag());
            current = current.father();
        }
        if tags.is_empty() {
            *entry = AsciiString::new("0:");
            return;
        }
        let mut text = String::from("0");
        for tag in tags.iter().rev() {
            text.push(char::from(TAG_SEPARATOR));
            text.push_str(&tag_digits(*tag));
        }
        *entry = AsciiString::new(&text);
    }
    /// `TagList(aLabel, aTagList)`: the tags from the root to the label.
    pub fn tag_list(label: &TdfLabel, tags: &mut Vec<i32>) {
        tags.clear();
        let mut current = label.clone();
        while !current.is_null() {
            tags.insert(0, current.tag());
            if current.is_root() {
                break;
            }
            current = current.father();
        }
    }
    /// `TagList(anEntry, aTagList)`: the tags of an entry text; empty when it is not one.
    pub fn tag_list_from_entry(entry: &[u8], tags: &mut Vec<i32>) {
        tags.clear();
        let text = entry.split(|&byte| byte == 0).next().unwrap_or_default();
        let mut position = 0;
        let mut value: i32 = 0;
        while position < text.len() {
            while position < text.len() && text[position].is_ascii_digit() {
                value = value
                    .wrapping_mul(10)
                    .wrapping_add(i32::from(text[position] - b'0'));
                position += 1;
            }
            if position == text.len() || text[position] == TAG_SEPARATOR {
                tags.push(value);
                value = 0;
                position += usize::from(position < text.len());
            } else {
                tags.clear();
                return;
            }
        }
    }
    /// `Label(aDF, anEntry, aLabel, create)`: the label of an entry, through the fast access
    /// table when it is on.
    pub fn label_from_entry(
        data: &Handle<TdfData>,
        entry: &AsciiString,
        label: &mut TdfLabel,
        create: bool,
    ) -> Result<(), Failure> {
        let framework = live(data, "TDF_Tool::Label")?;
        if framework.is_access_by_entries() && framework.get_label(entry, label) {
            return Ok(());
        }
        let mut tags = Vec::new();
        Self::tag_list_from_entry(entry.as_bytes(), &mut tags);
        Self::label_from_tags(data, &tags, label, create)
    }
    /// `Label(aDF, anEntry, aLabel, create)` with a C-string entry (read up to its NUL).
    pub fn label_from_c_entry(
        data: &Handle<TdfData>,
        entry: &[u8],
        label: &mut TdfLabel,
        create: bool,
    ) -> Result<(), Failure> {
        let text =
            AsciiString::from_bytes(entry.split(|&byte| byte == 0).next().unwrap_or_default());
        Self::label_from_entry(data, &text, label, create)
    }
    /// `Label(aDF, aTagList, aLabel, create)`: the label of a tag list from the root; null
    /// for an empty list or a missing label.
    pub fn label_from_tags(
        data: &Handle<TdfData>,
        tags: &[i32],
        label: &mut TdfLabel,
        create: bool,
    ) -> Result<(), Failure> {
        let Some((_, children)) = tags.split_first() else {
            label.nullify();
            return Ok(());
        };
        *label = live(data, "TDF_Tool::Label")?.root();
        if tags.len() == 1 && tags[0] == 0 {
            return Ok(());
        }
        for tag in children {
            if label.is_null() {
                break;
            }
            *label = label.find_child(*tag, create)?;
        }
        Ok(())
    }
    /// `RelocateLabel`: the label under `to_root` with the tags `source` has under
    /// `from_root`; unchanged when `source` is not below `from_root`.
    pub fn relocate_label(
        source: &TdfLabel,
        from_root: &TdfLabel,
        to_root: &TdfLabel,
        target: &mut TdfLabel,
        create: bool,
    ) -> Result<(), Failure> {
        if !source.is_descendant(from_root) {
            return Ok(());
        }
        target.nullify();
        let mut tags = Vec::new();
        Self::tag_list(source, &mut tags);
        let mut to_tags = Vec::new();
        Self::tag_list(to_root, &mut to_tags);
        let skipped = usize::try_from(from_root.depth()? + 1)
            .unwrap_or(0)
            .min(tags.len());
        to_tags.extend_from_slice(&tags[skipped..]);
        Self::label_from_tags(&to_root.data(), &to_tags, target, create)
    }
    /// `CountLabels`: counts each label of the list in the map and removes the repeated
    /// ones from the list.
    pub fn count_labels(labels: &mut Vec<TdfLabel>, counts: &mut DataMap<TdfLabel, i32>) {
        labels.retain(|label| match counts.change_find(label) {
            Ok(count) => {
                *count += 1;
                false
            }
            Err(_) => {
                counts.bind(label.clone(), 1);
                true
            }
        });
    }
    /// `DeductLabels`: decrements the counts and removes the labels that reach 0.
    pub fn deduct_labels(labels: &mut Vec<TdfLabel>, counts: &mut DataMap<TdfLabel, i32>) {
        labels.retain(|label| {
            let Ok(count) = counts.change_find(label) else {
                return true;
            };
            *count -= 1;
            if *count != 0 {
                return true;
            }
            counts.unbind(label);
            false
        });
    }
    /// `DeepDump(anOS, aDF)`: the framework, then its label tree.
    pub fn deep_dump_data(
        stream: &mut dyn io::Write,
        data: &Handle<TdfData>,
    ) -> Result<(), Failure> {
        let framework = live(data, "TDF_Tool::DeepDump")?;
        framework.dump(stream).map_err(io_failure)?;
        Self::deep_dump(stream, &framework.root())
    }
    /// `DeepDump(anOS, aLabel)`: the label and its sub-tree.
    pub fn deep_dump(stream: &mut dyn io::Write, label: &TdfLabel) -> Result<(), Failure> {
        label.dump(stream).map_err(io_failure)?;
        let mut children = TdfChildIterator::new(label, false)?;
        while children.more() {
            Self::deep_dump(stream, &children.value())?;
            children.next();
        }
        Ok(())
    }
    /// `ExtendedDeepDump(anOS, aDF, aFilter)`.
    pub fn extended_deep_dump_data(
        stream: &mut dyn io::Write,
        data: &Handle<TdfData>,
        filter: &TdfIdFilter,
    ) -> Result<(), Failure> {
        let framework = live(data, "TDF_Tool::ExtendedDeepDump")?;
        framework.dump(stream).map_err(io_failure)?;
        Self::extended_deep_dump(stream, &framework.root(), filter)
    }
    /// `ExtendedDeepDump(anOS, aLabel, aFilter)`: the labels with attribute ranks, then the
    /// extended dump of every kept attribute.
    pub fn extended_deep_dump(
        stream: &mut dyn io::Write,
        label: &TdfLabel,
        filter: &TdfIdFilter,
    ) -> Result<(), Failure> {
        let mut map = AttributeIndexMap::default();
        extended_tree_dump(stream, label, filter, &mut map)?;
        super::tool_dump::dump_ranked_attributes(stream, filter, &mut map).map_err(io_failure)
    }
}

/// The digits of a tag as the native entry writes them (`char(tag % 10) + '0'`).
fn tag_digits(tag: i32) -> String {
    let mut digits = Vec::new();
    let mut value = tag;
    while value > 9 {
        digits.push(char::from((value % 10) as u8 + b'0'));
        value /= 10;
    }
    digits.push(char::from((value as u8).wrapping_add(b'0')));
    digits.iter().rev().collect()
}

/// The kept attributes of one label.
fn kept_attributes(label: &TdfLabel, filter: &TdfIdFilter) -> Result<i32, Failure> {
    let mut iterator = TdfAttributeIterator::new(label, true)?;
    let mut count = 0;
    while iterator.more() {
        count += i32::from(filter.is_kept_attribute(&iterator.value()));
        iterator.next();
    }
    Ok(count)
}

/// The file-static `TDF_Tool_ExtendedDeepDump`.
fn extended_tree_dump(
    stream: &mut dyn io::Write,
    label: &TdfLabel,
    filter: &TdfIdFilter,
    map: &mut AttributeIndexMap,
) -> Result<(), Failure> {
    label
        .extended_dump(stream, filter, map)
        .map_err(io_failure)?;
    let mut children = TdfChildIterator::new(label, false)?;
    while children.more() {
        extended_tree_dump(stream, &children.value(), filter, map)?;
        children.next();
    }
    Ok(())
}

/// A stream error as a `Standard_Failure`.
pub(crate) fn io_failure(error: io::Error) -> Failure {
    Failure::new(&format!("stream write failed: {error}"))
}
