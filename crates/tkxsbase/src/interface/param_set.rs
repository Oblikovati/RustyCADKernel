// Rust port of DataExchange/TKXSBase/Interface/Interface_ParamSet (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_ParamSet`: the parameters of the records of a file, numbered from 1, with
//! a reserved count; the parameters beyond it go to a next set.
use super::{InterfaceFileParameter, InterfaceParamList, InterfaceParamType};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
};

/// The fields of a parameter set. The native character buffer (`theval`, `thelnval`,
/// `thelnres`) that the parameters point into is replaced by the parameters' own texts.
struct ParamSetState {
    thenbpar: i32,
    themxpar: i32,
    thelist: Handle<InterfaceParamList>,
    thenext: Handle<InterfaceParamSet>,
}

/// `Interface_ParamSet`.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceParamSet, InterfaceParamType};
/// let set = InterfaceParamSet::new(10, 1);
/// assert_eq!(set.append(b"12", 2, InterfaceParamType::ParamInteger, 0).unwrap(), 1);
/// assert_eq!(set.param(1).unwrap().c_value(), b"12");
/// ```
pub struct InterfaceParamSet {
    transient: StandardTransient,
    state: SharedState<ParamSetState>,
}
impl_transient!(
    InterfaceParamSet,
    "Interface_ParamSet",
    StandardTransient,
    transient
);

impl InterfaceParamSet {
    /// `Interface_ParamSet(nres, nst = 1)`: room for `nres` parameters (`nst` is unused).
    pub fn new(nres: i32, _nst: i32) -> Self {
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(ParamSetState {
                thenbpar: 0,
                themxpar: nres,
                thelist: Handle::new(InterfaceParamList::new(256)),
                thenext: Handle::default(),
            }),
        }
    }
    /// `Append(val, lnval, typ, nument)`: a parameter of the `lnval` first bytes of
    /// `val` (all of them, up to a NUL, for a negative `lnval`); returns its number.
    /// Native quirk kept: beyond the reserved count, each append makes a new next set
    /// (forgetting the former one) and returns the count plus 1.
    pub fn append(
        &self,
        val: &[u8],
        lnval: i32,
        typ: InterfaceParamType,
        nument: i32,
    ) -> Result<i32, Failure> {
        let (nbpar, list) = {
            let mut state = self.state.write();
            state.thenbpar += 1;
            if state.thenbpar > state.themxpar {
                let next = Handle::new(InterfaceParamSet::new(state.themxpar, 1));
                state.thenext = next.clone();
                let nbpar = state.thenbpar;
                drop(state);
                return Ok(nbpar + next.append(val, lnval, typ, nument)?);
            }
            (state.thenbpar, state.thelist.clone())
        };
        let text = parameter_text(val, lnval);
        live_list(&list)?.change_value(nbpar, |param| {
            param.init_text(text, typ);
            if nument != 0 {
                param.set_entity_number(nument);
            }
        })?;
        Ok(nbpar)
    }
    /// `Append(FP)`: the same overflow quirk as the other form.
    pub fn append_parameter(&self, fp: &InterfaceFileParameter) -> Result<i32, Failure> {
        let mut state = self.state.write();
        state.thenbpar += 1;
        if state.thenbpar > state.themxpar {
            let next = Handle::new(InterfaceParamSet::new(state.themxpar, 1));
            state.thenext = next.clone();
            let nbpar = state.thenbpar;
            drop(state);
            return Ok(nbpar + next.append_parameter(fp)?);
        }
        live_list(&state.thelist)?.set_value(state.thenbpar, fp)?;
        Ok(state.thenbpar)
    }
    /// `NbParams`.
    pub fn nb_params(&self) -> i32 {
        self.state.read().thenbpar
    }
    /// `Param(num)`: beyond the reserved count, in the next set.
    pub fn param(&self, num: i32) -> Result<InterfaceFileParameter, Failure> {
        let (mxpar, list, next) = self.parts();
        if num > mxpar {
            return live_next(&next)?.param(num - mxpar);
        }
        live_list(&list)?.value(num)
    }
    /// `ChangeParam(num)`: edits a parameter (natively a modifiable reference).
    pub fn change_param<R>(
        &self,
        num: i32,
        edit: impl FnOnce(&mut InterfaceFileParameter) -> R,
    ) -> Result<R, Failure> {
        let (mxpar, list, next) = self.parts();
        if num > mxpar {
            return live_next(&next)?.change_param(num - mxpar, edit);
        }
        live_list(&list)?.change_value(num, edit)
    }
    /// `SetParam(num, FP)`.
    pub fn set_param(&self, num: i32, fp: &InterfaceFileParameter) -> Result<(), Failure> {
        let (mxpar, list, next) = self.parts();
        if num > mxpar {
            return live_next(&next)?.set_param(num - mxpar, fp);
        }
        live_list(&list)?.set_value(num, fp)
    }
    /// `Params(num, nb)`: the `nb` parameters from `num`; for (0, 0), all of them (the
    /// list itself when they fit the reserved count).
    pub fn params(&self, num: i32, nb: i32) -> Result<Handle<InterfaceParamList>, Failure> {
        let (mxpar, list, next) = self.parts();
        if num > mxpar {
            return live_next(&next)?.params(num - mxpar, nb);
        }
        let (mut n0, mut nbp) = (num - 1, nb);
        if num == 0 && nb == 0 {
            n0 = 0;
            nbp = self.nb_params();
            if nbp <= mxpar {
                return Ok(list); // and there you go
            }
        }
        let result = InterfaceParamList::new(256);
        if nb == 0 {
            return Ok(Handle::new(result));
        }
        for i in 1..=nbp {
            result.set_value(i, &self.param(n0 + i)?)?;
        }
        Ok(Handle::new(result))
    }
    /// `Destroy`: forgets the next set and the parameters.
    pub fn destroy(&self) {
        let mut state = self.state.write();
        state.thenext.nullify();
        if let Some(list) = state.thelist.get() {
            list.clear();
        }
        state.thelist.nullify();
    }
    fn parts(&self) -> (i32, Handle<InterfaceParamList>, Handle<InterfaceParamSet>) {
        let state = self.state.read();
        (state.themxpar, state.thelist.clone(), state.thenext.clone())
    }
}

/// The text of a new parameter: `lnval` bytes, or up to the NUL for a negative length.
fn parameter_text(val: &[u8], lnval: i32) -> &[u8] {
    let length = match usize::try_from(lnval) {
        Ok(n) => n.min(val.len()),
        Err(_) => val.iter().position(|b| *b == 0).unwrap_or(val.len()),
    };
    &val[..length]
}

/// The list of the parameters (null after `Destroy`, dereferenced natively).
fn live_list(list: &Handle<InterfaceParamList>) -> Result<&InterfaceParamList, Failure> {
    list.get().ok_or_else(|| {
        exceptions::null_object(
            "Interface_ParamSet: the parameter list is null after Destroy; expected a live set",
        )
    })
}

/// The next set, dereferenced natively without a check.
fn live_next(next: &Handle<InterfaceParamSet>) -> Result<&InterfaceParamSet, Failure> {
    next.get().ok_or_else(|| {
        exceptions::null_object("Interface_ParamSet: no next set; expected a parameter number within the reserved count")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_overflow_and_lists() {
        let set = InterfaceParamSet::new(2, 1);
        set.append(b"a\0zz", -1, InterfaceParamType::ParamText, 0)
            .unwrap();
        set.append(b"#5", 2, InterfaceParamType::ParamIdent, 5)
            .unwrap();
        assert_eq!(set.param(2).unwrap().entity_number(), 5);
        assert_eq!(set.params(0, 0).unwrap().length(), 2);
        assert_eq!(set.params(2, 1).unwrap().value(1).unwrap().c_value(), b"#5");
        assert_eq!(set.params(1, 0).unwrap().length(), 0);
        // Beyond the reserved count, the count plus 1 is returned and a new next set
        // holds the parameter (native quirk).
        assert_eq!(
            set.append(b"x", 1, InterfaceParamType::ParamText, 0)
                .unwrap(),
            4
        );
        assert_eq!(set.param(3).unwrap().c_value(), b"x");
        assert_eq!(
            set.append_parameter(&InterfaceFileParameter::new())
                .unwrap(),
            5
        );
        set.set_param(1, &InterfaceFileParameter::new()).unwrap();
        set.change_param(2, |p| p.set_entity_number(9)).unwrap();
        assert_eq!(set.param(2).unwrap().entity_number(), 9);
        assert_eq!(set.param(1).unwrap().c_value(), b"");
        set.destroy();
        assert!(set.param(1).is_err());
    }
}
