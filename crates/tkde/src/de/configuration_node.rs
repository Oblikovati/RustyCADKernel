// Rust port of DataExchange/TKDE/DE/DE_ConfigurationNode.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_ConfigurationNode`: the settings of one format and vendor, which build the provider
//! that transfers files of that format.
use super::configuration_node_base::DeConfigurationNodeBase;
use super::{DeConfigurationContext, DeProvider, DeSectionGlobal};
use rustycad_tkernel::{
    impl_transient_interface,
    message::Message,
    ncollection::{Buffer, List},
    osd::{CadPath, File, OpenMode, OsdErrorAccess, Protection},
    standard::{Failure, Handle, StandardTransient, Transient},
    tcollection::AsciiString,
};

/// `DE_ConfigurationNode`: the settings of one provider type, grouped by format and vendor.
///
/// A concrete node embeds a [`DeConfigurationNodeBase`] and implements the pure virtual
/// members; the virtual ones are provided with the native default body. `Load(path)` is
/// [`DeConfigurationNode::load_resource_file`] and `Save(path)` is
/// [`DeConfigurationNode::save_resource_file`] (de-design.md section 1).
pub trait DeConfigurationNode: Transient {
    /// The embedded base with the native private fields.
    fn configuration_node_base(&self) -> &DeConfigurationNodeBase;
    /// `Load(theResource)` (pure virtual): reads the node's values from a context.
    fn load(&self, resource: &Handle<DeConfigurationContext>) -> bool;
    /// `Save()` (pure virtual): the node's values as resource text.
    fn save(&self) -> AsciiString;
    /// `BuildProvider()` (pure virtual): a new provider for this node's format.
    fn build_provider(&self) -> Handle<dyn DeProvider>;
    /// `Copy()` (pure virtual): a new node with the same values.
    fn copy(&self) -> Handle<dyn DeConfigurationNode>;
    /// `GetFormat()` (pure virtual): the CAD format name.
    fn get_format(&self) -> AsciiString;
    /// `GetVendor()` (pure virtual): the vendor name.
    fn get_vendor(&self) -> AsciiString;
    /// `GetExtensions()` (pure virtual): the file extensions of the format.
    fn get_extensions(&self) -> List<AsciiString>;

    /// `Load(theResourcePath = "")`: reads a resource file into a new context and loads it.
    /// An empty path raises `Standard_ProgramError` from `OSD_File::Open`, as natively.
    fn load_resource_file(&self, resource_path: &AsciiString) -> Result<bool, Failure> {
        let resource: Handle<DeConfigurationContext> = Handle::new(DeConfigurationContext::new());
        if let Some(context) = resource.get() {
            context.load_file(resource_path)?;
        }
        Ok(self.load(&resource))
    }
    /// `UpdateLoad(theToImport, theToKeep)`: whether the node can be used; `true` by default.
    fn update_load(&self, to_import: bool, to_keep: bool) -> bool {
        let _ = (to_import, to_keep);
        true
    }
    /// `IsImportSupported()`: `false` by default.
    fn is_import_supported(&self) -> bool {
        false
    }
    /// `IsExportSupported()`: `false` by default.
    fn is_export_supported(&self) -> bool {
        false
    }
    /// `IsStreamSupported()`: `false` by default.
    fn is_stream_supported(&self) -> bool {
        false
    }
    /// `CheckExtension`: compares an extension, with or without its leading dot, to
    /// [`DeConfigurationNode::get_extensions`] without case.
    fn check_extension(&self, extension: &AsciiString) -> bool {
        matches_extension(&self.get_extensions(), extension)
    }
    /// `CheckContent(theBuffer)`: `false` by default.
    fn check_content(&self, buffer: &Handle<Buffer>) -> bool {
        let _ = buffer;
        false
    }
    /// `CustomActivation`: activation of a commercial component; nothing by default.
    fn custom_activation(&self, arguments: &List<AsciiString>) {
        let _ = arguments;
    }
    /// `Save(theResourcePath)` (non-virtual): writes [`DeConfigurationNode::save`] to a
    /// file; `false` with a failure message when the file cannot be created.
    fn save_resource_file(&self, resource_path: &AsciiString) -> Result<bool, Failure> {
        let text: AsciiString = self.save();
        write_resource_file(resource_path, &text, true)
    }
    /// `IsEnabled()`: the loading status.
    fn is_enabled(&self) -> bool {
        self.configuration_node_base().is_enabled()
    }
    /// `SetEnabled(theIsLoaded)`.
    fn set_enabled(&self, is_loaded: bool) {
        self.configuration_node_base().set_enabled(is_loaded);
    }
    /// The public `GlobalParameters` field.
    fn global_parameters(&self) -> DeSectionGlobal {
        self.configuration_node_base().global_parameters()
    }
    /// Assigns the public `GlobalParameters` field.
    fn set_global_parameters(&self, parameters: DeSectionGlobal) {
        self.configuration_node_base()
            .set_global_parameters(parameters);
    }
}
impl_transient_interface!(
    dyn DeConfigurationNode,
    "DE_ConfigurationNode",
    StandardTransient
);

/// The body of `CheckExtension`.
fn matches_extension(extensions: &List<AsciiString>, extension: &AsciiString) -> bool {
    let mut wanted: AsciiString = extension.clone();
    if wanted.is_empty() {
        return false;
    }
    if wanted.as_bytes().first() == Some(&b'.') {
        // Removing the first byte of a nonempty string cannot fail.
        let _ = wanted.remove(1, 1);
    }
    extensions
        .iter()
        .any(|candidate| candidate.is_same_string(&wanted, false))
}

/// The `OSD_File::Build` + `Write` of `DE_ConfigurationNode::Save(path)` and
/// `DE_Wrapper::Save(path)`; `report` sends the node's failure message (the wrapper sends
/// none).
pub(crate) fn write_resource_file(
    resource_path: &AsciiString,
    text: &AsciiString,
    report: bool,
) -> Result<bool, Failure> {
    let mut file: File = File::with_path(CadPath::new(resource_path.as_bytes()));
    // The native try/catch around Build turns its exceptions into a failed save.
    if file
        .build(OpenMode::ReadWrite, &Protection::default())
        .is_err()
        || file.failed()
    {
        if report {
            Message::send_fail_text(&AsciiString::new(
                "Error: Configuration writing process was stopped. Can't build file.",
            ));
        }
        return Ok(false);
    }
    file.write(text, text.length())?;
    file.close()?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extensions(items: &[&str]) -> List<AsciiString> {
        let mut list: List<AsciiString> = List::default();
        for item in items {
            list.append(AsciiString::new(item));
        }
        list
    }

    #[test]
    fn check_extension_ignores_case_and_one_leading_dot() {
        let list = extensions(&["step", "stp"]);
        assert!(matches_extension(&list, &AsciiString::new(".STP")));
        assert!(matches_extension(&list, &AsciiString::new("Step")));
        assert!(!matches_extension(&list, &AsciiString::new("")));
        assert!(!matches_extension(&list, &AsciiString::new(".")));
        assert!(!matches_extension(&list, &AsciiString::new("..stp")));
    }

    #[test]
    fn a_resource_file_in_a_missing_directory_is_not_written() {
        let text = AsciiString::new("a : 1\n");
        let written = write_resource_file(&AsciiString::new("/no/such/dir/x.txt"), &text, false);
        assert!(!written.expect("no failure"));
    }
}
