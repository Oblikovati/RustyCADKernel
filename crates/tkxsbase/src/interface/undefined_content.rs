// Rust port of DataExchange/TKXSBase/Interface/Interface_UndefinedContent (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_UndefinedContent`: the parameters of an entity the norm does not know, as
//! literals and entities. Each parameter is a descriptor word with the native layout:
//! type in bits 0-4, "entity" flag in bits 5-7, rank (in the literals or the entities)
//! from bit 8.
use super::errors::interface_error;
use super::int_array::{
    InterfaceIntArray, int_at, int_upper, new_int_array, regrown_int_array, set_int,
};
use super::{InterfaceCopyTool, InterfaceEntityList, InterfaceParamType, InterfaceTypedValue};
use crate::moni_tool::MoniToolValueType;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::HAsciiString,
};

const CONTENT_TYPE_FIELD: i32 = 31;
const CONTENT_LOCAL_FIELD: i32 = 7;
const CONTENT_LOCAL_REF: i32 = 1;
const CONTENT_LOCAL_SHIFT: i32 = 5;
const CONTENT_NUMBER_SHIFT: i32 = 8;

/// The fields of an undefined content.
#[derive(Default)]
struct ContentState {
    thenbparams: i32,
    thenbstr: i32,
    theparams: InterfaceIntArray,
    thevalues: Option<Vec<Handle<HAsciiString>>>,
    theentities: InterfaceEntityList,
}

/// `Interface_UndefinedContent`.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceParamType, InterfaceUndefinedContent};
/// use rustycad_tkernel::{standard::Handle, tcollection::HAsciiString};
/// let content = InterfaceUndefinedContent::new();
/// content.add_literal(InterfaceParamType::ParamInteger, &Handle::new(HAsciiString::new("3"))).unwrap();
/// assert_eq!(content.param_value(1).unwrap().string().as_bytes(), b"3");
/// ```
#[derive(Default)]
pub struct InterfaceUndefinedContent {
    transient: StandardTransient,
    state: SharedState<ContentState>,
}
impl_transient!(
    InterfaceUndefinedContent,
    "Interface_UndefinedContent",
    StandardTransient,
    transient
);

fn local_of(desc: i32) -> i32 {
    (desc >> CONTENT_LOCAL_SHIFT) & CONTENT_LOCAL_FIELD
}

fn type_of(desc: i32) -> InterfaceParamType {
    let value = desc & CONTENT_TYPE_FIELD;
    let kinds = [
        MoniToolValueType::ValueMisc,
        MoniToolValueType::ValueInteger,
        MoniToolValueType::ValueReal,
        MoniToolValueType::ValueIdent,
        MoniToolValueType::ValueVoid,
        MoniToolValueType::ValueText,
        MoniToolValueType::ValueEnum,
        MoniToolValueType::ValueLogical,
        MoniToolValueType::ValueSub,
        MoniToolValueType::ValueHexa,
        MoniToolValueType::ValueBinary,
    ];
    let kind = kinds.get(value as usize).copied().unwrap_or_default();
    InterfaceTypedValue::value_type_to_param_type(kind)
}

fn value_slot(
    values: &mut Option<Vec<Handle<HAsciiString>>>,
    rank: i32,
) -> Result<&mut Handle<HAsciiString>, Failure> {
    let list = values.as_mut().ok_or_else(|| {
        exceptions::null_object(
            "Interface_UndefinedContent: no literal array; expected a reserved content",
        )
    })?;
    let len = list.len();
    let slot = usize::try_from(rank - 1).ok().and_then(|i| list.get_mut(i));
    slot.ok_or_else(|| {
        exceptions::out_of_range(&format!(
            "Interface_UndefinedContent: literal {rank}; expected [1, {len}]"
        ))
    })
}

impl InterfaceUndefinedContent {
    /// `Interface_UndefinedContent()`: no parameter.
    pub fn new() -> Self {
        Self::default()
    }
    /// `NbParams`.
    pub fn nb_params(&self) -> i32 {
        self.state.read().thenbparams
    }
    /// `NbLiterals`.
    pub fn nb_literals(&self) -> i32 {
        self.state.read().thenbstr
    }
    /// `ParamData(num, ptype, ent, val)`: (is an entity, type, entity, literal);
    /// `Standard_OutOfRange` out of 1..NbParams.
    pub fn param_data(
        &self,
        num: i32,
    ) -> Result<
        (
            bool,
            InterfaceParamType,
            Handle<StandardTransient>,
            Handle<HAsciiString>,
        ),
        Failure,
    > {
        let mut state = self.state.write();
        if num < 1 || num > state.thenbparams {
            return Err(exceptions::out_of_range(&format!(
                "Interface UndefinedContent : ParamData: {num}; expected [1, {}]",
                state.thenbparams
            )));
        }
        let desc = int_at(&state.theparams, num)?;
        let adr = desc >> CONTENT_NUMBER_SHIFT;
        if local_of(desc) == CONTENT_LOCAL_REF {
            return Ok((
                true,
                type_of(desc),
                state.theentities.value(adr)?,
                Handle::default(),
            ));
        }
        let val = value_slot(&mut state.thevalues, adr)?.clone();
        Ok((false, type_of(desc), Handle::default(), val))
    }
    /// `ParamType(num)`.
    pub fn param_type(&self, num: i32) -> Result<InterfaceParamType, Failure> {
        Ok(type_of(int_at(&self.state.read().theparams, num)?))
    }
    /// `IsParamEntity(num)`.
    pub fn is_param_entity(&self, num: i32) -> Result<bool, Failure> {
        Ok(local_of(int_at(&self.state.read().theparams, num)?) == CONTENT_LOCAL_REF)
    }
    /// `ParamEntity(num)`: `Interface_InterfaceError` for a literal.
    pub fn param_entity(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        let state = self.state.read();
        let desc = int_at(&state.theparams, num)?;
        if local_of(desc) != CONTENT_LOCAL_REF {
            return Err(interface_error(
                "UndefinedContent : Param is not Entity type",
            ));
        }
        state.theentities.value(desc >> CONTENT_NUMBER_SHIFT)
    }
    /// `ParamValue(num)`: `Interface_InterfaceError` for an entity.
    pub fn param_value(&self, num: i32) -> Result<Handle<HAsciiString>, Failure> {
        let mut state = self.state.write();
        let desc = int_at(&state.theparams, num)?;
        if local_of(desc) != 0 {
            return Err(interface_error("UndefinedContent : Param is not literal"));
        }
        Ok(value_slot(&mut state.thevalues, desc >> CONTENT_NUMBER_SHIFT)?.clone())
    }
    /// `Reservate(nb, nblit)`: room for `nb` parameters and `nblit` literals.
    pub fn reservate(&self, nb: i32, nblit: i32) -> Result<(), Failure> {
        let mut state = self.state.write();
        if nb > state.thenbparams {
            if state.theparams.is_null() {
                state.theparams = new_int_array(1, nb)?;
            } else if nb > int_upper(&state.theparams)? {
                let nbnew = (2 * state.thenbparams).max(nb); // reserve a bit more
                state.theparams =
                    regrown_int_array(&state.theparams, 1, nbnew, 1..=state.thenbparams)?;
            }
        }
        if nblit > state.thenbstr {
            let current = state.thevalues.as_ref().map_or(0, |v| v.len() as i32);
            if state.thevalues.is_none() {
                state.thevalues = Some(vec![Handle::default(); nblit as usize]);
            } else if nblit > current {
                let nbnew = (2 * state.thenbstr).max(nblit);
                if let Some(values) = state.thevalues.as_mut() {
                    values.resize(nbnew as usize, Handle::default());
                }
            }
        }
        Ok(())
    }
    /// `AddLiteral(ptype, val)`.
    pub fn add_literal(
        &self,
        ptype: InterfaceParamType,
        val: &Handle<HAsciiString>,
    ) -> Result<(), Failure> {
        let (nbparams, nbstr) = (self.nb_params(), self.nb_literals());
        self.reservate(nbparams + 1, nbstr + 1)?;
        let mut state = self.state.write();
        state.thenbstr += 1;
        state.thenbparams += 1;
        let (rank, num) = (state.thenbstr, state.thenbparams);
        *value_slot(&mut state.thevalues, rank)? = val.clone();
        set_int(
            &state.theparams,
            num,
            ptype as i32 + (rank << CONTENT_NUMBER_SHIFT),
        )
    }
    /// `AddEntity(ptype, ent)`: the rank of the entity is NbParams - NbLiterals.
    pub fn add_entity(
        &self,
        ptype: InterfaceParamType,
        ent: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        self.reservate(self.nb_params() + 1, 0)?;
        let mut state = self.state.write();
        state.theentities.append(ent)?;
        state.thenbparams += 1;
        let desc = ptype as i32
            + (CONTENT_LOCAL_REF << CONTENT_LOCAL_SHIFT)
            + ((state.thenbparams - state.thenbstr) << CONTENT_NUMBER_SHIFT);
        let num = state.thenbparams;
        set_int(&state.theparams, num, desc)
    }
    /// `RemoveParam(num)`: removes the parameter and renumbers the following ones of the
    /// same kind.
    pub fn remove_param(&self, num: i32) -> Result<(), Failure> {
        let mut state = self.state.write();
        let desc = int_at(&state.theparams, num)?;
        let (rang, local) = (desc >> CONTENT_NUMBER_SHIFT, local_of(desc));
        if local == CONTENT_LOCAL_REF {
            state.theentities.remove_rank(rang)?;
        } else {
            remove_literal(&mut state, rang)?;
        }
        for np in num + 1..=state.thenbparams {
            let moved = int_at(&state.theparams, np)?;
            set_int(&state.theparams, np - 1, moved)?;
        }
        let last = state.thenbparams;
        set_int(&state.theparams, last, 0)?;
        state.thenbparams -= 1;
        renumber(&state, |d| {
            local_of(d) == local && (d >> CONTENT_NUMBER_SHIFT) > rang
        })
    }
    /// `SetLiteral(num, ptype, val)`: an entity parameter becomes a new literal.
    pub fn set_literal(
        &self,
        num: i32,
        ptype: InterfaceParamType,
        val: &Handle<HAsciiString>,
    ) -> Result<(), Failure> {
        let desc = int_at(&self.state.read().theparams, num)?;
        let mut rang = desc >> CONTENT_NUMBER_SHIFT;
        if local_of(desc) == CONTENT_LOCAL_REF {
            {
                let mut state = self.state.write();
                state.theentities.remove_rank(rang)?;
                renumber(&state, |d| {
                    local_of(d) == CONTENT_LOCAL_REF && (d >> CONTENT_NUMBER_SHIFT) > rang
                })?;
            }
            self.reservate(self.nb_params(), self.nb_literals() + 1)?;
            let mut state = self.state.write();
            state.thenbstr += 1;
            rang = state.thenbstr;
        }
        let mut state = self.state.write();
        *value_slot(&mut state.thevalues, rang)? = val.clone();
        set_int(
            &state.theparams,
            num,
            ptype as i32 + (rang << CONTENT_NUMBER_SHIFT),
        )
    }
    /// `SetEntity(num, ptype, ent)`: a literal parameter becomes a new entity.
    pub fn set_entity_typed(
        &self,
        num: i32,
        ptype: InterfaceParamType,
        ent: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        let desc = int_at(&state.theparams, num)?;
        let mut rang = desc >> CONTENT_NUMBER_SHIFT;
        if local_of(desc) == CONTENT_LOCAL_REF {
            state.theentities.set_value(rang, ent)?;
        } else {
            shift_literals_down(&mut state, rang)?;
            renumber(&state, |d| {
                local_of(d) == 0 && (d >> CONTENT_NUMBER_SHIFT) > rang
            })?;
            state.thenbstr -= 1;
            rang = state.thenbparams - state.thenbstr;
            state.theentities.append(ent)?;
        }
        let desc = ptype as i32
            + (CONTENT_LOCAL_REF << CONTENT_LOCAL_SHIFT)
            + (rang << CONTENT_NUMBER_SHIFT);
        set_int(&state.theparams, num, desc)
    }
    /// `SetEntity(num, ent)`: `Interface_InterfaceError` for a literal parameter.
    pub fn set_entity(&self, num: i32, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        let mut state = self.state.write();
        let desc = int_at(&state.theparams, num)?;
        if local_of(desc) != CONTENT_LOCAL_REF {
            return Err(interface_error("UndefinedContent : SetEntity"));
        }
        state
            .theentities
            .set_value(desc >> CONTENT_NUMBER_SHIFT, ent)
    }
    /// `EntityList`: the entity parameters (a copy sharing the list, as natively).
    pub fn entity_list(&self) -> InterfaceEntityList {
        self.state.read().theentities.clone()
    }
    /// `GetFromAnother(other, TC)`: the parameters of `other`, its entities copied by `TC`.
    pub fn get_from_another(
        &self,
        other: &InterfaceUndefinedContent,
        tc: &mut InterfaceCopyTool,
    ) -> Result<(), Failure> {
        let nb = other.nb_params();
        {
            let mut state = self.state.write();
            state.theentities.clear();
            state.thevalues = None;
            state.theparams.nullify();
        }
        self.reservate(nb, other.nb_literals())?;
        for i in 1..=nb {
            let (is_entity, ptype, ent, val) = other.param_data(i)?;
            if is_entity {
                let copied = tc.transferred(&ent)?;
                self.add_entity(ptype, &copied)?;
            } else {
                self.add_literal(ptype, &val)?;
            }
        }
        Ok(())
    }
}

/// Removes literal `rang` (the array is shifted down, its last place nulled).
fn remove_literal(state: &mut ContentState, rang: i32) -> Result<(), Failure> {
    shift_literals_down(state, rang)?;
    state.thenbstr -= 1;
    Ok(())
}

fn shift_literals_down(state: &mut ContentState, rang: i32) -> Result<(), Failure> {
    for i in rang + 1..=state.thenbstr {
        let moved = value_slot(&mut state.thevalues, i)?.clone();
        *value_slot(&mut state.thevalues, i - 1)? = moved;
    }
    let last = state.thenbstr;
    *value_slot(&mut state.thevalues, last)? = Handle::default();
    Ok(())
}

/// Decrements the rank of the parameters `select` chooses.
fn renumber(state: &ContentState, select: impl Fn(i32) -> bool) -> Result<(), Failure> {
    for np in 1..=state.thenbparams {
        let desc = int_at(&state.theparams, np)?;
        if select(desc) {
            set_int(&state.theparams, np, desc - (1 << CONTENT_NUMBER_SHIFT))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_norm::{fake_chain, fake_entity};
    use super::*;

    fn text(value: &str) -> Handle<HAsciiString> {
        Handle::new(HAsciiString::new(value))
    }

    #[test]
    fn literals_entities_and_edits() {
        let content = InterfaceUndefinedContent::new();
        let (a, b) = (fake_entity("a", &[]), fake_entity("b", &[]));
        content
            .add_literal(InterfaceParamType::ParamText, &text("x"))
            .unwrap();
        content
            .add_entity(InterfaceParamType::ParamIdent, &a)
            .unwrap();
        content
            .add_literal(InterfaceParamType::ParamReal, &text("1.5"))
            .unwrap();
        content
            .add_entity(InterfaceParamType::ParamIdent, &b)
            .unwrap();
        assert_eq!((content.nb_params(), content.nb_literals()), (4, 2));
        assert!(content.param_entity(4).unwrap() == b);
        assert_eq!(content.param_value(3).unwrap().string().as_bytes(), b"1.5");
        assert!(content.param_value(2).is_err() && content.param_entity(1).is_err());
        assert!(content.param_data(5).is_err());
        assert_eq!(
            content.param_type(3).unwrap(),
            InterfaceParamType::ParamReal
        );
        content.remove_param(1).unwrap();
        assert_eq!(content.param_value(2).unwrap().string().as_bytes(), b"1.5");
        content
            .set_literal(2, InterfaceParamType::ParamText, &text("y"))
            .unwrap();
        assert_eq!(content.param_value(2).unwrap().string().as_bytes(), b"y");
        let c = fake_entity("c", &[]);
        content
            .set_entity_typed(2, InterfaceParamType::ParamIdent, &c)
            .unwrap();
        assert!(content.param_entity(2).unwrap() == c);
        assert_eq!(content.nb_literals(), 0);
        content.set_entity(3, &a).unwrap();
        assert!(content.param_entity(3).unwrap() == a);
        assert_eq!(content.entity_list().nb_entities(), 3);
        let literal = InterfaceUndefinedContent::new();
        literal
            .add_literal(InterfaceParamType::ParamText, &text("z"))
            .unwrap();
        assert!(literal.set_entity(1, &a).is_err());
        assert!(!literal.is_param_entity(1).unwrap());
    }

    #[test]
    fn copy_from_another() {
        let (model, ents) = fake_chain();
        let source = InterfaceUndefinedContent::new();
        source
            .add_entity(InterfaceParamType::ParamIdent, &ents[0])
            .unwrap();
        source
            .add_literal(InterfaceParamType::ParamText, &text("t"))
            .unwrap();
        let mut tc = InterfaceCopyTool::with_protocol(&model, &model.protocol()).unwrap();
        let target = InterfaceUndefinedContent::new();
        target.get_from_another(&source, &mut tc).unwrap();
        assert_eq!(target.nb_params(), 2);
        assert!(target.param_entity(1).unwrap() != ents[0]);
        let (is_entity, _, _, val) = target.param_data(2).unwrap();
        assert!(!is_entity && val.string().as_bytes() == b"t");
    }
}
