// Rust port of ApplicationFramework/TKTObj/TObj/TObj_TObj_msg.pxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The message resources of the TObj package (`TObj_TObj_msg`), loaded into the process
//! message catalog by the first `TObj_Application`.
use rustycad_tkernel::{
    message::{Msg, MsgFile},
    standard::{Failure, exceptions},
    tcollection::{AsciiString, ExtendedString},
};

/// The native resource text, line for line.
const TOBJ_MESSAGES: &str = concat!(
    "!!!!!!!!!! ----- Messages file for TObj packages --------- !!!!!!!!!\n",
    "!  Syntax for keywords: NameOfPackage_NameOfClass_NameOfMessage\n",
    "!                 where NameOfClass is optional or abbreviated\n",
    "!!!!!!!!!! --------------------------------------------------- !!!!!!!!!\n",
    "\n",
    ".TObj_Appl_SDriverFailure\n",
    "  Error saving document %s : driver failure. Check presence of resource files.\n",
    "\n",
    ".TObj_Appl_SWriteFailure\n",
    "  Error saving document %s : write failure\n",
    "  \n",
    ".TObj_Appl_SFailure\n",
    "  Error saving document %s : general failure of persistence driver\n",
    "  \n",
    ".TObj_Appl_SDocIsNull\n",
    "  Error saving document %s : No document to save\n",
    "  \n",
    ".TObj_Appl_SNoObj\n",
    " Error saving document %s : No objects written\n",
    " \n",
    ".TObj_Appl_SInfoSectionError\n",
    " Error saving document %s : Write info section failure\n",
    "\n",
    ".TObj_Appl_SUnknownFailure\n",
    "  Error saving document %s : unknown failure\n",
    "\n",
    ".TObj_Appl_RUnknownDocument\n",
    "  Error loading document %s : unknown document\n",
    "\n",
    ".TObj_Appl_RAlreadyRetrieved\n",
    "  Error loading document %s : already retrieved\n",
    "\n",
    ".TObj_Appl_RAlreadyRetrievedAndModified\n",
    "  Error loading document %s : already retrieved and modified\n",
    "\n",
    ".TObj_Appl_RNoDriver\n",
    "  Error loading document %s : no appropriate driver was found\n",
    "\n",
    ".TObj_Appl_ROpenError\n",
    "  Error loading document %s : cannot open file\n",
    "\n",
    ".TObj_Appl_RNoVersion\n",
    "  Error loading document %s : no version\n",
    "\n",
    ".TObj_Appl_RNoModel\n",
    "  Error loading document %s : no model\n",
    "\n",
    ".TObj_Appl_RNoDocument\n",
    "  Error loading document %s : no document\n",
    "\n",
    ".TObj_Appl_RFormatFailure\n",
    "  Error loading document %s : format failure\n",
    "\n",
    ".TObj_Appl_RTypeNotFound\n",
    "  Error loading document %s : type not found in schema\n",
    "\n",
    ".TObj_Appl_RBadFileFormat\n",
    "  Error loading document %s : unrecognized file format\n",
    "\n",
    ".TObj_Appl_RMakeFailure\n",
    "  Error loading document %s : failure making document\n",
    "\n",
    ".TObj_Appl_RPermissionDenied\n",
    "  Error loading document %s : permission denied\n",
    "\n",
    ".TObj_Appl_RDriverFailure\n",
    "  Error loading document %s : driver failure\n",
    "\n",
    ".TObj_Appl_RUnknownFail\n",
    "  Error loading document %s : unknown failure\n",
    "\n",
    ".TObj_Appl_RException\n",
    "  Error loading document %s : the file is probably corrupted\n",
    "\n",
    ".TObj_Appl_Exception\n",
    "  An exception was caught: %s\n",
    "\n",
    ".TObj_M_LoadDocument\n",
    "  +++++ Load document %s\n",
    "\n",
    ".TObj_M_WrongFile\n",
    "  File %s has wrong content\n",
    "\n",
    ".TObj_M_NoWriteAccess\n",
    "  Cannot write the file %s, check permissions\n",
);

/// The key the constructor of `TObj_Application` probes.
const PROBE_KEY: &str = "TObj_Appl_SUnknownFailure";

/// Loads the TObj messages into the process catalog once; `Standard_ProgramError` when
/// they cannot be loaded (the native constructor's check).
pub(crate) fn load_tobj_messages() -> Result<(), Failure> {
    let catalog = MsgFile::global();
    let key = AsciiString::new(PROBE_KEY);
    if catalog.has_msg(&key) {
        return Ok(());
    }
    catalog.load_from_string(TOBJ_MESSAGES.as_bytes(), TOBJ_MESSAGES.len() as i32)?;
    if !catalog.has_msg(&key) {
        return Err(exceptions::program_error(
            "Critical Error - message resources for TObj_Application are invalid or undefined!",
        ));
    }
    Ok(())
}

/// `Message_Msg(key) << argument`, converted to its text (`Message_Msg::Get`).
pub(crate) fn message_with(
    key: &str,
    argument: &ExtendedString,
) -> Result<ExtendedString, Failure> {
    let mut message = Msg::from_key(&AsciiString::new(key));
    message.arg_extended(argument)?;
    Ok(message.get()?.clone())
}

/// `Message_Msg(key)` without argument, converted to its text (`Message_Msg::Get`).
pub(crate) fn message_text(key: &str) -> Result<ExtendedString, Failure> {
    let mut message = Msg::from_key(&AsciiString::new(key));
    Ok(message.get()?.clone())
}
