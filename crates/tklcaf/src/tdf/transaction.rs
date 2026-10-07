// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Transaction.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_Transaction`: opens, commits or aborts one transaction of a framework.
use super::{TdfData, TdfDelta, data_commit};
use crate::live;
use rustycad_tkcdf::json_dump::{class_begin, field_dumped, field_integer, field_string};
use rustycad_tkernel::{
    standard::{Failure, Handle, dump::DumpStream, exceptions},
    tcollection::AsciiString,
};
use std::io;

/// `TDF_Transaction`: not copyable; dropping an open transaction aborts it.
///
/// ```
/// use rustycad_tklcaf::tdf::{TdfData, TdfTransaction};
/// use rustycad_tkernel::tcollection::AsciiString;
/// let data = TdfData::new();
/// let mut transaction = TdfTransaction::with_data(&data, &AsciiString::new("t"));
/// assert_eq!(transaction.open().unwrap(), 1);
/// transaction.commit(false).unwrap();
/// assert!(!transaction.is_open() && data.transaction() == 0);
/// ```
pub struct TdfTransaction {
    data: Handle<TdfData>,
    name: AsciiString,
    until_transaction: i32,
}

impl TdfTransaction {
    /// `TDF_Transaction(aName)`: no framework yet.
    pub fn new(name: &AsciiString) -> Self {
        Self::with_data(&Handle::default(), name)
    }
    /// `TDF_Transaction(aDF, aName)`.
    pub fn with_data(data: &Handle<TdfData>, name: &AsciiString) -> Self {
        Self {
            data: data.clone(),
            name: name.clone(),
            until_transaction: 0,
        }
    }
    /// `Initialize(aDF)`: aborts an open transaction, then targets another framework.
    pub fn initialize(&mut self, data: &Handle<TdfData>) -> Result<(), Failure> {
        if self.is_open() {
            data_commit::abort_until_transaction(
                &live(&self.data, "TDF_Transaction")?.core,
                self.until_transaction,
            )?;
        }
        self.data = data.clone();
        self.until_transaction = 0;
        Ok(())
    }
    /// `Open`: the new transaction number; `Standard_DomainError` when already open,
    /// `Standard_NullObject` without framework.
    pub fn open(&mut self) -> Result<i32, Failure> {
        if self.is_open() {
            return Err(exceptions::domain_error(&format!(
                "This transaction is already open (until transaction {}).",
                self.until_transaction
            )));
        }
        let data = self
            .data
            .get()
            .ok_or_else(|| exceptions::null_object("Null TDF_Data."))?;
        self.until_transaction = data_commit::open_transaction(&data.core);
        Ok(self.until_transaction)
    }
    /// `Commit(withDelta)`: commits down to this transaction; null when not open.
    pub fn commit(&mut self, with_delta: bool) -> Result<Handle<TdfDelta>, Failure> {
        if !self.is_open() {
            return Ok(Handle::default());
        }
        let until = self.until_transaction;
        self.until_transaction = 0;
        let data = live(&self.data, "TDF_Transaction::Commit")?;
        data_commit::commit_until_transaction(&data.core, until, with_delta)
    }
    /// `Abort`: undoes down to this transaction, when open.
    pub fn abort(&mut self) -> Result<(), Failure> {
        if !self.is_open() {
            return Ok(());
        }
        let data = live(&self.data, "TDF_Transaction::Abort")?;
        data_commit::abort_until_transaction(&data.core, self.until_transaction)?;
        self.until_transaction = 0;
        Ok(())
    }
    /// `Data`; `transaction.data()`.
    pub fn data(&self) -> Handle<TdfData> {
        self.data.clone()
    }
    /// `Transaction`: the number this transaction opened, 0 when closed.
    pub fn transaction(&self) -> i32 {
        self.until_transaction
    }
    /// `Name`; `transaction.name()`.
    pub fn name(&self) -> &AsciiString {
        &self.name
    }
    /// `IsOpen`: opened and not yet committed or aborted.
    pub fn is_open(&self) -> bool {
        self.until_transaction > 0
    }
    /// `DumpJson`: the framework, the transaction number and the name.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDF_Transaction")?;
        if let Some(data) = self.data.get() {
            let child = |s: &mut DumpStream, d: i32| data.dump_json(s, d);
            field_dumped(stream, depth, "myDF.get()", Some(&child))?;
        }
        field_integer(stream, "myUntilTransaction", self.until_transaction)?;
        field_string(
            stream,
            "myName",
            &String::from_utf8_lossy(self.name.as_bytes()),
        )
    }
}

impl Drop for TdfTransaction {
    /// `~TDF_Transaction`: `Abort()`; a failure cannot leave a destructor (section 5).
    fn drop(&mut self) {
        let _ = self.abort();
    }
}
