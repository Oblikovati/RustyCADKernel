// Rust port of DataExchange/TKXSBase/Interface/Interface_FileReaderData (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_FileReaderData`: the records of a file being read, with their parameters
//! and the entities bound to them. The non-virtual members are the extension trait
//! [`InterfaceFileReaderDataParams`].
//!
//! The native `Param` and `ChangeParam` keep a process-wide cache of the first parameter
//! rank of the last record read (valid for the last data created); the port computes the
//! rank each time, which gives the same parameter.
use super::{InterfaceFileParameter, InterfaceParamList, InterfaceParamSet, InterfaceParamType};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{
        Failure, Handle, SharedState, StandardTransient, Transient, c_string::atof_bytes,
        exceptions,
    },
};

/// The private fields of `Interface_FileReaderData`.
struct ReaderDataState {
    therrload: i32,
    theparams: Handle<InterfaceParamSet>,
    thenumpar: Vec<i32>,
    theents: Vec<Handle<StandardTransient>>,
}

/// The embedded base of a file reader data; `InterfaceFileReaderDataBase::new(nbr, npar)`
/// is the native protected constructor (records 0..nbr, `npar` parameters reserved).
pub struct InterfaceFileReaderDataBase {
    state: SharedState<ReaderDataState>,
}

impl InterfaceFileReaderDataBase {
    /// `Interface_FileReaderData(nbr, npar)`.
    pub fn new(nbr: i32, npar: i32) -> Self {
        let places = usize::try_from(nbr + 1).unwrap_or(0);
        Self {
            state: SharedState::new(ReaderDataState {
                therrload: 0,
                theparams: Handle::new(InterfaceParamSet::new(npar, 1)),
                thenumpar: vec![0; places],
                theents: vec![Handle::default(); places],
            }),
        }
    }
    /// `Fastof(str)`: `Strtod` of the text (the numeric prefix, 0 when none).
    pub fn fastof(text: &[u8]) -> f64 {
        atof_bytes(text)
    }
}

/// `Interface_FileReaderData`, abstract: `FindNextRecord` is the required method.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceFileReaderDataBase;
/// assert_eq!(InterfaceFileReaderDataBase::fastof(b"2.5E1 "), 25.0);
/// ```
pub trait InterfaceFileReaderData: Transient {
    /// The embedded private fields.
    fn reader_data_base(&self) -> &InterfaceFileReaderDataBase;
    /// `NbRecords`: the upper record number.
    fn nb_records(&self) -> i32 {
        self.reader_data_base().state.read().thenumpar.len() as i32 - 1
    }
    /// `NbEntities`: the count of records `FindNextRecord` gives.
    fn nb_entities(&self) -> i32 {
        let mut nb = 0;
        let mut num = self.find_next_record(0);
        while num > 0 {
            nb += 1;
            num = self.find_next_record(num);
        }
        nb
    }
    /// `FindNextRecord(num)`: the record after `num` that starts an entity (0 when none).
    fn find_next_record(&self, num: i32) -> i32;
}
impl_transient_interface!(
    dyn InterfaceFileReaderData,
    "Interface_FileReaderData",
    StandardTransient
);

fn out_of_records(num: i32, count: usize) -> Failure {
    exceptions::out_of_range(&format!(
        "Interface_FileReaderData: record {num}; expected [0, {}]",
        count as i32 - 1
    ))
}

/// The non-virtual members of `Interface_FileReaderData`.
pub trait InterfaceFileReaderDataParams: InterfaceFileReaderData {
    /// `InitParams(num)`: the parameters added so far end at record `num`.
    fn init_params(&self, num: i32) -> Result<(), Failure> {
        let mut state = self.reader_data_base().state.write();
        let count = state.theparams.nb_params();
        let len = state.thenumpar.len();
        let slot = usize::try_from(num)
            .ok()
            .and_then(|i| state.thenumpar.get_mut(i));
        *slot.ok_or_else(|| out_of_records(num, len))? = count;
        Ok(())
    }
    /// `AddParam(num, aval, atype, nument = 0)` (the record number is not used).
    fn add_param(
        &self,
        _num: i32,
        aval: &[u8],
        atype: InterfaceParamType,
        nument: i32,
    ) -> Result<(), Failure> {
        self.param_set().append(aval, -1, atype, nument).map(|_| ())
    }
    /// `AddParam(num, const TCollection_AsciiString&, atype, nument = 0)`.
    fn add_param_string(
        &self,
        _num: i32,
        aval: &rustycad_tkernel::tcollection::AsciiString,
        atype: InterfaceParamType,
        nument: i32,
    ) -> Result<(), Failure> {
        self.param_set()
            .append(aval.as_bytes(), aval.length(), atype, nument)
            .map(|_| ())
    }
    /// `AddParam(num, FP)`.
    fn add_param_parameter(&self, _num: i32, fp: &InterfaceFileParameter) -> Result<(), Failure> {
        self.param_set().append_parameter(fp).map(|_| ())
    }
    /// `SetParam(num, nump, FP)`.
    fn set_param(&self, num: i32, nump: i32, fp: &InterfaceFileParameter) -> Result<(), Failure> {
        let rank = self.first_rank_before(num)? + nump;
        self.param_set().set_param(rank, fp)
    }
    /// `NbParams(num)`: of record `num`; of the whole file for 0.
    fn nb_params(&self, num: i32) -> Result<i32, Failure> {
        if num > 1 {
            return Ok(self.param_first_rank(num)? - self.param_first_rank(num - 1)?);
        }
        if num == 1 {
            return self.param_first_rank(num);
        }
        Ok(self.param_set().nb_params())
    }
    /// `Params(num)`: the parameters of record `num`, of the whole file for 0.
    fn params(&self, num: i32) -> Result<Handle<InterfaceParamList>, Failure> {
        let set = self.param_set();
        match num {
            0 => set.params(0, 0),
            1 => set.params(0, self.param_first_rank(1)?),
            _ => {
                let before = self.param_first_rank(num - 1)?;
                set.params(before + 1, self.param_first_rank(num)? - before)
            }
        }
    }
    /// `Param(num, nump)`: parameter `nump` of record `num`.
    fn param(&self, num: i32, nump: i32) -> Result<InterfaceFileParameter, Failure> {
        let rank = self.first_rank_before(num)? + nump;
        self.param_set().param(rank)
    }
    /// `ChangeParam(num, nump)`: edits a parameter (natively a modifiable reference).
    fn change_param<R>(
        &self,
        num: i32,
        nump: i32,
        edit: impl FnOnce(&mut InterfaceFileParameter) -> R,
    ) -> Result<R, Failure> {
        let rank = self.first_rank_before(num)? + nump;
        self.param_set().change_param(rank, edit)
    }
    /// `ParamType(num, nump)`.
    fn param_type(&self, num: i32, nump: i32) -> Result<InterfaceParamType, Failure> {
        Ok(self.param(num, nump)?.param_type())
    }
    /// `ParamCValue(num, nump)`.
    fn param_c_value(&self, num: i32, nump: i32) -> Result<Vec<u8>, Failure> {
        Ok(self.param(num, nump)?.c_value().to_vec())
    }
    /// `IsParamDefined(num, nump)`: its type is not void.
    fn is_param_defined(&self, num: i32, nump: i32) -> Result<bool, Failure> {
        Ok(self.param_type(num, nump)? != InterfaceParamType::ParamVoid)
    }
    /// `ParamNumber(num, nump)`: the entity number of the parameter.
    fn param_number(&self, num: i32, nump: i32) -> Result<i32, Failure> {
        Ok(self.param(num, nump)?.entity_number())
    }
    /// `ParamEntity(num, nump)`: the entity bound to the record the parameter designates.
    fn param_entity(&self, num: i32, nump: i32) -> Result<Handle<StandardTransient>, Failure> {
        Ok(self.bound_entity(self.param_number(num, nump)?))
    }
    /// `ParamFirstRank(num)`: the rank of the last parameter of record `num`.
    fn param_first_rank(&self, num: i32) -> Result<i32, Failure> {
        let state = self.reader_data_base().state.read();
        let found = usize::try_from(num)
            .ok()
            .and_then(|i| state.thenumpar.get(i));
        found
            .copied()
            .ok_or_else(|| out_of_records(num, state.thenumpar.len()))
    }
    /// `BoundEntity(num)`: null out of the records.
    fn bound_entity(&self, num: i32) -> Handle<StandardTransient> {
        let state = self.reader_data_base().state.read();
        let found = usize::try_from(num).ok().and_then(|i| state.theents.get(i));
        found.cloned().unwrap_or_default()
    }
    /// `BindEntity(num, ent)`: `Standard_OutOfRange` out of the records.
    fn bind_entity(&self, num: i32, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        let mut state = self.reader_data_base().state.write();
        let len = state.theents.len();
        let slot = usize::try_from(num)
            .ok()
            .and_then(|i| state.theents.get_mut(i));
        *slot.ok_or_else(|| out_of_records(num, len))? = ent.clone();
        Ok(())
    }
    /// `SetErrorLoad(val)`.
    fn set_error_load(&self, val: bool) {
        self.reader_data_base().state.write().therrload = if val { 1 } else { -1 };
    }
    /// `IsErrorLoad`: an error state was set (true or false).
    fn is_error_load(&self) -> bool {
        self.reader_data_base().state.read().therrload != 0
    }
    /// `ResetErrorLoad`: whether the state was "error", then no state.
    fn reset_error_load(&self) -> bool {
        let mut state = self.reader_data_base().state.write();
        let res = state.therrload > 0;
        state.therrload = 0;
        res
    }
    /// `Destroy`: nothing (the native destructor's work).
    fn destroy(&self) {}
    /// `ChangeParameter(numpar)` (protected): edits the parameter of a global rank.
    fn change_parameter<R>(
        &self,
        numpar: i32,
        edit: impl FnOnce(&mut InterfaceFileParameter) -> R,
    ) -> Result<R, Failure> {
        self.param_set().change_param(numpar, edit)
    }
    /// `ParamPosition(numpar, num, nump)` (protected): the record and rank of a global
    /// parameter rank, as (num, nump).
    fn param_position(&self, numpar: i32) -> (i32, i32) {
        let state = self.reader_data_base().state.read();
        let nbe = state.thenumpar.len() as i32 - 1;
        if numpar <= 0 {
            return (0, 0);
        }
        for i in 1..=nbe {
            let first = state.thenumpar[i as usize];
            if first > numpar {
                return (i, numpar - first + 1);
            }
        }
        (
            nbe,
            numpar
                - state
                    .thenumpar
                    .get(nbe.max(0) as usize)
                    .copied()
                    .unwrap_or(0)
                + 1,
        )
    }
    /// The rank before the first parameter of record `num` (`thenumpar(num - 1)`).
    fn first_rank_before(&self, num: i32) -> Result<i32, Failure> {
        self.param_first_rank(num - 1)
    }
    /// The parameter set.
    fn param_set(&self) -> Handle<InterfaceParamSet> {
        self.reader_data_base().state.read().theparams.clone()
    }
}

impl<T: InterfaceFileReaderData + ?Sized> InterfaceFileReaderDataParams for T {}
