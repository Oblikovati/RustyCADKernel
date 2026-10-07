// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_PacketList (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_PacketList`: the packets of a share-out (lists of entity numbers of a
//! model), with the count of packets each entity is in.
use crate::interface::{
    InterfaceEntityIterator, InterfaceIntList, InterfaceInterfaceModel, InterfaceModelEntities,
    interface_error,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::AsciiString,
};

/// The fields of `IFSelect_PacketList`.
struct PacketState {
    thedupls: Vec<i32>,
    thepacks: InterfaceIntList,
    theflags: Vec<i32>,
    thelast: i32,
    thebegin: bool,
    thename: AsciiString,
}

/// `IFSelect_PacketList`.
pub struct IFSelectPacketList {
    transient: StandardTransient,
    themodel: Handle<dyn InterfaceInterfaceModel>,
    state: SharedState<PacketState>,
}
impl_transient!(
    IFSelectPacketList,
    "IFSelect_PacketList",
    StandardTransient,
    transient
);

impl IFSelectPacketList {
    /// `IFSelect_PacketList(model)`: named "Packets"; `Standard_NullObject` for a null
    /// model (a native null dereference).
    pub fn new(model: &Handle<dyn InterfaceInterfaceModel>) -> Result<Self, Failure> {
        let live = model.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_PacketList: null model; expected a model")
        })?;
        let nb = live.nb_entities().max(0) as usize + 1;
        let state = PacketState {
            thedupls: vec![0; nb],
            thepacks: InterfaceIntList::with_entities(100)?,
            theflags: vec![0; nb],
            thelast: 0,
            thebegin: false,
            thename: AsciiString::new("Packets"),
        };
        Ok(Self {
            transient: StandardTransient::default(),
            themodel: model.clone(),
            state: SharedState::new(state),
        })
    }
    /// `SetName(name)`.
    pub fn set_name(&self, name: &str) {
        self.state.write().thename = AsciiString::new(name);
    }
    /// `Name`.
    pub fn name(&self) -> AsciiString {
        self.state.read().thename.clone()
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.themodel.clone()
    }
    /// The live model (never null after the constructor).
    fn live_model(&self) -> Result<&dyn InterfaceInterfaceModel, Failure> {
        self.themodel
            .get()
            .ok_or_else(|| exceptions::null_object("IFSelect_PacketList: no model"))
    }
    /// `AddPacket`: a new current packet.
    pub fn add_packet(&self) -> Result<(), Failure> {
        let mut st = self.state.write();
        st.theflags.iter_mut().skip(1).for_each(|f| *f = 0);
        let nbl = st.thepacks.nb_entities();
        if st.thelast >= nbl {
            st.thepacks.set_nb_entities(nbl * 2)?;
        }
        if !st.thebegin {
            st.thelast += 1;
        }
        let last = st.thelast;
        st.thepacks.set_number(last)?;
        st.thebegin = false;
        Ok(())
    }
    /// `Add(ent)`: an entity of the model in the current packet (once);
    /// `Interface_InterfaceError` out of the model or before the first packet.
    pub fn add(&self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        let num = self.live_model()?.number(ent);
        if num == 0 {
            return Err(interface_error("PacketList:Add, Entity not in Model"));
        }
        let mut st = self.state.write();
        if st.thelast == 0 {
            return Err(interface_error("PacketList:Add, no Packet yet added"));
        }
        let index = num as usize;
        if st.theflags[index] != 0 {
            return Ok(());
        }
        st.theflags[index] = 1;
        st.thedupls[index] += 1;
        st.thepacks.add(num)?;
        st.thebegin = false;
        Ok(())
    }
    /// `AddList(list)`: `Add` of each entity.
    pub fn add_list(&self, list: Option<&[Handle<StandardTransient>]>) -> Result<(), Failure> {
        let Some(list) = list else { return Ok(()) };
        self.state
            .write()
            .thepacks
            .reservate(list.len() as i32 + 1)?;
        list.iter().try_for_each(|ent| self.add(ent))
    }
    /// `NbPackets`.
    pub fn nb_packets(&self) -> i32 {
        let st = self.state.read();
        if st.thebegin {
            st.thelast - 1
        } else {
            st.thelast
        }
    }
    /// The entity numbers of a packet (none out of range).
    fn packet(&self, numpack: i32) -> Result<Vec<i32>, Failure> {
        if numpack <= 0 || numpack > self.nb_packets() {
            return Ok(Vec::new());
        }
        let mut lisi = InterfaceIntList::from_other(&self.state.read().thepacks, false)?;
        lisi.set_number(numpack)?;
        (1..=lisi.length()).map(|i| lisi.value(i)).collect()
    }
    /// `NbEntities(numpack)`: 0 out of range.
    pub fn nb_entities(&self, numpack: i32) -> Result<i32, Failure> {
        Ok(self.packet(numpack)?.len() as i32)
    }
    /// `Entities(numpack)`: empty out of range.
    pub fn entities(&self, numpack: i32) -> Result<InterfaceEntityIterator, Failure> {
        let model = self.live_model()?;
        let mut list = InterfaceEntityIterator::new();
        for num in self.packet(numpack)? {
            list.add_item(&model.value(num)?);
        }
        Ok(list)
    }
    /// `HighestDuplicationCount`: the highest count of packets of an entity.
    pub fn highest_duplication_count(&self) -> i32 {
        self.state
            .read()
            .thedupls
            .iter()
            .skip(1)
            .copied()
            .max()
            .unwrap_or(0)
            .max(0)
    }
    /// The numbers of the entities in `count` packets (or more with `andmore`).
    fn duplicated_numbers(&self, count: i32, andmore: bool) -> Vec<i32> {
        let st = self.state.read();
        st.thedupls
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, j)| **j == count || (**j > count && andmore))
            .map(|(i, _)| i as i32)
            .collect()
    }
    /// `NbDuplicated(count, andmore)`.
    pub fn nb_duplicated(&self, count: i32, andmore: bool) -> i32 {
        self.duplicated_numbers(count, andmore).len() as i32
    }
    /// `Duplicated(count, andmore)`.
    pub fn duplicated(
        &self,
        count: i32,
        andmore: bool,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let model = self.live_model()?;
        let mut list = InterfaceEntityIterator::new();
        for num in self.duplicated_numbers(count, andmore) {
            list.add_item(&model.value(num)?);
        }
        Ok(list)
    }
}
