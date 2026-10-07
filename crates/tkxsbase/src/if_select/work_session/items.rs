// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the items.

//! The items of the session: identified by number (ident), optionally named, each
//! with an attribute (the item itself, its name, or null once removed).
use super::{IFSelectWorkSession, WorkSessionState};
use crate::c_text::atoi;
use crate::if_select::IFSelectDispatch;
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::{AsciiString, HAsciiString},
};

impl WorkSessionState {
    /// `theitems.FindIndex(item)`: the 1-based ident of a key, 0 when absent.
    pub(crate) fn find_index(&self, item: &Handle<StandardTransient>) -> i32 {
        self.theitems
            .iter()
            .position(|(key, _)| key == item)
            .map_or(0, |p| p as i32 + 1)
    }
    /// `theitems.Add(item, attr)`: the ident of the key, added with `attr` when absent
    /// (an existing attribute is kept).
    fn add_item_entry(
        &mut self,
        item: &Handle<StandardTransient>,
        attr: Handle<StandardTransient>,
    ) -> i32 {
        let id = self.find_index(item);
        if id > 0 {
            return id;
        }
        self.theitems.push((item.clone(), attr));
        self.theitems.len() as i32
    }
    /// The attribute of the ident `id` (1-based, in range).
    fn attribute_mut(&mut self, id: i32) -> &mut Handle<StandardTransient> {
        &mut self.theitems[id as usize - 1].1
    }
}

/// The name as a transient attribute.
fn name_attribute(name: &str) -> Handle<StandardTransient> {
    Handle::new(HAsciiString::new(name)).cast()
}

impl IFSelectWorkSession {
    /// `MaxIdent`: the highest ident (the removed items count).
    pub fn max_ident(&self) -> i32 {
        self.state.read().theitems.len() as i32
    }
    /// `Item(id)`: null out of range or once removed.
    pub fn item(&self, id: i32) -> Handle<StandardTransient> {
        let st = self.state.read();
        let Some((key, attr)) = usize::try_from(id - 1)
            .ok()
            .and_then(|i| st.theitems.get(i))
        else {
            return Handle::default();
        };
        if attr.is_null() {
            Handle::default()
        } else {
            key.clone()
        }
    }
    /// `ItemIdent(item)`: 0 when unknown or removed.
    pub fn item_ident(&self, item: &Handle<StandardTransient>) -> i32 {
        if item.is_null() {
            return 0;
        }
        let st = self.state.read();
        let id = st.find_index(item);
        if id == 0 || st.theitems[id as usize - 1].1.is_null() {
            0
        } else {
            id
        }
    }
    /// `NamedItem(name)`: the item of a name, or of an ident given as "#nnn"; null when
    /// unknown.
    pub fn named_item(&self, name: &str) -> Handle<StandardTransient> {
        if name.is_empty() {
            return Handle::default();
        }
        if let Some(digits) = name.strip_prefix('#') {
            return self.item(atoi(digits.as_bytes())); // #nnn : not a name but an id number
        }
        self.state
            .read()
            .thenames
            .seek(&AsciiString::new(name))
            .cloned()
            .unwrap_or_default()
    }
    /// `NamedItem(name)` for a name as a handle (null for a null name).
    pub fn named_item_h(&self, name: &Handle<HAsciiString>) -> Handle<StandardTransient> {
        match name.get() {
            Some(n) => self.named_item(&String::from_utf8_lossy(n.string().as_bytes())),
            None => Handle::default(),
        }
    }
    /// `NameIdent(name)`: the ident of a name ("#nnn" gives nnn), 0 when unknown.
    pub fn name_ident(&self, name: &str) -> i32 {
        if name.is_empty() {
            return 0;
        }
        if let Some(digits) = name.strip_prefix('#') {
            return atoi(digits.as_bytes()); // #nnn : not a name but an id number
        }
        let item = self
            .state
            .read()
            .thenames
            .seek(&AsciiString::new(name))
            .cloned();
        item.map_or(0, |i| self.item_ident(&i))
    }
    /// The attribute of an item, null when unknown.
    fn attribute(&self, item: &Handle<StandardTransient>) -> Handle<StandardTransient> {
        let st = self.state.read();
        let id = st.find_index(item);
        if id == 0 {
            Handle::default()
        } else {
            st.theitems[id as usize - 1].1.clone()
        }
    }
    /// `HasName(item)`: true when its attribute is a name.
    pub fn has_name(&self, item: &Handle<StandardTransient>) -> bool {
        item.is_some() && self.attribute(item).cast::<HAsciiString>().is_some()
    }
    /// `Name(item)`: its name, null when unnamed or unknown.
    pub fn name(&self, item: &Handle<StandardTransient>) -> Handle<HAsciiString> {
        if item.is_null() {
            return Handle::default();
        }
        self.attribute(item).cast()
    }
    /// `AddItem(item, active = true)`: its ident (the existing one, revived when it was
    /// removed); a dispatch is activated in the share-out when `active`.
    pub fn add_item(&self, item: &Handle<StandardTransient>, active: bool) -> Result<i32, Failure> {
        if item.is_null() {
            return Ok(0);
        }
        let id = {
            let mut st = self.state.write();
            let id = st.find_index(item);
            if id > 0 {
                let attr = st.attribute_mut(id);
                if attr.is_null() {
                    *attr = item.clone();
                }
                id
            } else {
                st.add_item_entry(item, item.clone())
            }
        };
        if active {
            self.set_active(item, true)?;
        }
        Ok(id)
    }
    /// `AddNamedItem(name, item, active = true)`: as `AddItem`, with a name (not starting
    /// with '#' or '!', 0 then); an empty name adds without name.
    pub fn add_named_item(
        &self,
        name: &str,
        item: &Handle<StandardTransient>,
        active: bool,
    ) -> Result<i32, Failure> {
        if item.is_null() || name.starts_with('#') || name.starts_with('!') {
            return Ok(0);
        }
        let id = {
            let mut st = self.state.write();
            if !name.is_empty() {
                st.thenames.bind(AsciiString::new(name), item.clone());
            }
            let id = st.find_index(item);
            if id > 0 {
                let attr = st.attribute_mut(id);
                if attr.is_null() {
                    *attr = item.clone();
                }
                if !name.is_empty() {
                    *attr = name_attribute(name);
                }
                id
            } else if !name.is_empty() {
                st.add_item_entry(item, name_attribute(name))
            } else {
                st.add_item_entry(item, item.clone())
            }
        };
        if active {
            self.set_active(item, true)?;
        }
        Ok(id)
    }
    /// `SetActive(item, mode)`: a dispatch added to (true) or removed from (false, not
    /// yet run, its file root cleared) the share-out; false for other items or no
    /// change. `Standard_NullObject` for a null item (a native null dereference).
    pub fn set_active(
        &self,
        item: &Handle<StandardTransient>,
        mode: bool,
    ) -> Result<bool, Failure> {
        if item.is_null() {
            return Err(exceptions::null_object(
                "IFSelect_WorkSession::SetActive: null item",
            ));
        }
        let disp = item.cast::<dyn IFSelectDispatch>();
        if disp.is_null() {
            return Ok(false);
        }
        let sho = self.share_out();
        let live = sho.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SetActive: no share-out")
        })?;
        let num = live.dispatch_rank(&disp);
        if num > live.nb_dispatches() {
            return Ok(false);
        }
        if mode {
            if num > 0 {
                return Ok(false);
            }
            live.add_dispatch(&disp);
            return Ok(true);
        }
        if num <= live.last_run() {
            return Ok(false);
        }
        live.remove_dispatch(num);
        self.set_file_root(&disp, "")?; // if onlynamed : also cleans ShareOut
        Ok(true)
    }
    /// `RemoveNamedItem(name)`: `RemoveItem` of the named item.
    pub fn remove_named_item(&self, name: &str) -> Result<bool, Failure> {
        let item = self.named_item(name);
        if item.is_null() {
            return Ok(false);
        }
        self.remove_item(&item)
    }
    /// `RemoveName(name)`: the name is forgotten, the item kept. Native quirk kept: the
    /// item keeps its name as attribute (`HasName` and `Name` still give it).
    pub fn remove_name(&self, name: &str) -> bool {
        let item = self.named_item(name);
        if item.is_null() {
            return false;
        }
        let mut st = self.state.write();
        st.add_item_entry(&item, item.clone()); // remains but without name
        st.thenames.unbind(&AsciiString::new(name))
    }
    /// `RemoveItem(item)`: removed from the share-out and from the names; its ident stays
    /// with a null attribute. False when unknown or already removed.
    pub fn remove_item(&self, item: &Handle<StandardTransient>) -> Result<bool, Failure> {
        if item.is_null() {
            return Ok(false);
        }
        let attr = self.attribute(item);
        if attr.is_null() {
            return Ok(false); // unknown or already canceled
        }
        let sho = self.share_out();
        if let Some(live) = sho.get() {
            live.remove_item(item)?;
        }
        let mut st = self.state.write();
        if let Some(name) = attr.cast::<HAsciiString>().get() {
            let key = AsciiString::new(&String::from_utf8_lossy(name.string().as_bytes()));
            if !st.thenames.unbind(&key) {
                return Ok(false);
            }
        }
        let id = st.find_index(item);
        *st.attribute_mut(id) = Handle::default(); // cf ChangeFromIndex
        Ok(true)
    }
    /// `ClearItems`: no item, no name, no dispatch nor modifier in the share-out.
    pub fn clear_items(&self) {
        let sho = {
            let mut st = self.state.write();
            st.thenames.clear(false);
            st.theitems.clear();
            st.theshareout.clone()
        };
        if let Some(live) = sho.get() {
            live.clear(false);
        }
    }
}
