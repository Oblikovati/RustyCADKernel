// The actions of `br_blayer` (tkbrep_probe_brepgraph_layers.cxx). Codes 0-5 are the
// `br_bgraph` ingestion actions; each action prints its result or the exception.
occ::handle<BRepGraph_LayerHistory> bl_history(BRepGraph& g){return g.LayerRegistry().Ensure<BRepGraph_LayerHistory>();}
occ::handle<BRepGraph_LayerTopoSupplement> bl_supplement(BRepGraph& g){return g.LayerRegistry().Ensure<BRepGraph_LayerTopoSupplement>();}
BRepGraph_LayerHistory::Kind bl_kind(BrReader& r){return BRepGraph_LayerHistory::Kind(r.integer());}
// 10-20: the history layer. 10 Record, 11 RecordBatch, 12 RecordDeleted,
// 13 RecordReplaced, 14 RecordReplacedBatch, 15 RecordUid, 16 RecordDeletedUid,
// 17 RecordItemUid, 18 RecordDeletedItemUid, 19 SetEnabled, 20 Clear.
bool bl_history_action(BRepGraph& g,BrReader& r,int code){
 if(code<10||code>20)return false;
 occ::handle<BRepGraph_LayerHistory> h=bl_history(g);
 switch(code){
  case 10:{TCollection_AsciiString l=bl_label(r);BRepGraph_NodeId o=bl_node(r);auto v=bl_list<BRepGraph_NodeId>(r,bl_node);h->Record(l,o,v.ToArray1(),bl_kind(r));break;}
  case 11:{TCollection_AsciiString l=bl_label(r);auto o=bl_list<BRepGraph_NodeId>(r,bl_node);auto v=bl_list<BRepGraph_NodeId>(r,bl_node);
   TCollection_AsciiString x=bl_label(r);h->RecordBatch(l,o.ToArray1(),v.ToArray1(),x,bl_kind(r));break;}
  case 12:{TCollection_AsciiString l=bl_label(r);h->RecordDeleted(l,bl_list<BRepGraph_NodeId>(r,bl_node).ToArray1());break;}
  case 13:{TCollection_AsciiString l=bl_label(r);BRepGraph_NodeId o=bl_node(r);h->RecordReplaced(l,o,bl_node(r));break;}
  case 14:{TCollection_AsciiString l=bl_label(r);auto o=bl_list<BRepGraph_NodeId>(r,bl_node);auto v=bl_list<BRepGraph_NodeId>(r,bl_node);
   h->RecordReplacedBatch(l,o.ToArray1(),v.ToArray1(),bl_label(r));break;}
  case 15:{TCollection_AsciiString l=bl_label(r);BRepGraph_UID o=bl_uid(r);auto v=bl_list<BRepGraph_UID>(r,bl_uid);h->RecordUid(l,o,v.ToArray1(),bl_kind(r));break;}
  case 16:{TCollection_AsciiString l=bl_label(r);h->RecordDeletedUid(l,bl_list<BRepGraph_UID>(r,bl_uid).ToArray1());break;}
  case 17:{TCollection_AsciiString l=bl_label(r);BRepGraph_ItemUID o=bl_item_uid(r);auto v=bl_list<BRepGraph_ItemUID>(r,bl_item_uid);
   h->RecordItemUid(l,o,v.ToArray1(),bl_kind(r));break;}
  case 18:{TCollection_AsciiString l=bl_label(r);h->RecordDeletedItemUid(l,bl_list<BRepGraph_ItemUID>(r,bl_item_uid).ToArray1());break;}
  case 19:h->SetEnabled(r.flag());break;
  case 20:h->Clear();break;
  default:return false;}
 br_int(int(h->NbRecords()));return true;}
// 21-29: registry dispatch and registration. 21 DispatchOnNodeRemoved, 22
// DispatchOnNodeReplaced, 23 DispatchOnRefRemoved, 24 DispatchNodeModified, 25
// DispatchNodesModified, 26 DispatchRefModified, 27 InvalidateAll, 28 ClearAll,
// 29 register a logging layer (index, kinds, ref kinds), 30 unregister by slot.
bool bl_registry_action(BRepGraph& g,BlState& st,BrReader& r,int code){
 BRepGraph_LayerRegistry& reg=g.LayerRegistry();
 switch(code){
  case 21:reg.DispatchOnNodeRemoved(bl_node(r));break;
  case 22:{BRepGraph_NodeId o=bl_node(r);reg.DispatchOnNodeReplaced(o,bl_node(r));break;}
  case 23:reg.DispatchOnRefRemoved(bl_ref(r));break;
  case 24:reg.DispatchNodeModified(bl_node(r));break;
  case 25:{auto v=bl_list<BRepGraph_NodeId>(r,bl_node);reg.DispatchNodesModified(v.ToArray1(),r.integer());break;}
  case 26:reg.DispatchRefModified(bl_ref(r));break;
  case 27:reg.InvalidateAll();break;
  case 28:reg.ClearAll();break;
  case 29:{int i=r.integer();int k=r.integer();occ::handle<BlEventLayer> l=new BlEventLayer(i,k,r.integer());
   st.events.push_back(l);br_int(reg.RegisterLayer(l));break;}
  case 30:{occ::handle<BRepGraph_Layer> l=reg.Layer(uint32_t(r.integer()));if(!l.IsNull())reg.UnregisterLayer(l->ID());break;}
  default:return false;}
 br_int(reg.NbLayers());return true;}
// 31-34: the supplement layer. 31 AddAttachment (owner, kind, shape), 32
// AddAttachmentWithUid (owner, uid, kind, shape), 33 RemoveAttachment (uid), 34 Ensure.
bool bl_supplement_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 if(code<31||code>34)return false;
 occ::handle<BRepGraph_LayerTopoSupplement> l=bl_supplement(g);
 using AK=BRepGraph_LayerTopoSupplement::AttachmentKind;
 switch(code){
  case 31:{BRepGraph_NodeId o=bl_node(r);AK k=AK(r.integer());br_int(l->AddAttachment(o,k,s.graph.shape(r.integer())));break;}
  case 32:{BRepGraph_NodeId o=bl_node(r);uint64_t u=uint64_t(r.integer());AK k=AK(r.integer());br_flag(l->AddAttachmentWithUid(o,u,k,s.graph.shape(r.integer())));break;}
  case 33:br_flag(l->RemoveAttachment(uint64_t(r.integer())));break;
  case 34:br_int(l->Revision());break;
  default:return false;}
 return true;}
// 35-36: history import. 35 Shapes().AddWithHistory (shape, input roots, history
// operations, option bits), 36 Absorb(g, g, inputs of roots, outputs of roots, history).
bool bl_import_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 const TCollection_AsciiString l("Import");
 if(code==35){TopoDS_Shape sh=s.graph.shape(r.integer());auto roots=bl_list<BRepGraph_NodeId>(r,bl_node);
  occ::handle<BRepTools_History> h=bl_history_ops(s,r);int bits=r.integer();
  bg_add_result(bits<0?g.Shapes().AddWithHistory(sh,roots.ToArray1(),h,l):g.Shapes().AddWithHistory(sh,roots.ToArray1(),h,l,bg_options(bits)));return true;}
 if(code==36){auto in=bl_inputs(g,r);auto out=bl_inputs(g,r);occ::handle<BRepTools_History> h=bl_history_ops(s,r);
  bl_history(g)->Absorb(g,g,in,out,h,l);br_int(int(bl_history(g)->NbRecords()));return true;}
 return false;}
// 40-45: the probe cache. 40 RegisterCache (new service), 41 Ensure, 42 UnregisterCache,
// 43 bind an entry (type, id) in the last service, 44 ClearAll, 45 Clear.
bool bl_cache_action(BRepGraph& g,BlState& st,BrReader& r,int code){
 BRepGraph_CacheRegistry& reg=g.CacheRegistry();
 switch(code){
  case 40:{occ::handle<BlCache> c=new BlCache();st.caches.push_back(c);br_int(reg.RegisterCache(c));break;}
  case 41:{occ::handle<BlCache> c=reg.Ensure<BlCache>();st.caches.push_back(c);br_flag(c.IsNull());break;}
  case 42:reg.UnregisterCache(BlCache::GetID());break;
  case 43:{int t=r.integer();br_flag(!st.caches.empty()&&st.caches.back()->Bind(t,r));break;}
  case 44:reg.ClearAll();break;
  case 45:reg.Clear();break;
  default:return false;}
 br_int(reg.NbCaches());return true;}
// 50-58: a standalone (detached) deferred layer. 50 RegisterDeferredRepresentations
// (item, provider, key, count, (kind role name index)...), 51 the Direct variant,
// 52 UnregisterDeferred, 53 BeginBulkRegistration, 54 EndBulkRegistration,
// 55 OnNodeReplaced, 56 OnRefRemoved, 57 Clear, 58 ReserveDeferredItems.
NCollection_LinearVector<BRepGraph_LayerDeferred::Representation> bl_representations(BrReader& r){
 return bl_list<BRepGraph_LayerDeferred::Representation>(r,[](BrReader& x){BRepGraph_LayerDeferred::Representation p;
  p.Kind=BRepGraph_LayerDeferred::RepresentationKind(x.integer());p.Role=uint32_t(x.integer());p.Name=bl_label(x);p.SourceIndex=uint32_t(x.real());return p;});}
bool bl_deferred_action(BlState& st,BrReader& r,int code){
 const occ::handle<BRepGraph_LayerDeferred>& d=st.deferred;
 switch(code){
  case 50:case 51:{BRepGraph_ItemId i=bl_item(r);TCollection_AsciiString p=bl_label(r),k=bl_label(r);auto v=bl_representations(r);
   if(code==50)d->RegisterDeferredRepresentations(i,p,k,v.IsEmpty()?nullptr:&v.First(),v.Size());
   else d->RegisterDeferredRepresentationsDirect(i,p,k,v.IsEmpty()?nullptr:&v.First(),v.Size());break;}
  case 52:d->UnregisterDeferred(bl_item(r));break;
  case 53:d->BeginBulkRegistration();break;
  case 54:d->EndBulkRegistration();break;
  case 55:{BRepGraph_NodeId o=bl_node(r);d->OnNodeReplaced(o,bl_node(r));break;}
  case 56:d->OnRefRemoved(bl_ref(r));break;
  case 57:d->Clear();break;
  case 58:d->ReserveDeferredItems(size_t(r.integer()));break;
  default:return false;}
 br_int(int(d->Revision()));return true;}
// 60 Graph Clear().
void bl_action(BRepGraph& g,const BrScene& s,BlState& st,BrReader& r){
 int code=r.integer();
 try{
  if(code<6){bg_run(g,s,r,code);return;}
  if(code==60){g.Clear();br_word("-");return;}
  if(bl_history_action(g,r,code)||bl_registry_action(g,st,r,code)||bl_supplement_action(g,s,r,code)
     ||bl_import_action(g,s,r,code)||bl_cache_action(g,st,r,code)||bl_deferred_action(st,r,code))return;
  br_word("?");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
