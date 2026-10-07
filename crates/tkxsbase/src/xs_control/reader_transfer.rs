// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Reader (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the virtual members and the transfers.

//! [`XSControlReaderOps`]: the virtual members of `XSControl_Reader` (`ReadFile`,
//! `ReadStream`, `NbRootsForTransfer`, the default shape processing) and the members
//! that call them (the roots for transfer and the transfers).
use super::XSControlReader;
use super::reader::live_reader;
use crate::if_select::{IFSelectReturnStatus, IFSelectTransientList};
use crate::interface::InterfaceShareFlags;
use crate::xs_algo::DEShapeFixParameters;
use rustycad_tkernel::{
    message::{ProgressRange, ProgressScope},
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use rustycad_tkshhealing::shape_process::ShapeProcessOperationsFlags;
use std::io::Read;

/// `XSControl_Reader` seen through its virtual members: a norm reader (natively a
/// subclass) overrides them; the other default methods are the non-virtual members that
/// call them (do not override those). [`XSControlReader`] implements it with the native
/// bodies.
///
/// ```
/// use rustycad_tkxsbase::xs_control::{XSControlReader, XSControlReaderOps};
/// let mut reader = XSControlReader::new();
/// assert_eq!(reader.transfer_list(None, &Default::default()).unwrap(), 0);
/// // Without a model the session has no graph: Standard_DomainError, as natively.
/// assert!(reader.transfer_roots(&Default::default()).is_err());
/// ```
pub trait XSControlReaderOps {
    /// The embedded reader.
    fn xs_reader(&self) -> &XSControlReader;
    /// The embedded reader, modifiable.
    fn xs_reader_mut(&mut self) -> &mut XSControlReader;
    /// `ReadFile(filename)`: the file read in the session, then the transfer begun.
    fn read_file(&mut self, filename: &str) -> Result<IFSelectReturnStatus, Failure> {
        let ws = self.xs_reader().session()?;
        let stat = ws.read_file(filename)?;
        ws.init_transfer_reader(4)?;
        Ok(stat)
    }
    /// `ReadStream(theName, theIStream)`: as `ReadFile`, from a stream.
    fn read_stream(
        &mut self,
        name: &str,
        istream: &mut dyn Read,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let ws = self.xs_reader().session()?;
        let stat = ws.read_stream(name, istream)?;
        ws.init_transfer_reader(4)?;
        Ok(stat)
    }
    /// `NbRootsForTransfer`: computed once, the roots of the graph that the transfer
    /// reader recognizes.
    fn nb_roots_for_transfer(&mut self) -> Result<i32, Failure> {
        if self.xs_reader().therootsta {
            return Ok(self.xs_reader().theroots.len() as i32);
        }
        self.xs_reader_mut().therootsta = true;
        let roots = recognized_roots(self.xs_reader())?;
        self.xs_reader_mut().theroots.extend(roots);
        Ok(self.xs_reader().theroots.len() as i32)
    }
    /// `GetDefaultShapeFixParameters` (protected): the defaults of the norm (empty by
    /// default).
    fn default_shape_fix_parameters(&self) -> DEShapeFixParameters {
        DEShapeFixParameters::default()
    }
    /// `GetDefaultShapeProcessFlags` (protected): the defaults of the norm (none by
    /// default).
    fn default_shape_process_flags(&self) -> ShapeProcessOperationsFlags {
        ShapeProcessOperationsFlags::default()
    }
    /// `GiveList(first = "", second = "")`: the list the texts give in the session, or
    /// the roots for transfer for an empty `first`.
    fn give_list(&mut self, first: &str, second: &str) -> Result<IFSelectTransientList, Failure> {
        if !first.is_empty() {
            return self.xs_reader().session()?.give_list_texts(first, second);
        }
        let nbr = self.nb_roots_for_transfer()?;
        let roots = (1..=nbr)
            .map(|i| self.root_for_transfer(i))
            .collect::<Result<Vec<_>, Failure>>()?;
        Ok(Some(roots))
    }
    /// `RootForTransfer(num = 1)`: null out of the roots.
    fn root_for_transfer(&mut self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        let nbr = self.nb_roots_for_transfer()?;
        if num < 1 || num > nbr {
            return Ok(Handle::default());
        }
        Ok(self.xs_reader().theroots[(num - 1) as usize].clone())
    }
    /// `TransferOneRoot(num = 1, theProgress)`: the root `num` transferred.
    fn transfer_one_root(
        &mut self,
        num: i32,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let root = self.root_for_transfer(num)?;
        self.transfer_entity(&root, progress)
    }
    /// `TransferOne(num, theProgress)`: the entity `num` of the model transferred.
    fn transfer_one(&mut self, num: i32, progress: &ProgressRange<'_>) -> Result<bool, Failure> {
        let start = self.xs_reader().session()?.starting_entity(num);
        self.transfer_entity(&start, progress)
    }
    /// `TransferEntity(theStart, theProgress)`: the entity transferred (the transfer
    /// begun, the missing parameters set), its shape result added; false for null or
    /// without result.
    fn transfer_entity(
        &mut self,
        start: &Handle<StandardTransient>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if start.is_null() {
            return Ok(false);
        }
        let tr = begin_reading(self)?;
        if tr.transfer_one(start, true, progress)? == 0 {
            return Ok(false);
        }
        let shape = tr.shape_result(start)?;
        self.xs_reader_mut().shapes().push(shape);
        Ok(true)
    }
    /// `TransferList(theList, theProgress)`: the shapes cleared, then each entity
    /// transferred and its shape result added; the count of the transferred entities (0
    /// for `None`).
    fn transfer_list(
        &mut self,
        list: Option<&[Handle<StandardTransient>]>,
        progress: &ProgressRange<'_>,
    ) -> Result<i32, Failure> {
        let Some(list) = list else {
            return Ok(0);
        };
        let scope = ProgressScope::unnamed(progress, list.len() as f64, false);
        transfer_starts(self, list, &scope)
    }
    /// `TransferRoots(theProgress)`: the shapes cleared, then each root for transfer
    /// transferred and its shape result added; the count of the transferred roots.
    fn transfer_roots(&mut self, progress: &ProgressRange<'_>) -> Result<i32, Failure> {
        self.nb_roots_for_transfer()?;
        let roots = self.xs_reader().theroots.clone();
        let scope = ProgressScope::new(
            progress,
            &AsciiString::new("Root"),
            roots.len() as f64,
            false,
        );
        transfer_starts(self, &roots, &scope)
    }
}

impl XSControlReaderOps for XSControlReader {
    fn xs_reader(&self) -> &XSControlReader {
        self
    }
    fn xs_reader_mut(&mut self) -> &mut XSControlReader {
        self
    }
}

/// The roots of the graph of the session that the transfer reader recognizes.
fn recognized_roots(reader: &XSControlReader) -> Result<Vec<Handle<StandardTransient>>, Failure> {
    let ws = reader.session()?;
    let hgraph = ws.graph()?;
    let graph = hgraph
        .get()
        .ok_or_else(|| exceptions::null_object("XSControl_Reader::NbRootsForTransfer: no graph"))?;
    let sf = InterfaceShareFlags::from_graph(&graph.graph())?;
    let tr = live_reader(ws)?;
    let mut roots = Vec::new();
    for i in 1..=sf.nb_roots() {
        let start = sf.root(i)?;
        if tr.recognize(&start) {
            roots.push(start);
        }
    }
    Ok(roots)
}

/// The transfer begun and the missing shape processing parameters set
/// (`InitializeMissingParameters`, private): the transfer reader.
fn begin_reading<R: XSControlReaderOps + ?Sized>(
    reader: &R,
) -> Result<Handle<super::XSControlTransferReader>, Failure> {
    let tr = live_reader(reader.xs_reader().session()?)?;
    tr.begin_transfer();
    let base = reader.xs_reader();
    if base.shape_fix_parameters().is_empty() {
        base.set_shape_fix_parameters_from(
            &reader.default_shape_fix_parameters(),
            &Default::default(),
        );
    }
    if !base.shape_process_flags().1 {
        base.set_shape_process_flags(reader.default_shape_process_flags());
    }
    Ok(tr)
}

/// The shapes cleared, then each start transferred on a step of `scope` and its shape
/// result added; the count of the transferred starts.
fn transfer_starts<R: XSControlReaderOps + ?Sized>(
    reader: &mut R,
    starts: &[Handle<StandardTransient>],
    scope: &ProgressScope<'_>,
) -> Result<i32, Failure> {
    let tr = begin_reading(reader)?;
    reader.xs_reader_mut().clear_shapes();
    let mut count = 0;
    for start in starts {
        if !scope.more() {
            break;
        }
        if tr.transfer_one(start, true, &scope.next(1.0))? == 0 {
            continue;
        }
        let shape = tr.shape_result(start)?;
        reader.xs_reader_mut().shapes().push(shape);
        count += 1;
    }
    Ok(count)
}
