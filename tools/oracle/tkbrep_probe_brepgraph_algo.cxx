// BRepGraph algorithms (phase 2, wave 2c, area C1): a graph script drives the
// `br_bedit` actions on one of two graphs and runs BRepGraph_Copy (full, node and
// self copies), BRepGraph_Compact and BRepGraph_Deduplicate, then reports both graphs:
// the definitions, references, UIDs, layers with the history records, the caches and
// the shapes. The Rust twin is crates/tkbrep/examples/br_probe/brep_graph_algo.rs.
#include <BRepGraph_Compact.hxx>
#include <BRepGraph_Copy.hxx>
#include <BRepGraph_Deduplicate.hxx>
// The graphs of a request; the actions run on the current one and copy into the other.
struct BaGraphs{BRepGraph g;BRepGraph t;bool onTarget=false;
 BRepGraph& cur(){return onTarget?t:g;}BRepGraph& other(){return onTarget?g:t;}};
// The copy policies as integers: geometry and mesh 0 Copy 1 Share 2 Drop, cache 0
// Drop 1 CopyFresh.
struct BaPolicies{BRepGraph_Copy::GeomPolicy geom;BRepGraph_Copy::MeshPolicy mesh;BRepGraph_Copy::CachePolicy cache;};
BaPolicies ba_policies(BrReader& r){
 BaPolicies p;p.geom=BRepGraph_Copy::GeomPolicy(r.integer());p.mesh=BRepGraph_Copy::MeshPolicy(r.integer());
 p.cache=BRepGraph_Copy::CachePolicy(r.integer());return p;}
void ba_compact_result(const BRepGraph_Compact::Result& x){
 for(uint32_t v:{x.NbRemovedVertices,x.NbRemovedEdges,x.NbRemovedWires,x.NbRemovedFaces,x.NbRemovedShells,
   x.NbRemovedSolids,x.NbRemovedCompounds,x.NbRemovedCompSolids,x.NbRemovedSurfaces,x.NbRemovedCurves,
   x.NbNodesBefore,x.NbNodesAfter,x.NbUnmappedActiveDefs})br_int(v);}
void ba_dedup_result(const BRepGraph_Deduplicate::Result& x){
 for(uint32_t v:{x.NbCanonicalSurfaces,x.NbCanonicalCurves,x.NbSurfaceRewrites,x.NbCurveRewrites,
   x.NbNullifiedSurfaces,x.NbNullifiedCurves,x.NbHistoryRecords})br_int(v);
 br_flag(x.IsEntityMergeApplied);
 for(uint32_t v:{x.NbMergedVertices,x.NbMergedEdges,x.NbMergedWires,x.NbMergedFaces,x.NbReorderedWires,
   x.NbToleranceOrderedWires,x.NbPartialOrderedWires})br_int(v);
 bg_list(x.AffectedFaces);bg_list(x.AffectedEdges);}
// 240-246: 240 Copy::Perform(current, other, policies), 241 CopyNode(current, other,
// node, policies), 242 CopyNode(current, current, node, policies), 243
// Compact::Perform(current, history, cache mode), 244 Deduplicate::Perform(current,
// analyze, history, merge, comp tolerance, hash tolerance), 245 switch the current
// graph, 246 Copy::Perform(current, current).
bool ba_action(BaGraphs& gs,BrReader& r,int code){
 switch(code){
  case 240:{BaPolicies p=ba_policies(r);br_flag(BRepGraph_Copy::Perform(gs.cur(),gs.other(),p.geom,p.mesh,p.cache));return true;}
  case 241:{BRepGraph_NodeId n=bl_node(r);BaPolicies p=ba_policies(r);bg_node(BRepGraph_Copy::CopyNode(gs.cur(),gs.other(),n,p.geom,p.mesh,p.cache));return true;}
  case 242:{BRepGraph_NodeId n=bl_node(r);BaPolicies p=ba_policies(r);bg_node(BRepGraph_Copy::CopyNode(gs.cur(),gs.cur(),n,p.geom,p.mesh,p.cache));return true;}
  case 243:{BRepGraph_Compact::Options o;o.HistoryMode=r.flag();o.CacheMode=BRepGraph_Compact::Options::CachePolicy(r.integer());
   ba_compact_result(BRepGraph_Compact::Perform(gs.cur(),o));return true;}
  case 244:{BRepGraph_Deduplicate::Options o;o.AnalyzeOnly=r.flag();o.HistoryMode=r.flag();o.MergeEntitiesWhenSafe=r.flag();
   o.CompTolerance=r.real();o.HashTolerance=r.real();ba_dedup_result(BRepGraph_Deduplicate::Perform(gs.cur(),o));return true;}
  case 245:gs.onTarget=!gs.onTarget;br_word("-");return true;
  case 246:br_flag(BRepGraph_Copy::Perform(gs.cur(),gs.cur()));return true;
  default:return false;}}
// One graph's report: the `br_bedit` reports with the node and reference UIDs.
void ba_report(const BrScene& s,BRepGraph& g,const BlState& st){
 br_flag(g.IsEmpty());br_flag(g.ValidateRelations());bg_list(g.RootProductIds());
 bg_topo_report(s,g);bg_ref_entries(s,g);bg_ref_groups(g);bg_node_ids(g);bg_ref_ids(s,g);
 br_flag(g.Editor().IsDeferredMode());
 bl_registry(g,st);bl_history_report(g);bl_supplement_report(s,g);bl_cache_report(g,st);
 bc_mesh_report(s,g);bc_view_report(s,g);bg_shapes(s,g);}
// `br_balgo script nactions (action)...`: the actions, then both graphs' reports and,
// after the graphs are gone, every logging layer's event count.
void br_balgo_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);BlState st;
 {BaGraphs gs;int n=r.integer();
  for(int i=0;i<n;++i){int code=r.integer();
   try{if(ba_action(gs,r,code))continue;}catch(const Standard_Failure& e){br_word(e.ExceptionType());continue;}
   be_action_code(gs.cur(),s,st,r,code);}
  ba_report(s,gs.g,st);ba_report(s,gs.t,st);}
 for(const auto& l:st.events){br_int(int(l->Events.size()));}
 br_end();}
