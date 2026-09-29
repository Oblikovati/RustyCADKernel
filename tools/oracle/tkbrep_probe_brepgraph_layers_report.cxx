// The reports of `br_blayer` (tkbrep_probe_brepgraph_layers.cxx).
// The registry: every slot's name, GUID, revision and subscriptions, the masks, the
// slots of the known layer GUIDs and the layer iterator's slots.
void bl_registry(const BRepGraph& g,const BlState& st){
 const BRepGraph_LayerRegistry& reg=g.LayerRegistry();br_int(reg.NbLayers());
 for(uint32_t i=0;i<=reg.NbLayers();++i){occ::handle<BRepGraph_Layer> l=reg.Layer(i);if(l.IsNull()){br_word("N");continue;}
  bl_text(l->Name());bl_guid(l->ID());br_int(l->Revision());br_int(l->SubscribedKinds());br_int(l->SubscribedRefKinds());}
 br_flag(reg.HasModificationSubscribers());br_int(reg.SubscribedKindsMask());
 br_flag(reg.HasRefModificationSubscribers());br_int(reg.SubscribedRefKindsMask());
 for(const Standard_GUID& id:{BRepGraph_LayerHistory::GetID(),BRepGraph_LayerTopoSupplement::GetID(),BRepGraph_LayerDeferred::GetID()}){
  uint32_t slot=0;br_flag(reg.FindSlot(id,slot));br_int(slot);}
 for(BRepGraph_LayerIterator it(reg);it.More();it.Next()){br_int(it.Slot());br_flag(it.Value().IsNull());}
 br_int(int(st.events.size()));for(const auto& l:st.events){br_int(int(l->Events.size()));for(long long e:l->Events)br_int(e);}}
// One history event: the name, sequence, kind, the three mappings in map order, the extra info.
void bl_event(const BRepGraph_LayerHistory::Event& e){
 bl_text(e.OperationName);br_int(int(e.SequenceNumber));br_int(int(e.RecordKind));
 br_int(e.Mapping.Extent());for(NCollection_DataMap<BRepGraph_NodeId,NCollection_LinearVector<BRepGraph_NodeId>>::Iterator it(e.Mapping);it.More();it.Next()){bg_node(it.Key());bl_nodes(it.Value());}
 br_int(e.UidMapping.Extent());for(NCollection_DataMap<BRepGraph_UID,NCollection_LinearVector<BRepGraph_UID>>::Iterator it(e.UidMapping);it.More();it.Next()){bl_print_uid(it.Key());bl_uids(it.Value());}
 br_int(e.ItemUidMapping.Extent());for(NCollection_DataMap<BRepGraph_ItemUID,NCollection_LinearVector<BRepGraph_ItemUID>>::Iterator it(e.ItemUidMapping);it.More();it.Next()){bl_print_item_uid(it.Key());bl_item_uids(it.Value());}
 bl_text(e.ExtraInfo);}
template<class P> void bl_opt_nodes(const P* p){if(p==nullptr)br_word("N");else bl_nodes(*p);}
template<class P> void bl_opt_uids(const P* p){if(p==nullptr)br_word("N");else bl_uids(*p);}
template<class P> void bl_opt_item_uids(const P* p){if(p==nullptr)br_word("N");else bl_item_uids(*p);}
void bl_uid_queries(const BRepGraph& g,const BRepGraph_LayerHistory& h,const BRepGraph_UID& u){
 bl_opt_uids(h.FindModified(u));bl_opt_uids(h.FindGenerated(u));br_flag(h.IsDeleted(u));br_flag(h.HasKnownInput(u));
 bl_uids(h.FindModified(g,u));bl_uids(h.FindGenerated(g,u));br_flag(h.IsDeleted(g,u));}
void bl_item_uid_queries(const BRepGraph_LayerHistory& h,const BRepGraph_ItemUID& u){
 bl_opt_item_uids(h.FindModified(u));bl_opt_item_uids(h.FindGenerated(u));br_flag(h.IsDeleted(u));br_flag(h.HasKnownInput(u));}
// The node queries of every node (one past each kind's end too) and of its UIDs.
void bl_node_queries(const BRepGraph& g,const BRepGraph_LayerHistory& h){
 for(int k=0;k<12;++k){BRepGraph_NodeId::Kind kind=BRepGraph_NodeId::Kind(k);if(!BRepGraph_NodeId::IsValidKind(kind))continue;
  for(uint32_t i=0;i<=g.Topo().Gen().Nb(kind);++i){BRepGraph_NodeId n(kind,i);
   bg_node(h.FindOriginal(n));bl_nodes(h.FindDerived(n));bl_opt_nodes(h.FindModified(n));bl_opt_nodes(h.FindGenerated(n));
   bl_opt_nodes(h.FindOriginals(n));br_flag(h.IsDeleted(n));
   bl_uid_queries(g,h,g.UIDs().Of(n));bl_item_uid_queries(h,g.UIDs().Of(BRepGraph_ItemId(n)));}}}
// The history layer: state, events, the queries of every node and of every UID the
// events name, then the deleted sets in set order.
void bl_history_report(const BRepGraph& g){
 occ::handle<BRepGraph_LayerHistory> h=g.LayerRegistry().Find<BRepGraph_LayerHistory>();if(h.IsNull()){br_word("N");return;}
 br_flag(h->IsEnabled());br_int(h->Revision());br_int(int(h->NbRecords()));
 for(size_t i=0;i<h->NbRecords();++i)bl_event(h->Record(i));
 bl_node_queries(g,*h);
 for(size_t i=0;i<h->NbRecords();++i){const auto& e=h->Record(i);
  for(NCollection_DataMap<BRepGraph_UID,NCollection_LinearVector<BRepGraph_UID>>::Iterator it(e.UidMapping);it.More();it.Next()){
   bl_uid_queries(g,*h,it.Key());for(const auto& u:it.Value())bl_uid_queries(g,*h,u);}
  for(NCollection_DataMap<BRepGraph_ItemUID,NCollection_LinearVector<BRepGraph_ItemUID>>::Iterator it(e.ItemUidMapping);it.More();it.Next()){
   bl_item_uid_queries(*h,it.Key());for(const auto& u:it.Value())bl_item_uid_queries(*h,u);}}
 br_int(h->DeletedNodes().Size());for(const auto& n:h->DeletedNodes())bg_node(n);
 br_int(h->DeletedUids().Size());for(const auto& u:h->DeletedUids())bl_print_uid(u);
 br_int(h->DeletedItemUids().Size());for(const auto& u:h->DeletedItemUids())bl_print_item_uid(u);
 bl_uids(h->DeletedUids(g));}
// The supplement layer: every owner's uids, entries and iterator, then Validate.
void bl_supplement_report(const BrScene& s,const BRepGraph& g){
 occ::handle<BRepGraph_LayerTopoSupplement> l=g.LayerRegistry().Find<BRepGraph_LayerTopoSupplement>();if(l.IsNull()){br_word("N");return;}
 br_int(l->Revision());
 for(int k=0;k<12;++k){BRepGraph_NodeId::Kind kind=BRepGraph_NodeId::Kind(k);if(!BRepGraph_NodeId::IsValidKind(kind))continue;
  for(uint32_t i=0;i<=g.Topo().Gen().Nb(kind);++i){BRepGraph_NodeId n(kind,i);const auto& uids=l->AttachedTo(n);br_int(uids.Size());
   for(uint64_t u:uids){const auto* e=l->FindByUid(u);if(e==nullptr){br_word("N");continue;}br_int(u);bg_node(e->BaseOwner);br_int(e->LocalUid);br_int(int(e->Kind));br_desc(s.graph,e->Shape);}
   for(BRepGraph_SupplementIterator it(g,n);it.More();it.Next()){br_int(it.Uid());br_int(int(it.Value().Kind));}br_word("|");}}
 try{l->Validate();br_word("-");}catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
// The caches: every slot's name and the probe services' entries and generations.
void bl_cache_report(const BRepGraph& g,const BlState& st){
 const BRepGraph_CacheRegistry& reg=g.CacheRegistry();br_int(reg.NbCaches());
 for(BRepGraph_CacheIterator it=reg.CacheIter();it.More();it.Next()){br_int(it.Slot());bl_text(it.Value()->Name());}
 uint32_t slot=0;br_flag(reg.FindSlot(BlCache::GetID(),slot));br_int(slot);
 br_int(int(st.caches.size()));for(const auto& c:st.caches){if(c.IsNull()){br_word("N");continue;}c->Report(g);}}
// The standalone deferred layer: its entries in map order and the kind queries.
void bl_deferred_entry(const BRepGraph_ItemId& i,const BRepGraph_LayerDeferred::Entry& e){
 br_int(int(i.ItemDomain()));br_int(i.RawKind());br_int(i.Index());bl_text(e.Provider);bl_text(e.SourceKey);br_int(int(e.Representations.Size()));
 for(size_t k=0;k<e.Representations.Size();++k){const auto& p=e.Representations.Value(k);br_int(int(p.Kind));br_int(p.Role);bl_text(p.Name);br_int(p.SourceIndex);}}
void bl_deferred_report(const BlState& st){
 const BRepGraph_LayerDeferred& d=*st.deferred;br_int(int(d.Revision()));br_flag(d.HasDeferredItems());
 d.ForEachDeferred([](const BRepGraph_ItemId i,const BRepGraph_LayerDeferred::Entry& e){bl_deferred_entry(i,e);return true;});br_word("|");
 for(int k=0;k<6;++k){BRepGraph_LayerDeferred::RepresentationKind kind=BRepGraph_LayerDeferred::RepresentationKind(k);
  BRepGraph_ItemId first;const auto* e=d.FindFirstDeferred(kind,&first);br_flag(e!=nullptr);if(e!=nullptr)bl_deferred_entry(first,*e);
  int n=0;d.ForEachDeferred(kind,[&n](const BRepGraph_ItemId,const BRepGraph_LayerDeferred::Entry&){++n;return n<2;});br_int(n);}}
// `br_blayer script nactions (action)...`: the actions, then the reports.
void br_blayer_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);BlState st;
 {BRepGraph g;int n=r.integer();for(int i=0;i<n;++i)bl_action(g,s,st,r);
  bl_registry(g,st);bl_history_report(g);bl_supplement_report(s,g);bl_cache_report(g,st);bl_deferred_report(st);bg_shapes(s,g);}
 // After the graph is gone: the logging layers saw their detachment, the caches were cleared.
 for(const auto& l:st.events){br_int(int(l->Events.size()));}for(const auto& c:st.caches)if(!c.IsNull())br_int(c->Clears);
 br_end();}
