// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SignType, IFSelect_SignAncestor,
// IFSelect_SignCategory, IFSelect_SignValidity and IFSelect_SignMultiple (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The signatures by type (dynamic or class), ancestor type, category, validity, and the
//! signature made of several ones.
use super::{IFSelectSignature, IFSelectSignatureBase, impl_signature_text};
use crate::interface::{
    InterfaceCategory, InterfaceInterfaceModel, InterfaceModelEntities, InterfaceModelReports,
    InterfaceSignType,
};
use crate::moni_tool::MoniToolSignText;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, RuntimeType, SharedState, StandardTransient},
    tcollection::AsciiString,
};

/// Applies `read` to the type a signature by type reads: the entity itself when it is a
/// type, else its dynamic type (`None` for null).
fn with_entity_type<R>(
    ent: &Handle<StandardTransient>,
    read: impl FnOnce(&RuntimeType) -> R,
) -> Option<R> {
    let kind = ent.cast::<RuntimeType>();
    match kind.get() {
        Some(live) => Some(read(live)),
        None => ent.dynamic_type().map(read),
    }
}

/// `IFSelect_SignType`: the type name of the entity (without its package with `nopk`).
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceSignType;
/// use rustycad_tkxsbase::if_select::IFSelectSignType;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let sign = IFSelectSignType::new(true);
/// assert_eq!(sign.value(&Handle::new(StandardTransient::default()), &Handle::default()), "Transient");
/// ```
pub struct IFSelectSignType {
    transient: StandardTransient,
    base: IFSelectSignatureBase,
    thenopk: bool,
}
impl_transient!(IFSelectSignType, "IFSelect_SignType", dyn IFSelectSignature, transient, interfaces: [dyn MoniToolSignText, dyn InterfaceSignType, dyn IFSelectSignature]);
impl_signature_text!(IFSelectSignType);

impl IFSelectSignType {
    /// `IFSelect_SignType(nopk = false)`: "Class Type" (without package) or "Dynamic Type".
    pub fn new(nopk: bool) -> Self {
        let name = if nopk { "Class Type" } else { "Dynamic Type" };
        Self {
            transient: StandardTransient::default(),
            base: IFSelectSignatureBase::new(name),
            thenopk: nopk,
        }
    }
}

/// `IFSelect_SignType::Value`: the type name, after the first '_' with `nopk`.
fn sign_type_value(ent: &Handle<StandardTransient>, nopk: bool) -> String {
    with_entity_type(ent, |kind| {
        let name = kind.name();
        if !nopk {
            return name.to_owned();
        }
        name.split_once('_')
            .map_or(name, |(_, rest)| rest)
            .to_owned()
    })
    .unwrap_or_default()
}

impl InterfaceSignType for IFSelectSignType {
    fn value(
        &self,
        ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> String {
        sign_type_value(ent, self.thenopk)
    }
}

impl IFSelectSignature for IFSelectSignType {
    fn signature_base(&self) -> &IFSelectSignatureBase {
        &self.base
    }
}

/// `IFSelect_SignAncestor`: a signature by type which matches the ancestor types too;
/// embeds its `IFSelect_SignType` (registered as its RTTI parent).
pub struct IFSelectSignAncestor {
    base: IFSelectSignType,
}
impl_transient!(IFSelectSignAncestor, "IFSelect_SignAncestor", IFSelectSignType, base, interfaces: [dyn MoniToolSignText, dyn InterfaceSignType, dyn IFSelectSignature]);
impl_signature_text!(IFSelectSignAncestor);

impl IFSelectSignAncestor {
    /// `IFSelect_SignAncestor(nopk = false)`.
    pub fn new(nopk: bool) -> Self {
        Self {
            base: IFSelectSignType::new(nopk),
        }
    }
}

impl InterfaceSignType for IFSelectSignAncestor {
    fn value(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> String {
        self.base.value(ent, model)
    }
}

impl IFSelectSignature for IFSelectSignAncestor {
    fn signature_base(&self) -> &IFSelectSignatureBase {
        &self.base.base
    }
    /// `Matches`: the type of `ent` is `text` or inherits it (by name).
    fn matches(
        &self,
        ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
        text: &AsciiString,
        _exact: bool,
    ) -> Result<bool, Failure> {
        let name = String::from_utf8_lossy(text.as_bytes()).into_owned();
        Ok(with_entity_type(ent, |kind| kind.is_subtype_name(&name)).unwrap_or(false))
    }
}

/// `IFSelect_SignCategory`: the category of the entity in the model.
pub struct IFSelectSignCategory {
    transient: StandardTransient,
    base: IFSelectSignatureBase,
}
impl_transient!(IFSelectSignCategory, "IFSelect_SignCategory", dyn IFSelectSignature, transient, interfaces: [dyn MoniToolSignText, dyn InterfaceSignType, dyn IFSelectSignature]);
impl_signature_text!(IFSelectSignCategory);

impl Default for IFSelectSignCategory {
    fn default() -> Self {
        Self::new()
    }
}

impl IFSelectSignCategory {
    /// `IFSelect_SignCategory()`: "Category", with the category names as cases.
    pub fn new() -> Self {
        let sign = Self {
            transient: StandardTransient::default(),
            base: IFSelectSignatureBase::new("Category"),
        };
        InterfaceCategory::init();
        for i in 1..=InterfaceCategory::nb_categories() {
            sign.add_case(&InterfaceCategory::name(i));
        }
        sign
    }
}

impl InterfaceSignType for IFSelectSignCategory {
    /// `Value`: the category name of the entity (empty for null).
    fn value(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> String {
        match model.get().filter(|_| ent.is_some()) {
            Some(live) => InterfaceCategory::name(live.category_number(live.number(ent))),
            None => String::new(),
        }
    }
}

impl IFSelectSignature for IFSelectSignCategory {
    fn signature_base(&self) -> &IFSelectSignatureBase {
        &self.base
    }
}

/// `IFSelect_SignValidity`: the validity of the entity (unknown, unloaded, errors,
/// warnings, OK).
pub struct IFSelectSignValidity {
    transient: StandardTransient,
    base: IFSelectSignatureBase,
}
impl_transient!(IFSelectSignValidity, "IFSelect_SignValidity", dyn IFSelectSignature, transient, interfaces: [dyn MoniToolSignText, dyn InterfaceSignType, dyn IFSelectSignature]);
impl_signature_text!(IFSelectSignValidity);

impl Default for IFSelectSignValidity {
    fn default() -> Self {
        Self::new()
    }
}

impl IFSelectSignValidity {
    /// `IFSelect_SignValidity()`: "Validity", with its seven cases.
    pub fn new() -> Self {
        let sign = Self {
            transient: StandardTransient::default(),
            base: IFSelectSignatureBase::new("Validity"),
        };
        for case in [
            "UNKNOWN",
            "UNLOADED",
            "Load-Error",
            "Data-Error",
            "Load-Warning",
            "Data-Warning",
            "OK",
        ] {
            sign.add_case(case);
        }
        sign
    }
    /// `CVal(ent, model)`: the validity of `ent` (empty for null).
    pub fn c_val(
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> &'static str {
        let Some(live) = model.get().filter(|_| ent.is_some()) else {
            return "";
        };
        let num = live.number(ent);
        if live.is_unknown_entity(num) {
            return "UNKNOWN";
        }
        if live.is_redefined_content(num) {
            return "UNLOADED";
        }
        let (syntactic, semantic) = (live.check(num, true), live.check(num, false));
        if syntactic.nb_fails() > 0 {
            return "Load-Error";
        }
        if semantic.nb_fails() > 0 {
            return "Data-Error";
        }
        if syntactic.nb_warnings() > 0 {
            return "Load-Warning";
        }
        if semantic.nb_warnings() > 0 {
            return "Data-Warning";
        }
        "OK"
    }
}

impl InterfaceSignType for IFSelectSignValidity {
    fn value(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> String {
        Self::c_val(ent, model).to_owned()
    }
}

impl IFSelectSignature for IFSelectSignValidity {
    fn signature_base(&self) -> &IFSelectSignatureBase {
        &self.base
    }
}

/// The fields of `IFSelect_SignMultiple`: the sub-signatures and their widths.
#[derive(Default)]
struct MultipleState {
    thesubs: Vec<Handle<dyn IFSelectSignature>>,
    thetabs: Vec<i32>,
}

/// `IFSelect_SignMultiple`: the values of several signatures, in columns.
pub struct IFSelectSignMultiple {
    transient: StandardTransient,
    base: IFSelectSignatureBase,
    state: SharedState<MultipleState>,
}
impl_transient!(IFSelectSignMultiple, "IFSelect_SignMultiple", dyn IFSelectSignature, transient, interfaces: [dyn MoniToolSignText, dyn InterfaceSignType, dyn IFSelectSignature]);
impl_signature_text!(IFSelectSignMultiple);

impl IFSelectSignMultiple {
    /// `IFSelect_SignMultiple(name)`: no sub-signature.
    pub fn new(name: &str) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: IFSelectSignatureBase::new(name),
            state: SharedState::default(),
        }
    }
    /// `Add(subsign, width = 0, maxi = false)`: a column of `width` (at most, with
    /// `maxi`); nothing for null.
    pub fn add(&self, subsign: &Handle<dyn IFSelectSignature>, width: i32, maxi: bool) {
        if subsign.is_null() {
            return;
        }
        let mut st = self.state.write();
        st.thesubs.push(subsign.clone());
        st.thetabs.push(if maxi { -width } else { width });
    }
}

/// One column of `IFSelect_SignMultiple::Value`: `val` justified to `width` and three
/// blanks. Native defect kept as its loop computes it: a value longer than a maximal
/// width is not truncated; `Remove` gets a negative count and copies the characters from
/// the width after the value (a buffer overrun natively), then the last character of the
/// value becomes '.'.
fn column(val: &str, tabul: i32) -> String {
    let maxi = tabul < 0;
    let width = tabul.unsigned_abs() as usize;
    let bytes = val.as_bytes();
    let sl = bytes.len();
    let mut out = bytes.to_vec();
    if sl < width {
        out.resize(width, b' ');
    }
    if sl > width && maxi {
        out.extend_from_slice(&bytes[width..sl]);
        out[sl - 1] = b'.';
    }
    out.extend_from_slice(b"   ");
    String::from_utf8_lossy(&out).into_owned()
}

impl InterfaceSignType for IFSelectSignMultiple {
    /// `Value`: the values of the sub-signatures, in columns.
    fn value(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> String {
        let st = self.state.read();
        st.thesubs
            .iter()
            .zip(&st.thetabs)
            .map(|(sign, tabul)| {
                column(
                    &sign.get().map(|s| s.value(ent, model)).unwrap_or_default(),
                    *tabul,
                )
            })
            .collect()
    }
}

impl IFSelectSignature for IFSelectSignMultiple {
    fn signature_base(&self) -> &IFSelectSignatureBase {
        &self.base
    }
    /// `Matches`: exact on the whole value, else when a sub-signature matches.
    fn matches(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
        text: &AsciiString,
        exact: bool,
    ) -> Result<bool, Failure> {
        if exact {
            return IFSelectSignatureBase::match_value(&self.value(ent, model), text, exact);
        }
        let subs = self.state.read().thesubs.clone();
        for sign in subs {
            if let Some(live) = sign.get()
                && live.matches(ent, model, text, exact)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
