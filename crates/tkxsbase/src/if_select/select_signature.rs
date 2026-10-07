// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SelectSignature (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_SelectSignature`: the input entities whose signature (or counter value)
//! matches a text, or a list of criteria ("A|B!C", "<10", ">=2"...).
use super::{
    IFSelectSelectDeduct, IFSelectSelectDeductBase, IFSelectSelectExtract,
    IFSelectSelectExtractBase, IFSelectSelection, IFSelectSelectionIterator,
    IFSelectSignCounterOps, IFSelectSignature, IFSelectSignatureBase, deduct_fill_iterator,
    extract_label, extract_root_result,
};
use crate::c_text::atoi;
use crate::interface::{InterfaceEntityIterator, InterfaceGraph, InterfaceInterfaceModel};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::AsciiString,
};

/// `multsign` (static): splits a criteria text into its items and modes (1: or, 2: not,
/// +2 with '=' (exact), +16 '<', +32 '>'); the number of items (0 for a plain text).
/// Native quirk kept: a text ending with '|' or '!' counts one item more than it has.
fn multsign(
    signtext: &AsciiString,
    signlist: &mut Vec<AsciiString>,
    signmode: &mut Vec<i32>,
) -> i32 {
    let bytes = signtext.as_bytes();
    if !bytes.iter().any(|c| b"|!=<>".contains(c)) {
        return 0;
    }
    let (mut mode, mut imod, mut item) = (1, 1, Vec::new());
    for &unsign in bytes {
        match unsign {
            b'|' | b'!' => {
                if !item.is_empty() {
                    signlist.push(AsciiString::from_raw(&item));
                    signmode.push(imod);
                    item.clear();
                    mode += 1;
                }
                imod = if unsign == b'|' { 1 } else { 2 };
            }
            b'<' => imod += 16,
            b'>' => imod += 32,
            b'=' => imod += if imod < 8 { 2 } else { 8 },
            _ => item.push(unsign),
        }
    }
    if !item.is_empty() {
        signlist.push(AsciiString::from_raw(&item));
        signmode.push(imod);
    }
    mode
}

/// `IFSelect_SelectSignature`.
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectSelectSignature, IFSelectSelection, IFSelectSignType};
/// use rustycad_tkernel::standard::Handle;
/// let sel = IFSelectSelectSignature::with_signature(&Handle::new(IFSelectSignType::new(true)).cast(), "Line", false);
/// assert_eq!(sel.label().as_bytes(), b"Picked: Class Type containing Line");
/// ```
pub struct IFSelectSelectSignature {
    transient: StandardTransient,
    base: IFSelectSelectExtractBase,
    thematcher: Handle<dyn IFSelectSignature>,
    thecounter: Handle<dyn IFSelectSignCounterOps>,
    thesigntext: AsciiString,
    theexact: i32,
    thesignlist: Vec<AsciiString>,
    thesignmode: Vec<i32>,
}
impl_transient!(IFSelectSelectSignature, "IFSelect_SelectSignature", dyn IFSelectSelectExtract, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectExtract]);

impl IFSelectSelectSignature {
    /// The common constructor: exact (-1), else the criteria of the text.
    fn build(
        matcher: Handle<dyn IFSelectSignature>,
        counter: Handle<dyn IFSelectSignCounterOps>,
        signtext: &AsciiString,
        exact: bool,
    ) -> Self {
        let (mut thesignlist, mut thesignmode) = (Vec::new(), Vec::new());
        let theexact = if exact {
            -1
        } else {
            multsign(signtext, &mut thesignlist, &mut thesignmode)
        };
        Self {
            transient: StandardTransient::default(),
            base: IFSelectSelectExtractBase::default(),
            thematcher: matcher,
            thecounter: counter,
            thesigntext: signtext.clone(),
            theexact,
            thesignlist,
            thesignmode,
        }
    }
    /// `IFSelect_SelectSignature(matcher, signtext, exact = true)` (both text forms).
    pub fn with_signature(
        matcher: &Handle<dyn IFSelectSignature>,
        signtext: &str,
        exact: bool,
    ) -> Self {
        Self::build(
            matcher.clone(),
            Handle::default(),
            &AsciiString::new(signtext),
            exact,
        )
    }
    /// `IFSelect_SelectSignature(counter, signtext, exact = true)`.
    pub fn with_counter(
        counter: &Handle<dyn IFSelectSignCounterOps>,
        signtext: &str,
        exact: bool,
    ) -> Self {
        Self::build(
            Handle::default(),
            counter.clone(),
            &AsciiString::new(signtext),
            exact,
        )
    }
    /// `Signature`.
    pub fn signature(&self) -> Handle<dyn IFSelectSignature> {
        self.thematcher.clone()
    }
    /// `Counter`.
    pub fn counter(&self) -> Handle<dyn IFSelectSignCounterOps> {
        self.thecounter.clone()
    }
    /// `SignatureText`.
    pub fn signature_text(&self) -> &AsciiString {
        &self.thesigntext
    }
    /// `IsExact`.
    pub fn is_exact(&self) -> bool {
        self.theexact < 0
    }
    /// The value of `ent`: the signature value, else the computed sign of the counter;
    /// `Standard_NullObject` without both (a native null dereference).
    fn value_of(
        &self,
        ent: &Handle<StandardTransient>,
        g: &InterfaceGraph,
    ) -> Result<String, Failure> {
        if let Some(matcher) = self.thematcher.get() {
            return Ok(matcher.value(ent, g.model()));
        }
        let counter = self.thecounter.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_SelectSignature: no signature and no counter")
        })?;
        Ok(String::from_utf8_lossy(counter.computed_sign(ent, g)?.as_bytes()).into_owned())
    }
    /// The text match of `ent` against `text`.
    fn text_matches(
        &self,
        ent: &Handle<StandardTransient>,
        g: &InterfaceGraph,
        text: &AsciiString,
        exact: bool,
    ) -> Result<bool, Failure> {
        match self.thematcher.get() {
            Some(matcher) => matcher.matches(ent, g.model(), text, exact),
            None => IFSelectSignatureBase::match_value(&self.value_of(ent, g)?, text, exact),
        }
    }
    /// One criterion of the list (an item and its mode).
    fn criterion(
        &self,
        ent: &Handle<StandardTransient>,
        g: &InterfaceGraph,
        text: &AsciiString,
        hmod: i32,
    ) -> Result<bool, Failure> {
        let (jmod, imod) = (hmod / 8, hmod % 8);
        if jmod == 0 {
            return self.text_matches(ent, g, text, imod > 2);
        }
        let (val, lav) = (
            atoi(self.value_of(ent, g)?.as_bytes()),
            atoi(text.as_bytes()),
        );
        Ok(match jmod {
            2 => val < lav,
            3 => val <= lav,
            4 => val > lav,
            5 => val >= lav,
            _ => false,
        })
    }
    /// The mode of item `i` (from 1); `Standard_OutOfRange` beyond the list.
    fn mode_at(&self, i: i32) -> Result<(AsciiString, i32), Failure> {
        let index = usize::try_from(i - 1).ok();
        let text = index.and_then(|n| self.thesignlist.get(n));
        let mode = index.and_then(|n| self.thesignmode.get(n));
        match (text, mode) {
            (Some(t), Some(m)) => Ok((t.clone(), *m)),
            _ => Err(exceptions::out_of_range(&format!(
                "IFSelect_SelectSignature: criterion {i}; expected [1, {}]",
                self.thesignlist.len()
            ))),
        }
    }
}

impl IFSelectSelection for IFSelectSelectSignature {
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        extract_root_result(self, g)
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        deduct_fill_iterator(self, iter)
    }
    fn label(&self) -> AsciiString {
        extract_label(self)
    }
}

impl IFSelectSelectDeduct for IFSelectSelectSignature {
    fn deduct_base(&self) -> &IFSelectSelectDeductBase {
        &self.base.deduct
    }
}

impl IFSelectSelectExtract for IFSelectSelectSignature {
    fn extract_base(&self) -> &IFSelectSelectExtractBase {
        &self.base
    }
    /// `SortInGraph(rank, ent, G)`: the text match, or the criteria in turn (an "or"
    /// criterion sets the result, a "not" one clears it).
    fn sort_in_graph(
        &self,
        _rank: i32,
        ent: &Handle<StandardTransient>,
        g: &InterfaceGraph,
    ) -> Result<bool, Failure> {
        if self.theexact <= 0 {
            return self.text_matches(ent, g, &self.thesigntext, self.theexact < 0);
        }
        let first = self.mode_at(1)?.1 % 8;
        let mut res = first == 2 || first == 4;
        for i in 1..=self.theexact {
            let (text, hmod) = self.mode_at(i)?;
            let imod = hmod % 8;
            let quid = self.criterion(ent, g, &text, hmod)?;
            if (imod == 1 || imod == 3) && quid {
                res = true;
            }
            if (imod == 2 || imod == 4) && quid {
                res = false;
            }
        }
        Ok(res)
    }
    /// `Sort`: true (not used: `SortInGraph` matches).
    fn sort(
        &self,
        _rank: i32,
        _ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `ExtractLabel`: the signature (or counter) name, "matching", "containing" or
    /// "matching list", and the text.
    fn extract_label(&self) -> AsciiString {
        let name = match self.thematcher.get() {
            Some(matcher) => matcher.name(),
            None => self.thecounter.get().map(|c| c.name()).unwrap_or_default(),
        };
        let kind = match self.theexact {
            e if e < 0 => " matching ",
            0 => " containing ",
            _ => " matching list ",
        };
        let mut lab = AsciiString::new(&format!("{name}{kind}"));
        lab.assign_cat(&self.thesigntext);
        lab
    }
}
