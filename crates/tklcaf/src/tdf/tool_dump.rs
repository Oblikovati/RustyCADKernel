// Rust port of the dumps of ApplicationFramework/TKLCAF/TDF/TDF_Label.cxx and the ranked
// attribute dump of TDF_Tool::ExtendedDeepDump at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_Label::Dump`, `ExtendedDump`, `EntryDump` and the attribute part of
//! `TDF_Tool::ExtendedDeepDump`.
use super::{
    TdfAttributeIterator, TdfAttributeState, TdfIdFilter, TdfLabel, TdfTool,
    attribute::dump_attribute,
};
use crate::json::AttributeIndexMap;
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use std::io;

impl TdfLabel {
    /// `Dump` (and `operator<<`): the entry, the flags and the attributes.
    pub fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let mut map = AttributeIndexMap::default();
        self.internal_dump(stream, &TdfIdFilter::default(), &mut map, false)
    }
    /// `ExtendedDump`: as `Dump`, with the rank of each kept attribute in `map`.
    pub fn extended_dump(
        &self,
        stream: &mut dyn io::Write,
        filter: &TdfIdFilter,
        map: &mut AttributeIndexMap,
    ) -> io::Result<()> {
        self.internal_dump(stream, filter, map, true)
    }
    /// `EntryDump`: the entry, or `This label is null.`.
    pub fn entry_dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        if self.is_null() {
            return write!(stream, "This label is null.");
        }
        stream.write_all(entry_text(self).as_bytes())
    }
    /// Private `InternalDump`.
    fn internal_dump(
        &self,
        stream: &mut dyn io::Write,
        filter: &TdfIdFilter,
        map: &mut AttributeIndexMap,
        extended: bool,
    ) -> io::Result<()> {
        if self.is_null() {
            return write!(stream, "This label is null.");
        }
        write!(
            stream,
            "{}\t",
            String::from_utf8_lossy(entry_text(self).as_bytes())
        )?;
        write!(
            stream,
            "{} imported; ",
            if self.is_imported() { "IS " } else { "NOT" }
        )?;
        write!(
            stream,
            "{} modified; ",
            if self.may_be_modified() {
                "MAYBE"
            } else {
                "NOT"
            }
        )?;
        let attributes = if self.attributes_modified() {
            "HAS attributes"
        } else {
            "NO attribute"
        };
        write!(stream, "{attributes} modified; ")?;
        let count = self.nb_attributes().unwrap_or(0);
        if count == 0 {
            return writeln!(stream, " has no attribute");
        }
        writeln!(
            stream,
            "has {count} attribute{}.",
            if count > 1 { "s" } else { "" }
        )?;
        let Ok(mut iterator) = TdfAttributeIterator::new(self, true) else {
            return Ok(());
        };
        while iterator.more() {
            let attribute = iterator.value();
            if extended && filter.is_kept_attribute(&attribute) {
                write!(stream, "\t# {}", map.add(attribute.clone()))?;
            }
            if let Some(attribute) = attribute.get() {
                dump_attribute(attribute, stream)?;
            }
            iterator.next();
        }
        Ok(())
    }
}

/// The entry of a label as text.
fn entry_text(label: &TdfLabel) -> AsciiString {
    let mut entry = AsciiString::default();
    TdfTool::entry(label, &mut entry);
    entry
}

/// The second half of `TDF_Tool::ExtendedDeepDump(anOS, aLabel, aFilter)`: the count of
/// ranked attributes, then the extended dump of each kept one.
pub(crate) fn dump_ranked_attributes(
    stream: &mut dyn io::Write,
    filter: &TdfIdFilter,
    map: &mut AttributeIndexMap,
) -> io::Result<()> {
    let total = map.extent();
    writeln!(
        stream,
        "{total} attribute{} referenced by the label structure.",
        plural(total)
    )?;
    writeln!(stream, "\nExtended dump of filtered attribute(s):")?;
    let mut dumped = 0;
    let mut index = 1;
    while index <= map.extent() {
        let attribute: Handle<_> = map.find_key(index).cloned().unwrap_or_default();
        if filter.is_kept_attribute(&attribute) {
            dumped += 1;
            write!(stream, "# {index}")?;
            let label = attribute.get().map(|a| a.label()).unwrap_or_default();
            if label.is_null() {
                writeln!(stream, " (no label)")?;
            } else {
                writeln!(
                    stream,
                    " (label: {})",
                    String::from_utf8_lossy(entry_text(&label).as_bytes())
                )?;
            }
            if let Some(attribute) = attribute.get() {
                attribute.extended_dump(stream, filter, map)?;
            }
            writeln!(stream)?;
        }
        index += 1;
    }
    writeln!(
        stream,
        "\n{dumped} attribute{} dumped between {}",
        plural(dumped),
        index - 1
    )
}

fn plural(count: usize) -> &'static str {
    if count > 1 { "s" } else { "" }
}
