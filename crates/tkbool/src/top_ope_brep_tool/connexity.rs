// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_connexity.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_connexity`: the shapes connected to a key shape, sorted by the
//! orientation of the key in them (FORWARD 1, REVERSED 2, INTERNAL 3, EXTERNAL 4,
//! CLOSING 5).
use crate::top_ope_brep_tool::tool::{
    TOOL_CLOSING, TOOL_EXTERNAL, TOOL_FORWARD, TOOL_INTERNAL, TOOL_REVERSED, TopOpeBRepToolTOOL,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// The connexity of a key shape; `let c = TopOpeBRepToolConnexity::with_key(&v);`.
#[derive(Clone, Default)]
pub struct TopOpeBRepToolConnexity {
    the_key: Shape,
    /// `theItems(1, 5)`: the items per orientation code.
    the_items: [List<Shape>; 5],
}

/// Standard_OutOfRange of an orientation code outside 1..5.
fn out_of_range(member: &str, ori_key: i32) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("TopOpeBRepTool_connexity::{member}: orientation code {ori_key}; expected 1 to 5"),
        "TopOpeBRepTool_connexity",
    )
}

impl TopOpeBRepToolConnexity {
    /// `TopOpeBRepTool_connexity()`; `TopOpeBRepToolConnexity::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepTool_connexity(Key)`; `TopOpeBRepToolConnexity::with_key(&v)`.
    pub fn with_key(key: &Shape) -> Self {
        Self {
            the_key: key.clone(),
            ..Self::default()
        }
    }

    /// `SetKey(Key)`; `c.set_key(&v)`.
    pub fn set_key(&mut self, key: &Shape) {
        self.the_key = key.clone();
    }

    /// `Key()`; `c.key()`.
    pub fn key(&self) -> &Shape {
        &self.the_key
    }

    /// The list of an orientation code.
    fn items_of(&self, member: &str, ori_key: i32) -> Result<&List<Shape>, Failure> {
        usize::try_from(ori_key - 1)
            .ok()
            .and_then(|i| self.the_items.get(i))
            .ok_or_else(|| out_of_range(member, ori_key))
    }

    /// `Item(OriKey, Item)`: a copy of the items of the code and their count;
    /// `let (n, items) = c.item(1)?;`.
    pub fn item(&self, ori_key: i32) -> Result<(i32, List<Shape>), Failure> {
        let items = self.items_of("Item", ori_key)?.clone();
        Ok((items.extent(), items))
    }

    /// `AllItems(Item)`: the items of the codes 1 to 4 and their count;
    /// `let (n, items) = c.all_items();`.
    pub fn all_items(&self) -> (i32, List<Shape>) {
        let mut item: List<Shape> = List::default();
        for list in &self.the_items[..4] {
            for s in list.iter() {
                item.append(s.clone());
            }
        }
        (item.extent(), item)
    }

    /// `AddItem(OriKey, Item)`: the shapes appended to the code; `c.add_items(1, &l)?`.
    pub fn add_items(&mut self, ori_key: i32, item: &List<Shape>) -> Result<(), Failure> {
        let list = self.change_item(ori_key)?;
        for s in item.iter() {
            list.append(s.clone());
        }
        Ok(())
    }

    /// `AddItem(OriKey, Item)`: the shape appended to the code; `c.add_item(1, &e)?`.
    pub fn add_item(&mut self, ori_key: i32, item: &Shape) -> Result<(), Failure> {
        self.change_item(ori_key)?.append(item.clone());
        Ok(())
    }

    /// `RemoveItem(OriKey, Item)`: the first item of the code equal to the shape
    /// removed; whether there was one; `c.remove_item_of(1, &e)?`.
    pub fn remove_item_of(&mut self, ori_key: i32, item: &Shape) -> Result<bool, Failure> {
        let list = self.change_item(ori_key)?;
        let Some(pos) = list.iter().position(|s| s.is_equal(item)) else {
            return Ok(false);
        };
        let mut index = 0;
        list.retain(|_| {
            index += 1;
            index != pos + 1
        });
        Ok(true)
    }

    /// `RemoveItem(Item)`: the shape removed from every code; whether it was in one;
    /// `c.remove_item(&e)`.
    pub fn remove_item(&mut self, item: &Shape) -> bool {
        let mut removed = false;
        for i in 1..=5 {
            if self.remove_item_of(i, item).unwrap_or(false) {
                removed = true;
            }
        }
        removed
    }

    /// `ChangeItem(OriKey)`; `c.change_item(1)?.append(e)`.
    pub fn change_item(&mut self, ori_key: i32) -> Result<&mut List<Shape>, Failure> {
        usize::try_from(ori_key - 1)
            .ok()
            .and_then(|i| self.the_items.get_mut(i))
            .ok_or_else(|| out_of_range("ChangeItem", ori_key))
    }

    /// The count of the items of a valid code.
    fn count(&self, ori_key: i32) -> i32 {
        self.the_items[(ori_key - 1) as usize].extent()
    }

    /// `IsMultiple()`: more than one FORWARD or CLOSING item; `c.is_multiple()`.
    pub fn is_multiple(&self) -> bool {
        self.count(TOOL_FORWARD) + self.count(TOOL_CLOSING) > 1
    }

    /// `IsFaulty()`: different counts of FORWARD and REVERSED items; `c.is_faulty()`.
    pub fn is_faulty(&self) -> bool {
        self.count(TOOL_FORWARD) != self.count(TOOL_REVERSED)
    }

    /// Whether an item oriented `o` keeps the key INTERNAL in it.
    fn key_internal_in(&self, item: &Shape) -> Result<bool, Failure> {
        let o_key = TopOpeBRepToolTOOL::oriin_sor(
            &self.the_key,
            &item.oriented(Orientation::Forward),
            false,
        )?;
        Ok(o_key == TOOL_INTERNAL)
    }

    /// `IsInternal(Item)`: the INTERNAL items (only the INTERNAL-oriented ones in which
    /// the key is INTERNAL are kept, the others whatever) then the EXTERNAL-oriented
    /// EXTERNAL items in which the key is INTERNAL, and their count;
    /// `let (n, items) = c.is_internal()?;`.
    pub fn is_internal(&self) -> Result<(i32, List<Shape>), Failure> {
        let mut l_int: List<Shape> = List::default();
        for item1 in self.the_items[(TOOL_INTERNAL - 1) as usize].iter() {
            if item1.orientation() != Orientation::Internal || self.key_internal_in(item1)? {
                l_int.append(item1.clone());
            }
        }
        for item2 in self.the_items[(TOOL_EXTERNAL - 1) as usize].iter() {
            if item2.orientation() == Orientation::External && self.key_internal_in(item2)? {
                l_int.append(item2.clone());
            }
        }
        Ok((l_int.extent(), l_int))
    }
}
