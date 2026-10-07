// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ShareOutResult (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_ShareOutResult`: the packets a share-out (or one dispatch) gives on a
//! graph, iterated packet by packet with the dispatch that produced each.
use super::{IFSelectDispatch, IFSelectPacketList, IFSelectShareOut};
use crate::if_graph::{IFGraphAllShared, IFGraphSubPartsEvaluate, IFGraphSubPartsIterator};
use crate::interface::{
    InterfaceEntityIterator, InterfaceGraph, InterfaceGraphContent, InterfaceGraphContentEvaluate,
    InterfaceInterfaceModel,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, exceptions},
    tcollection::AsciiString,
};

/// `IFSelect_ShareOutResult`.
pub struct IFSelectShareOutResult {
    thegraph: InterfaceGraph,
    thedispres: IFGraphSubPartsIterator,
    theshareout: Handle<IFSelectShareOut>,
    thedispatch: Handle<dyn IFSelectDispatch>,
    theeval: bool,
    thedispnum: i32,
    thepacknum: i32,
    thepackdisp: i32,
    thenbindisp: i32,
    thedisplist: Vec<i32>,
}

impl IFSelectShareOutResult {
    /// The common constructor on a graph.
    fn build(
        g: &InterfaceGraph,
        sho: Handle<IFSelectShareOut>,
        disp: Handle<dyn IFSelectDispatch>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            thegraph: InterfaceGraph::from_graph(g, false)?,
            thedispres: IFGraphSubPartsIterator::new(g, false)?,
            theshareout: sho,
            thedispatch: disp,
            theeval: false,
            thedispnum: 0,
            thepacknum: 0,
            thepackdisp: 0,
            thenbindisp: 0,
            thedisplist: Vec::new(),
        })
    }
    /// `IFSelect_ShareOutResult(sho, mod)`: on a new graph of the model.
    pub fn with_share_out(
        sho: &Handle<IFSelectShareOut>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<Self, Failure> {
        Self::build(
            &InterfaceGraph::new(model, true)?,
            sho.clone(),
            Handle::default(),
        )
    }
    /// `IFSelect_ShareOutResult(sho, G)`.
    pub fn with_share_out_graph(
        sho: &Handle<IFSelectShareOut>,
        g: &InterfaceGraph,
    ) -> Result<Self, Failure> {
        Self::build(g, sho.clone(), Handle::default())
    }
    /// `IFSelect_ShareOutResult(disp, mod)`: on a new graph of the model.
    pub fn with_dispatch(
        disp: &Handle<dyn IFSelectDispatch>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<Self, Failure> {
        Self::build(
            &InterfaceGraph::new(model, true)?,
            Handle::default(),
            disp.clone(),
        )
    }
    /// `IFSelect_ShareOutResult(disp, G)`.
    pub fn with_dispatch_graph(
        disp: &Handle<dyn IFSelectDispatch>,
        g: &InterfaceGraph,
    ) -> Result<Self, Failure> {
        Self::build(g, Handle::default(), disp.clone())
    }
    /// `ShareOut`.
    pub fn share_out(&self) -> Handle<IFSelectShareOut> {
        self.theshareout.clone()
    }
    /// `Graph`.
    pub fn graph(&self) -> &InterfaceGraph {
        &self.thegraph
    }
    /// `Reset`: to be evaluated again.
    pub fn reset(&mut self) {
        self.theeval = false;
    }
    /// `Evaluate`: `Prepare` once.
    pub fn evaluate(&mut self) -> Result<(), Failure> {
        if self.theeval {
            return Ok(());
        }
        self.prepare()?;
        self.theeval = true;
        Ok(())
    }
    /// `Packets(complete = true)`: the packets (their contents, or roots).
    pub fn packets(&mut self, complete: bool) -> Result<IFSelectPacketList, Failure> {
        self.evaluate()?;
        let list = IFSelectPacketList::new(self.thegraph.model())?;
        while self.more()? {
            list.add_packet()?;
            let content = if complete {
                self.packet_content()?
            } else {
                self.packet_root()?
            };
            list.add_list(Some(&content.content()))?;
            self.next_packet()?;
        }
        Ok(list)
    }
    /// `NbPackets`.
    pub fn nb_packets(&mut self) -> Result<i32, Failure> {
        self.evaluate()?;
        Ok(self.thedispres.nb_parts())
    }
    /// The dispatch `i` of the share-out, else the own dispatch.
    fn dispatch_at(&self, i: i32) -> Result<Handle<dyn IFSelectDispatch>, Failure> {
        match self.theshareout.get() {
            Some(sho) => sho.dispatch(i),
            None => Ok(self.thedispatch.clone()),
        }
    }
    /// `Prepare`: the packets of the dispatches not yet run, recorded as parts. Native
    /// defect kept: `Reset` of the parts drops the statuses of their graph, so no entity
    /// can be recorded in the parts afterwards.
    pub fn prepare(&mut self) -> Result<(), Failure> {
        self.thedisplist.clear();
        self.thedispres.reset()?;
        let mut all = IFGraphAllShared::new(&self.thegraph)?;
        let (first, nb) = match self.theshareout.get() {
            Some(sho) => (sho.last_run() + 1, sho.nb_dispatches()),
            None => (1, 1),
        };
        for i in first..=nb {
            let disp = self.dispatch_at(i)?;
            let live = disp.get().ok_or_else(|| {
                exceptions::null_object("IFSelect_ShareOutResult::Prepare: null dispatch")
            })?;
            if live.final_selection().is_null() {
                continue;
            }
            let mut packs = IFGraphSubPartsIterator::new(&self.thegraph, false)?;
            live.packets(&self.thegraph, &mut packs)?;
            packs.start()?;
            while packs.more()? {
                let iter = packs.entities()?;
                packs.next();
                if iter.nb_entities() == 0 {
                    continue;
                }
                self.thedispres.add_part();
                self.thedispres.get_from_iter(&iter)?;
                all.reset_data()?;
                all.get_from_iter(&iter)?;
                self.thedisplist.push(i);
            }
        }
        (self.thedispnum, self.thepacknum, self.thepackdisp) = (1, 1, 1);
        self.thenbindisp = self.count_in_dispatch(1);
        Ok(())
    }
    /// The count of packets of the current dispatch from packet `from`.
    fn count_in_dispatch(&self, from: i32) -> i32 {
        self.thedisplist
            .iter()
            .skip((from - 1).max(0) as usize)
            .take_while(|d| **d == self.thedispnum)
            .count() as i32
    }
    /// `More`.
    pub fn more(&mut self) -> Result<bool, Failure> {
        self.thedispres.more()
    }
    /// `Next`: the next packet (and its dispatch).
    pub fn next_packet(&mut self) -> Result<(), Failure> {
        self.thedispres.next();
        self.thepacknum += 1;
        let Some(dispnum) = usize::try_from(self.thepacknum - 1)
            .ok()
            .and_then(|i| self.thedisplist.get(i))
            .copied()
        else {
            self.thenbindisp = 0;
            return Ok(());
        };
        if self.thedispnum == dispnum {
            self.thepackdisp += 1;
            return Ok(());
        }
        self.enter_dispatch(dispnum)
    }
    /// The first packet of the dispatch `dispnum`.
    fn enter_dispatch(&mut self, dispnum: i32) -> Result<(), Failure> {
        (self.thedispnum, self.thepackdisp) = (dispnum, 1);
        self.thenbindisp = self.count_in_dispatch(self.thepacknum);
        if let Some(sho) = self.theshareout.get() {
            self.thedispatch = sho.dispatch(self.thedispnum)?;
        }
        Ok(())
    }
    /// `NextDispatch`: the first packet of the next dispatch.
    pub fn next_dispatch(&mut self) -> Result<(), Failure> {
        while self.thepacknum <= self.thedisplist.len() as i32 {
            self.thedispres.next();
            let value = self.thedisplist[self.thepacknum as usize - 1];
            if self.thedispnum != value {
                return self.enter_dispatch(value);
            }
            self.thepacknum += 1;
        }
        self.thepacknum = self.thedisplist.len() as i32 + 1;
        (self.thedispnum, self.thepackdisp, self.thenbindisp) = (0, 0, 0);
        Ok(())
    }
    /// `Dispatch`.
    pub fn dispatch(&self) -> Handle<dyn IFSelectDispatch> {
        self.thedispatch.clone()
    }
    /// `DispatchRank`.
    pub fn dispatch_rank(&self) -> i32 {
        self.thedispnum
    }
    /// `PacketsInDispatch(numpack, nbpacks)`: the rank of the packet in its dispatch and
    /// the count of packets of the dispatch.
    pub fn packets_in_dispatch(&self) -> (i32, i32) {
        (self.thepackdisp, self.thenbindisp)
    }
    /// `PacketRoot`: the root entities of the current packet.
    pub fn packet_root(&self) -> Result<InterfaceEntityIterator, Failure> {
        self.thedispres.entities()
    }
    /// `PacketContent`: the current packet with the entities it shares.
    pub fn packet_content(&self) -> Result<InterfaceEntityIterator, Failure> {
        let iter = self.thedispres.entities()?;
        let mut graph = InterfaceGraph::from_graph(&self.thegraph, false)?;
        for ent in iter.content() {
            graph.get_from_entity(&ent, true, 0)?;
        }
        InterfaceGraphContent::from_graph(&graph)?.result()
    }
    /// `FileName`: the name of the current packet file; `Standard_NullObject` without
    /// share-out (a native null dereference).
    pub fn file_name(&self) -> Result<AsciiString, Failure> {
        let sho = self.theshareout.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_ShareOutResult::FileName: no share-out")
        })?;
        let (np, nbp) = self.packets_in_dispatch();
        sho.file_name(self.dispatch_rank(), np, nbp)
    }
}
