// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SelectFlag,
// IFSelect_SelectIncorrectEntities, IFSelect_SelectRoots, IFSelect_SelectRootComps and
// IFSelect_SelectRange (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extractions by graph flag, by local roots (of entities or of components) and by
//! rank range.
use super::{
    IFSelectIntParam, IFSelectSelectDeduct, IFSelectSelectDeductBase, IFSelectSelectExtract,
    IFSelectSelectExtractBase, IFSelectSelection, IFSelectSelectionIterator, deduct_fill_iterator,
    extract_label,
};
use crate::if_graph::{IFGraphCumulate, IFGraphStrongComponants, IFGraphSubPartsEvaluate};
use crate::interface::{InterfaceEntityIterator, InterfaceGraph, InterfaceInterfaceModel};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};

/// Implements `IFSelectSelectDeduct` and the inherited `FillIterator`/`Label` of an
/// extraction that overrides `RootResult` (its `root_result` is given).
macro_rules! extract_with_root_result {
    ($type:ty, $base:ident, $root:expr, $unique:expr) => {
        impl IFSelectSelection for $type {
            fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
                ($root)(self, g)
            }
            fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
                deduct_fill_iterator(self, iter)
            }
            fn label(&self) -> AsciiString {
                extract_label(self)
            }
            fn has_unique_result(&self) -> bool {
                $unique
            }
        }
        impl IFSelectSelectDeduct for $type {
            fn deduct_base(&self) -> &IFSelectSelectDeductBase {
                &self.$base().deduct
            }
        }
    };
}

/// `IFSelect_SelectFlag`: the input entities with (direct) or without a flag of the
/// graph bit map.
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectSelectFlag, IFSelectSelection};
/// let flag = IFSelectSelectFlag::new("Incorrect");
/// assert_eq!(flag.label().as_bytes(), b"Picked: Entities Flagged by Incorrect");
/// ```
pub struct IFSelectSelectFlag {
    transient: StandardTransient,
    base: IFSelectSelectExtractBase,
    thename: AsciiString,
}
impl_transient!(IFSelectSelectFlag, "IFSelect_SelectFlag", dyn IFSelectSelectExtract, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectExtract]);

impl IFSelectSelectFlag {
    /// `IFSelect_SelectFlag(flagname)`.
    pub fn new(flagname: &str) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: IFSelectSelectExtractBase::default(),
            thename: AsciiString::new(flagname),
        }
    }
    /// `FlagName`.
    pub fn flag_name(&self) -> &AsciiString {
        &self.thename
    }
    fn flag_base(&self) -> &IFSelectSelectExtractBase {
        &self.base
    }
}

/// `IFSelect_SelectFlag::RootResult`: the input entities whose flag value equals the
/// direction (empty for an unknown flag).
fn flag_root_result<S: IFSelectSelectExtract + ?Sized>(
    sel: &S,
    name: &AsciiString,
    g: &InterfaceGraph,
) -> Result<InterfaceEntityIterator, Failure> {
    let mut res = InterfaceEntityIterator::new();
    let direct = sel.is_direct();
    let bm = g.bit_map();
    let flag = bm.flag_number(&String::from_utf8_lossy(name.as_bytes()));
    if flag == 0 {
        return Ok(res);
    }
    for ent in sel.input_result(g)?.content() {
        let num = g.entity_number(&ent);
        if num != 0 && direct == bm.value(num, flag)? {
            res.add_item(&g.entity(num)?);
        }
    }
    Ok(res)
}

extract_with_root_result!(
    IFSelectSelectFlag,
    flag_base,
    |s: &IFSelectSelectFlag, g| flag_root_result(s, &s.thename, g),
    false
);

impl IFSelectSelectExtract for IFSelectSelectFlag {
    fn extract_base(&self) -> &IFSelectSelectExtractBase {
        &self.base
    }
    /// `Sort`: false (not used: `RootResult` reads the flags).
    fn sort(
        &self,
        _rank: i32,
        _ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    /// `ExtractLabel`: "Entities Flagged by " and the flag name.
    fn extract_label(&self) -> AsciiString {
        let mut lab = AsciiString::new("Entities Flagged by ");
        lab.assign_cat(&self.thename);
        lab
    }
}

/// `IFSelect_SelectIncorrectEntities`: the entities flagged "Incorrect"; embeds its
/// `IFSelect_SelectFlag` (registered as its RTTI parent).
pub struct IFSelectSelectIncorrectEntities {
    base: IFSelectSelectFlag,
}
impl_transient!(IFSelectSelectIncorrectEntities, "IFSelect_SelectIncorrectEntities", IFSelectSelectFlag, base, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectExtract]);

impl Default for IFSelectSelectIncorrectEntities {
    fn default() -> Self {
        Self::new()
    }
}

impl IFSelectSelectIncorrectEntities {
    /// `IFSelect_SelectIncorrectEntities()`: the flag "Incorrect".
    pub fn new() -> Self {
        Self {
            base: IFSelectSelectFlag::new("Incorrect"),
        }
    }
    /// `FlagName` (inherited).
    pub fn flag_name(&self) -> &AsciiString {
        self.base.flag_name()
    }
    fn flag_base(&self) -> &IFSelectSelectExtractBase {
        &self.base.base
    }
}

extract_with_root_result!(
    IFSelectSelectIncorrectEntities,
    flag_base,
    |s: &IFSelectSelectIncorrectEntities, g| flag_root_result(s, s.flag_name(), g),
    false
);

impl IFSelectSelectExtract for IFSelectSelectIncorrectEntities {
    fn extract_base(&self) -> &IFSelectSelectExtractBase {
        &self.base.base
    }
    fn sort(
        &self,
        rank: i32,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        self.base.sort(rank, ent, model)
    }
    fn extract_label(&self) -> AsciiString {
        self.base.extract_label()
    }
}

/// The entities of `input` selected at most once by `cumulate` (or more, when not
/// direct).
fn local_roots(
    input: &[Handle<StandardTransient>],
    cumulate: &IFGraphCumulate,
    direct: bool,
) -> Result<InterfaceEntityIterator, Failure> {
    let mut iter = InterfaceEntityIterator::new();
    for ent in input {
        if (cumulate.nb_times(ent)? <= 1) == direct {
            iter.get_one_item(ent);
        }
    }
    Ok(iter)
}

/// Declares an extraction of local roots with its `Sort` (true) and `ExtractLabel`.
macro_rules! roots_extract {
    ($(#[$doc:meta])* $name:ident, $native:literal, $label:literal, $root:ident) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $name {
            transient: StandardTransient,
            base: IFSelectSelectExtractBase,
        }
        impl_transient!($name, $native, dyn IFSelectSelectExtract, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectExtract]);
        impl $name {
            #[doc = concat!("`", $native, "()`: direct, no input.")]
            pub fn new() -> Self {
                Self::default()
            }
            fn roots_base(&self) -> &IFSelectSelectExtractBase {
                &self.base
            }
        }
        extract_with_root_result!($name, roots_base, $root, true);
        impl IFSelectSelectExtract for $name {
            fn extract_base(&self) -> &IFSelectSelectExtractBase {
                &self.base
            }
            /// `Sort`: true (not used: `RootResult` counts the sharings).
            fn sort(&self, _rank: i32, _ent: &Handle<StandardTransient>, _model: &Handle<dyn InterfaceInterfaceModel>) -> Result<bool, Failure> {
                Ok(true)
            }
            fn extract_label(&self) -> AsciiString {
                AsciiString::new($label)
            }
        }
    };
}

/// `IFSelect_SelectRoots::RootResult`: the input entities taken at most once by the
/// cumulation of the input. Native defect kept: `IFGraph_Cumulate::GetFromEntity` adds
/// nothing, so every input entity is a root (direct).
fn roots_root_result(
    sel: &IFSelectSelectRoots,
    g: &InterfaceGraph,
) -> Result<InterfaceEntityIterator, Failure> {
    let input = sel.input_result(g)?.content();
    let mut cumulate = IFGraphCumulate::new(g)?;
    for ent in &input {
        cumulate.get_from_entity(ent)?;
    }
    local_roots(&input, &cumulate, sel.is_direct())
}

roots_extract!(
    /// `IFSelect_SelectRoots`: the local roots of the input (not shared inside it).
    IFSelectSelectRoots, "IFSelect_SelectRoots", "Local Roots", roots_root_result
);

/// `IFSelect_SelectRootComps::RootResult`: as `SelectRoots`, on the first entity of each
/// strong component of the input.
fn root_comps_root_result(
    sel: &IFSelectSelectRootComps,
    g: &InterfaceGraph,
) -> Result<InterfaceEntityIterator, Failure> {
    let input = sel.input_result(g)?;
    let mut comps = IFGraphStrongComponants::new(g, false)?;
    comps.set_load();
    comps.get_from_iter(&input)?;
    let mut cumulate = IFGraphCumulate::new(g)?;
    let mut firsts = Vec::new();
    comps.start()?;
    while comps.more()? {
        let ent = comps.first_entity()?;
        cumulate.get_from_entity(&ent)?;
        firsts.push(ent);
        comps.next();
    }
    local_roots(&firsts, &cumulate, sel.is_direct())
}

roots_extract!(
    /// `IFSelect_SelectRootComps`: the local roots of the strong components of the input.
    IFSelectSelectRootComps, "IFSelect_SelectRootComps", "Local Root Components", root_comps_root_result
);

/// The fields of a rank range (`IFSelect_SelectRange`, `IFSelect_SelectAnyList`).
#[derive(Default)]
pub struct IFSelectRankRange {
    thelower: SharedState<Handle<IFSelectIntParam>>,
    theupper: SharedState<Handle<IFSelectIntParam>>,
}

impl IFSelectRankRange {
    /// `SetRange(rankfrom, rankto)`.
    pub fn set_range(
        &self,
        rankfrom: &Handle<IFSelectIntParam>,
        rankto: &Handle<IFSelectIntParam>,
    ) {
        *self.thelower.write() = rankfrom.clone();
        *self.theupper.write() = rankto.clone();
    }
    /// `SetOne(rank)`: the same lower and upper rank.
    pub fn set_one(&self, rank: &Handle<IFSelectIntParam>) {
        self.set_range(rank, rank);
    }
    /// `SetFrom(rankfrom)`: no upper rank.
    pub fn set_from(&self, rankfrom: &Handle<IFSelectIntParam>) {
        self.set_range(rankfrom, &Handle::default());
    }
    /// `SetUntil(rankto)`: no lower rank.
    pub fn set_until(&self, rankto: &Handle<IFSelectIntParam>) {
        self.set_range(&Handle::default(), rankto);
    }
    /// `HasLower`.
    pub fn has_lower(&self) -> bool {
        self.lower().is_some()
    }
    /// `Lower`.
    pub fn lower(&self) -> Handle<IFSelectIntParam> {
        self.thelower.read().clone()
    }
    /// `LowerValue`: 0 without lower rank.
    pub fn lower_value(&self) -> i32 {
        self.lower().get().map_or(0, IFSelectIntParam::value)
    }
    /// `HasUpper`.
    pub fn has_upper(&self) -> bool {
        self.upper().is_some()
    }
    /// `Upper`.
    pub fn upper(&self) -> Handle<IFSelectIntParam> {
        self.theupper.read().clone()
    }
    /// `UpperValue`: 0 without upper rank.
    pub fn upper_value(&self) -> i32 {
        self.upper().get().map_or(0, IFSelectIntParam::value)
    }
}

/// `IFSelect_SelectRange`: the input entities of a rank range; dereferences to its
/// [`IFSelectRankRange`].
#[derive(Default)]
pub struct IFSelectSelectRange {
    transient: StandardTransient,
    base: IFSelectSelectExtractBase,
    range: IFSelectRankRange,
}
impl_transient!(IFSelectSelectRange, "IFSelect_SelectRange", dyn IFSelectSelectExtract, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectExtract]);
super::impl_extract_selection!(IFSelectSelectRange, base);

impl std::ops::Deref for IFSelectSelectRange {
    type Target = IFSelectRankRange;
    fn deref(&self) -> &IFSelectRankRange {
        &self.range
    }
}

impl IFSelectSelectRange {
    /// `IFSelect_SelectRange()`: no rank.
    pub fn new() -> Self {
        Self::default()
    }
}

impl IFSelectSelectExtract for IFSelectSelectRange {
    fn extract_base(&self) -> &IFSelectSelectExtractBase {
        &self.base
    }
    /// `Sort`: the rank is in the range (no upper rank for 0).
    fn sort(
        &self,
        rank: i32,
        _ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        let (from, to) = (self.lower_value(), self.upper_value());
        Ok(rank >= from && (to == 0 || to >= rank))
    }
    /// `ExtractLabel`: "Rank no n", "Until no n", "From no n" or "From n Until m".
    /// Native quirk kept: "From no" prints the upper rank (0).
    fn extract_label(&self) -> AsciiString {
        let (from, to) = (self.lower_value(), self.upper_value());
        AsciiString::new(&if from == to {
            format!("Rank no {from}")
        } else if from == 0 {
            format!("Until no {to}")
        } else if to == 0 {
            format!("From no {to}")
        } else {
            format!("From {from} Until {to}")
        })
    }
}
