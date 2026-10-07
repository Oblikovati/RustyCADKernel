// Rust port of TDocStd_MultiTransactionManager::DumpTransaction
// (ApplicationFramework/TKLCAF/TDocStd/TDocStd_MultiTransactionManager.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text dump of a multi-transaction manager.
use super::TDocStdMultiTransactionManager;
use rustycad_tkernel::standard::Handle;
use std::io;

/// `DumpTransaction`: the documents and the mode, then the undos (oldest first, the last
/// action marked) and the redos.
pub(crate) fn dump_transaction(
    manager: &TDocStdMultiTransactionManager,
    stream: &mut dyn io::Write,
) -> io::Result<()> {
    let (documents, nested, undos, redos) = manager.dump_state();
    if documents.is_empty() {
        writeln!(stream, "Manager is empty")?;
    } else {
        if documents.len() == 1 {
            write!(stream, "There is one document ( ")?;
        } else {
            write!(stream, "There are {} documents ( ", documents.len())?;
        }
        for document in &documents {
            write!(stream, "\"{:p}\" ", document.object_address())?;
        }
        writeln!(stream, ") in the manager ")?;
        writeln!(
            stream,
            "Nested transaction mode is {}",
            if nested { "on" } else { "off" }
        )?;
        writeln!(stream, " ")?;
    }
    for (index, delta) in undos.iter().enumerate().rev() {
        write!(stream, " Undo: ")?;
        if let Some(delta) = delta.get() {
            delta.dump(stream)?;
        }
        if index == 0 {
            writeln!(stream, "  < Last action")?;
        } else {
            writeln!(stream)?;
        }
    }
    for delta in redos.iter().filter_map(Handle::get) {
        write!(stream, " Redo: ")?;
        delta.dump(stream)?;
        writeln!(stream)?;
    }
    Ok(())
}
