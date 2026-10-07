// Rust port of DataExchange/TKXSBase/Interface/Interface_Category (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_Category`: the category numbers of entities, from their general modules,
//! and the process-wide list of category names.
use super::int_array::{InterfaceIntArray, int_at, int_upper, new_int_array, set_int};
use super::{
    InterfaceGTool, InterfaceInterfaceModel, InterfaceModelEntities, InterfaceProtocol,
    InterfaceShareTool,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use std::sync::{Mutex, OnceLock, PoisonError};

/// The category names (natively a 0-based `NCollection_DynamicArray`) and the native
/// "initialised" flag.
struct CategoryNames {
    names: Vec<AsciiString>,
    initialized: bool,
}

fn names() -> &'static Mutex<CategoryNames> {
    static NAMES: OnceLock<Mutex<CategoryNames>> = OnceLock::new();
    NAMES.get_or_init(|| {
        Mutex::new(CategoryNames {
            names: Vec::new(),
            initialized: false,
        })
    })
}

fn with_names<R>(read: impl FnOnce(&mut CategoryNames) -> R) -> R {
    read(&mut names().lock().unwrap_or_else(PoisonError::into_inner))
}

/// `Interface_Category`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceCategory;
/// InterfaceCategory::init();
/// assert_eq!(InterfaceCategory::name(1), "Drawing");
/// ```
pub struct InterfaceCategory {
    my_g_tool: Handle<InterfaceGTool>,
    my_num: InterfaceIntArray,
}

impl Default for InterfaceCategory {
    fn default() -> Self {
        Self::new()
    }
}

impl InterfaceCategory {
    /// `Interface_Category()`: a new GTool without protocol.
    pub fn new() -> Self {
        Self::with_gtool(&Handle::new(InterfaceGTool::new()))
    }
    /// `Interface_Category(protocol)`: a new GTool of `protocol`.
    pub fn with_protocol(protocol: &Handle<dyn InterfaceProtocol>) -> Self {
        Self::with_gtool(&Handle::new(InterfaceGTool::with_protocol(protocol, 0)))
    }
    /// `Interface_Category(gtool)`.
    pub fn with_gtool(gtool: &Handle<InterfaceGTool>) -> Self {
        Self::init();
        Self {
            my_g_tool: gtool.clone(),
            my_num: InterfaceIntArray::default(),
        }
    }
    /// `SetProtocol(protocol)`.
    pub fn set_protocol(&self, protocol: &Handle<dyn InterfaceProtocol>) {
        self.my_g_tool.set_protocol(protocol, false);
    }
    /// `CatNum(ent, shares)`: the category number its module gives, 0 when none.
    pub fn cat_num(&self, ent: &Handle<StandardTransient>, shares: &InterfaceShareTool) -> i32 {
        let Some((module, cn)) = self.my_g_tool.select(ent, false) else {
            return 0;
        };
        module.category_number(cn, ent, shares)
    }
    /// `ClearNums`.
    pub fn clear_nums(&mut self) {
        self.my_num.nullify();
    }
    /// `Compute(model, shares)`: the category number of every entity of the model.
    pub fn compute(
        &mut self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        shares: &InterfaceShareTool,
    ) -> Result<(), Failure> {
        self.clear_nums();
        let Some(live) = model.get() else {
            return Ok(());
        };
        let nb = live.nb_entities();
        self.my_g_tool.reservate(nb, false);
        if nb == 0 {
            return Ok(());
        }
        self.my_num = new_int_array(1, nb)?;
        for i in 1..=nb {
            let ent = live.value(i)?;
            if ent.is_null() {
                continue;
            }
            if let Some((module, cn)) = self.my_g_tool.select(&ent, false) {
                set_int(&self.my_num, i, module.category_number(cn, &ent, shares))?;
            }
        }
        Ok(())
    }
    /// `Num(nument)`: the computed category of an entity, 0 out of range.
    pub fn num(&self, nument: i32) -> i32 {
        if self.my_num.is_null()
            || nument < 1
            || int_upper(&self.my_num).map_or(true, |u| nument > u)
        {
            return 0;
        }
        int_at(&self.my_num, nument).unwrap_or(0)
    }
    /// `AddCategory(name)`: the number of a new or known name. Native quirks kept: a new
    /// name returns the count plus 1 (not its index), and the first name (index 0) is
    /// never found, so it is added again.
    pub fn add_category(name: &str) -> i32 {
        let num = Self::number(name);
        if num > 0 {
            return num;
        }
        with_names(|cats| {
            cats.names.push(AsciiString::new(name));
            cats.names.len() as i32 + 1
        })
    }
    /// `NbCategories`.
    pub fn nb_categories() -> i32 {
        with_names(|cats| cats.names.len() as i32)
    }
    /// `Name(num)`: the name of index `num` (from 0); "" for a negative number,
    /// "unspecified" beyond the list.
    pub fn name(num: i32) -> String {
        if num < 0 {
            return String::new();
        }
        with_names(|cats| match cats.names.get(num as usize) {
            Some(text) => String::from_utf8_lossy(text.as_bytes()).into_owned(),
            None => String::from("unspecified"),
        })
    }
    /// `Number(name)`: the index of a name, 0 when unknown (or the first name).
    pub fn number(name: &str) -> i32 {
        with_names(|cats| {
            cats.names
                .iter()
                .position(|n| n.is_equal(name))
                .map_or(0, |i| i as i32)
        })
    }
    /// `Init`: once per process, the standard categories (Shape, Drawing, Structure,
    /// Description, Auxiliary, Professional, FEA, Kinematics, Piping).
    pub fn init() {
        let first = with_names(|cats| !std::mem::replace(&mut cats.initialized, true));
        if !first {
            return;
        }
        let standard = [
            "Shape",
            "Drawing",
            "Structure",
            "Description",
            "Auxiliary",
            "Professional",
            "FEA",
            "Kinematics",
            "Piping",
        ];
        standard.iter().for_each(|name| {
            Self::add_category(name);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_norm::{fake_chain, fake_protocol};
    use super::*;

    #[test]
    fn standard_names_and_quirks() {
        InterfaceCategory::init();
        assert!(InterfaceCategory::nb_categories() >= 9);
        assert_eq!(InterfaceCategory::name(0), "Shape");
        assert_eq!(InterfaceCategory::name(-1), "");
        assert_eq!(InterfaceCategory::name(10_000), "unspecified");
        assert_eq!(InterfaceCategory::number("Drawing"), 1);
        assert_eq!(InterfaceCategory::number("Shape"), 0);
        assert_eq!(InterfaceCategory::add_category("Drawing"), 1);
    }

    #[test]
    fn computed_numbers() {
        let (model, ents) = fake_chain();
        let shares = InterfaceShareTool::new(&model).unwrap();
        let mut category = InterfaceCategory::with_protocol(&fake_protocol());
        category.set_protocol(&fake_protocol());
        assert_eq!(category.cat_num(&ents[0], &shares), 0);
        category.compute(&model, &shares).unwrap();
        assert_eq!(category.num(1), 0);
        assert_eq!(category.num(99), 0);
        category.clear_nums();
        assert_eq!(InterfaceCategory::new().num(1), 0);
    }
}
