// Rust port of DE_Wrapper::Load, Save and sort of DataExchange/TKDE/DE/DE_Wrapper.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The resource text of `DE_Wrapper`: the vendor priority of every format, the units and
//! the settings of every node.
use super::configuration_node::write_resource_file;
use super::{DeConfigurationContext, DeConfigurationNode, DeSectionGlobal, DeWrapper, live};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
    tcollection::AsciiString,
};

/// The native `THE_CONFIGURATION_SCOPE()` of `DE_Wrapper`.
const CONFIGURATION_SCOPE: &str = "global";

/// The header lines of a saved configuration.
const SAVE_HEADER: &str = "!Description of the config file for DE toolkit\n\
!*****************************************************************************\n\
!\n\
!Format of the file is compliant with the standard Open CASCADE resource files\n\
!Each key defines a sequence of either further keys.\n\
!Keys can be nested down to an arbitrary level.\n\
!\n\
!*****************************************************************************\n\
!DE_Wrapper\n\
!Priority vendor list. For every CAD format set indexed list of vendors\n";

/// The closing line of a saved configuration.
const SAVE_FOOTER: &str =
    "!*****************************************************************************\n";

/// One format of a save: its name and its vendors (key and node) in priority order.
type FormatNodes = (
    AsciiString,
    Vec<(AsciiString, Handle<dyn DeConfigurationNode>)>,
);

impl DeWrapper {
    /// `Load(theResource, theIsRecursive)`: loads a resource file, or the text itself when
    /// no such file exists, then [`DeWrapper::load_context`]. A blank line in the text
    /// raises `Standard_OutOfRange`, as natively.
    pub fn load(&self, resource: &AsciiString, is_recursive: bool) -> Result<bool, Failure> {
        let context: Handle<DeConfigurationContext> = Handle::new(DeConfigurationContext::new());
        live(&context, "DE_Wrapper::Load")?.load(resource)?;
        self.load_context(&context, is_recursive)
    }

    /// `Load(theResource, theIsRecursive)` of a context: reads the units of the `global`
    /// scope; recursively, loads every node and applies the `global.priority.FORMAT` lists
    /// (listed vendors first, the others disabled). Always `true`; a null context raises
    /// `Standard_NullObject`.
    pub fn load_context(
        &self,
        resource: &Handle<DeConfigurationContext>,
        is_recursive: bool,
    ) -> Result<bool, Failure> {
        let context: &DeConfigurationContext = live(resource, "DE_Wrapper::Load")?;
        let mut units: DeSectionGlobal = self.global_parameters();
        units.length_unit = context.real_val(
            "general.length.unit",
            units.length_unit,
            CONFIGURATION_SCOPE,
        );
        units.system_unit = context.real_val(
            "general.system.unit",
            units.system_unit,
            CONFIGURATION_SCOPE,
        );
        self.set_global_parameters(units);
        if is_recursive {
            for node in self.node_snapshot() {
                live(&node, "DE_Wrapper::Load")?.load(resource)?;
            }
            self.sort(context);
        }
        Ok(true)
    }

    /// `Save(theResourcePath, theIsRecursive, theFormats, theVendors)`: writes
    /// [`DeWrapper::save`] to a file; `false` (without a message) when the file cannot be
    /// created.
    pub fn save_to_file(
        &self,
        resource_path: &AsciiString,
        is_recursive: bool,
        formats: &List<AsciiString>,
        vendors: &List<AsciiString>,
    ) -> Result<bool, Failure> {
        write_resource_file(resource_path, false, || {
            self.save(is_recursive, formats, vendors)
        })
    }

    /// `Save(theIsRecursive, theFormats, theVendors)`: the priority list of every format (the
    /// vendor names the nodes report), the units, then, recursively, the settings of the
    /// nodes of the listed formats and vendors (all of them for an empty list).
    pub fn save(
        &self,
        is_recursive: bool,
        formats: &List<AsciiString>,
        vendors: &List<AsciiString>,
    ) -> AsciiString {
        let snapshot: Vec<FormatNodes> = self.format_snapshot();
        let mut result: AsciiString = AsciiString::new(SAVE_HEADER);
        for (format, nodes) in &snapshot {
            append_priority_line(&mut result, format, nodes);
        }
        append_units(&mut result, self.global_parameters());
        if is_recursive {
            for (format, nodes) in &snapshot {
                append_node_settings(&mut result, format, nodes, formats, vendors);
            }
        }
        result += SAVE_FOOTER;
        result
    }

    /// The protected `sort`: applies each `global.priority.FORMAT` list of the context to its
    /// format, disabling the vendors it omits; a format without a list keeps its order.
    pub fn sort(&self, resource: &DeConfigurationContext) {
        let scope: String = format!("{CONFIGURATION_SCOPE}.priority");
        for format in self.format_keys() {
            let mut vendor_priority: List<AsciiString> = List::default();
            if resource.get_string_seq(&format, &mut vendor_priority, &scope) {
                self.change_priority(&format, &vendor_priority, true);
            }
        }
    }

    /// The formats with their vendors, in map order.
    fn format_snapshot(&self) -> Vec<FormatNodes> {
        let configuration = self.configuration.read();
        configuration
            .items()
            .map(|(format, vendors)| {
                let nodes = vendors
                    .items()
                    .map(|(vendor, node)| (vendor.clone(), node.clone()));
                (format.clone(), nodes.collect())
            })
            .collect()
    }
}

/// `global.priority.FORMAT :\t VENDOR VENDOR \n`, with the vendor name each node reports.
fn append_priority_line(
    result: &mut AsciiString,
    format: &AsciiString,
    nodes: &[(AsciiString, Handle<dyn DeConfigurationNode>)],
) {
    *result += format!("{CONFIGURATION_SCOPE}.priority.").as_str();
    *result += format;
    *result += " :\t ";
    for (_, node) in nodes {
        if let Some(object) = node.get() {
            *result += &object.get_vendor();
        }
        *result += " ";
    }
    *result += "\n";
}

/// The two unit lines with their comments; the values are written with `%g`.
fn append_units(result: &mut AsciiString, units: DeSectionGlobal) {
    *result += "!Global parameters. Used for all providers\n";
    *result += "!Length scale unit value. Should be more than 0. Default value: 1.0(MM)\n";
    *result += format!("{CONFIGURATION_SCOPE}.general.length.unit :\t ").as_str();
    *result += units.length_unit;
    *result += "\n";
    *result += "!System unit value. Should be more than 0. Default value: 1.0(MM)\n";
    *result += format!("{CONFIGURATION_SCOPE}.general.system.unit :\t ").as_str();
    *result += units.system_unit;
    *result += "\n";
}

/// The settings of the nodes of one format, each framed by `!` lines, filtered by the
/// format and vendor lists (an empty list keeps everything).
fn append_node_settings(
    result: &mut AsciiString,
    format: &AsciiString,
    nodes: &[(AsciiString, Handle<dyn DeConfigurationNode>)],
    formats: &List<AsciiString>,
    vendors: &List<AsciiString>,
) {
    if !formats.is_empty() && !formats.contains(format) {
        return;
    }
    for (vendor, node) in nodes {
        if !vendors.is_empty() && !vendors.contains(vendor) {
            continue;
        }
        *result += "!\n";
        if let Some(object) = node.get() {
            *result += &object.save();
        }
        *result += "!\n";
    }
}
