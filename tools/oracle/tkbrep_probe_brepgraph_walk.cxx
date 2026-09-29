// BRepGraph oracle, part 3: the navigation classes over a graph built by the br_bgraph
// actions and a list of assembly edits: the dense definition and reference iterators,
// the one-level defs and refs iterators of every parent, the reverse iterators of
// every child and the related iterator of every node. The Rust twin is
// crates/tkbrep/examples/br_probe/brep_graph_walk.rs.
#include <BRepGraph_DefsIterator.hxx>
#include <BRepGraph_EditorView.hxx>
#include <BRepGraph_Iterator.hxx>
#include <BRepGraph_RefsIterator.hxx>
#include <BRepGraph_RelatedIterator.hxx>
#include <BRepGraph_ReverseIterator.hxx>
// The native nodes of every valid kind, in kind then index order.
std::vector<BRepGraph_NodeId> bw_nodes(const BRepGraph& g){
 std::vector<BRepGraph_NodeId> nodes;
 for(int k=0;k<12;++k){BRepGraph_NodeId::Kind kind=BRepGraph_NodeId::Kind(k);if(!BRepGraph_NodeId::IsValidKind(kind))continue;
  for(uint32_t i=0;i<g.Topo().Gen().Nb(kind);++i)nodes.push_back(BRepGraph_NodeId(kind,i));}
 return nodes;}
void bw_id(const BRepGraph_NodeId& n){bg_node(n);}
template<class T> void bw_id(const T& id){br_int(id.Index);}
// Assembly edits: 0 Products().Add(root kind, index, location), 1 Products().Add(),
// 2 AppendDocumentRoot(product), 3 Products().Append(parent, child, location, parent occurrence).
void bw_edit(BrScene& s,BRepGraph& g,BrReader& r){
 int code=r.integer();
 try{
  switch(code){
   case 0:{int k=r.integer(),i=r.integer();const TopLoc_Location& l=s.graph.loc(r.integer());
    br_int(g.Editor().Products().Add(BRepGraph_NodeId(BRepGraph_NodeId::Kind(k),i),l).Index);break;}
   case 1:br_int(g.Editor().Products().Add().Index);break;
   case 2:g.Editor().Products().AppendDocumentRoot(BRepGraph_ProductId(r.integer()));br_word("-");break;
   default:{int p=r.integer(),c=r.integer();const TopLoc_Location& l=s.graph.loc(r.integer());int o=r.integer();
    BRepGraph_OccurrenceRefId ref;BRepGraph_OccurrenceId occ=g.Editor().Products().Append(BRepGraph_ProductId(p),BRepGraph_ProductId(c),l,
     o<0?BRepGraph_OccurrenceId():BRepGraph_OccurrenceId(o),&ref);br_int(occ.Index);br_int(ref.Index);break;}}
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
// The dense iterators: the skip-removed order, the full count and a start at 1.
template<class Def> void bw_iter(const BRepGraph& g){
 using It=BRepGraph_Iterator<Def>;int n=0;
 for(It it(g);it.More();it.Next()){br_int(it.CurrentId().Index);++n;}br_int(n);
 n=0;for(BRepGraph_Iterator<Def,true> it(g);it.More();it.Next())++n;br_int(n);
 It from(g,typename It::TypedId(1));br_flag(from.More());if(from.More())br_int(from.CurrentId().Index);}
template<class Ref> void bw_ref_iter(const BRepGraph& g){
 using It=BRepGraph_RefsIterator::RefIterator<Ref>;int n=0;
 for(It it(g);it.More();it.Next()){br_int(it.CurrentId().Index);++n;}br_int(n);
 n=0;for(BRepGraph_RefsIterator::RefIterator<Ref,true> it(g);it.More();it.Next())++n;br_int(n);
 It from(g,typename It::RefId(1));br_flag(from.More());if(from.More())br_int(from.CurrentId().Index);}
void bw_dense(const BRepGraph& g){
 bw_iter<BRepGraphInc::SolidDef>(g);bw_iter<BRepGraphInc::ShellDef>(g);bw_iter<BRepGraphInc::FaceDef>(g);
 bw_iter<BRepGraphInc::WireDef>(g);bw_iter<BRepGraphInc::EdgeDef>(g);bw_iter<BRepGraphInc::VertexDef>(g);
 bw_iter<BRepGraphInc::CoEdgeDef>(g);bw_iter<BRepGraphInc::CompoundDef>(g);bw_iter<BRepGraphInc::CompSolidDef>(g);
 bw_iter<BRepGraphInc::ProductDef>(g);bw_iter<BRepGraphInc::OccurrenceDef>(g);
 bw_ref_iter<BRepGraphInc::ShellRef>(g);bw_ref_iter<BRepGraphInc::FaceRef>(g);bw_ref_iter<BRepGraphInc::WireRef>(g);
 bw_ref_iter<BRepGraphInc::VertexRef>(g);bw_ref_iter<BRepGraphInc::SolidRef>(g);bw_ref_iter<BRepGraphInc::ChildRef>(g);
 bw_ref_iter<BRepGraphInc::OccurrenceRef>(g);
 for(BRepGraph_RootProductIterator it(g);it.More();it.Next())br_int(it.Current().Index);br_end();}
// One parent's defs and refs walks: `count (child ref index)...` then `count (ref index)...`.
template<class Traits> void bw_parent(const BRepGraph& g,typename Traits::ParentId p){
 int n=0;for(BRepGraph_DefsIterator::DefsOfParent<Traits> it(g,p);it.More();it.Next())++n;br_int(n);
 for(BRepGraph_DefsIterator::DefsOfParent<Traits> it(g,p);it.More();it.Next()){bw_id(it.CurrentId());br_int(it.CurrentRefId().Index);br_int(it.Index());}}
template<class Traits> void bw_parent_refs(const BRepGraph& g,typename Traits::ParentId p){
 int n=0;for(BRepGraph_RefsIterator::RefsOfParent<Traits> it(g,p);it.More();it.Next())++n;br_int(n);
 for(BRepGraph_RefsIterator::RefsOfParent<Traits> it(g,p);it.More();it.Next()){br_int(it.CurrentId().Index);br_int(it.Index());}}
void bw_children(const BRepGraph& g){
 namespace D=BRepGraph_DefsIterator;namespace R=BRepGraph_RefsIterator;const BRepGraph::TopoView& t=g.Topo();
 for(uint32_t i=0;i<=t.Solids().Nb();++i){bw_parent<D::ShellOfSolidTraits>(g,BRepGraph_SolidId(i));bw_parent_refs<R::ShellOfSolidTraits>(g,BRepGraph_SolidId(i));}
 for(uint32_t i=0;i<=t.Shells().Nb();++i){bw_parent<D::FaceOfShellTraits>(g,BRepGraph_ShellId(i));bw_parent_refs<R::FaceOfShellTraits>(g,BRepGraph_ShellId(i));}
 for(uint32_t i=0;i<=t.Faces().Nb();++i){bw_parent<D::WireOfFaceTraits>(g,BRepGraph_FaceId(i));bw_parent_refs<R::WireOfFaceTraits>(g,BRepGraph_FaceId(i));}
 for(uint32_t i=0;i<=t.Wires().Nb();++i){bw_parent<D::CoEdgeOfWireTraits>(g,BRepGraph_WireId(i));bw_parent<D::EdgeOfWireTraits>(g,BRepGraph_WireId(i));
  bw_parent_refs<R::CoEdgeOfWireTraits>(g,BRepGraph_WireId(i));}
 for(uint32_t i=0;i<=t.CompSolids().Nb();++i){bw_parent<D::SolidOfCompSolidTraits>(g,BRepGraph_CompSolidId(i));bw_parent_refs<R::SolidOfCompSolidTraits>(g,BRepGraph_CompSolidId(i));}
 for(uint32_t i=0;i<=t.Compounds().Nb();++i){bw_parent<D::ChildOfCompoundTraits>(g,BRepGraph_CompoundId(i));bw_parent_refs<R::ChildOfCompoundTraits>(g,BRepGraph_CompoundId(i));}
 for(uint32_t i=0;i<=t.Products().Nb();++i){bw_parent<D::OccurrenceOfProductTraits>(g,BRepGraph_ProductId(i));bw_parent_refs<R::OccurrenceOfProductTraits>(g,BRepGraph_ProductId(i));}
 for(uint32_t i=0;i<=t.Edges().Nb();++i){
  for(BRepGraph_DefsVertexOfEdge it(g,BRepGraph_EdgeId(i));it.More();it.Next()){br_int(it.CurrentId().Index);br_int(it.CurrentRefId().Index);br_int(it.Index());}
  for(BRepGraph_RefsVertexOfEdge it(g,BRepGraph_EdgeId(i));it.More();it.Next()){br_int(it.CurrentId().Index);br_int(it.Index());}br_word("|");}
 br_end();}
// The reverse walks: ids with their index (and references for the ref-based ones).
template<class It> void bw_ids(It it){int n=0;for(;it.More();it.Next()){br_int(it.CurrentId().Index);br_int(it.Index());++n;}br_int(n);}
template<class It> void bw_ref_ids(It it){for(;it.More();it.Next()){br_int(it.CurrentParentId().Index);br_int(it.CurrentRefId().Index);br_int(it.Index());}br_word("|");}
void bw_edge_parents(const BRepGraph& g){
 const BRepGraph::TopoView& t=g.Topo();
 for(uint32_t i=0;i<=t.Edges().Nb();++i){BRepGraph_EdgeId e(i);
  bw_ids(BRepGraph_CoEdgesOfEdge(g,t.Edges().CoEdges(e)));
  for(uint32_t start=0;start<3;++start){bw_ids(t.Edges().WiresOf(e,start));bw_ids(t.Edges().FacesOf(e,start));}}
 NCollection_LinearVector<BRepGraph_WireId> wires;for(uint32_t i=0;i<=t.Wires().Nb();++i)wires.Append(BRepGraph_WireId(i));
 for(uint32_t i=0;i<=t.CoEdges().Nb();++i)bw_ref_ids(BRepGraph_RefsWiresOfCoEdge(g,wires,BRepGraph_CoEdgeId(i)));}
void bw_reverse(const BRepGraph& g){
 const BRepGraph::TopoView& t=g.Topo();
 for(uint32_t i=0;i<t.Vertices().Nb();++i){BRepGraph_VertexId v(i);
  BRepGraph_EdgesOfVertex it(g,t.Vertices().Edges(v));br_int(it.Size());bw_ids(it);
  bw_ref_ids(BRepGraph_RefsEdgesOfVertex(g,t.Vertices().Edges(v),v));}
 bw_edge_parents(g);
 for(uint32_t i=0;i<t.Wires().Nb();++i){const auto& refs=t.Wires().Relations(BRepGraph_WireId(i)).ParentWireRefIds;
  bw_ref_ids(BRepGraph_RefsFacesOfWire(g,refs));bw_ids(BRepGraph_FacesOfWire(g,refs,1));}
 for(uint32_t i=0;i<t.Faces().Nb();++i)bw_ref_ids(BRepGraph_RefsShellsOfFace(g,t.Faces().Relations(BRepGraph_FaceId(i)).ParentFaceRefIds));
 for(uint32_t i=0;i<t.Shells().Nb();++i)bw_ref_ids(BRepGraph_RefsSolidsOfShell(g,t.Shells().Relations(BRepGraph_ShellId(i)).ParentShellRefIds));
 for(uint32_t i=0;i<t.Solids().Nb();++i)bw_ref_ids(BRepGraph_RefsCompSolidsOfSolid(g,t.Solids().Relations(BRepGraph_SolidId(i)).ParentSolidRefIds));
 for(uint32_t i=0;i<t.Occurrences().Nb();++i)bw_ref_ids(BRepGraph_RefsProductsOfOccurrence(g,t.Occurrences().Relations(BRepGraph_OccurrenceId(i)).ParentOccurrenceRefIds));
 for(const BRepGraph_NodeId& n:bw_nodes(g)){
  bw_ref_ids(BRepGraph_RefsCompoundsOfChild(g,t.Gen().CompoundRefIds(n)));bw_ids(BRepGraph_OccurrencesOfChild(g,t.Gen().OccurrenceRefIds(n)));}
 br_end();}
// The related nodes of every node, and of the first index past each kind's end.
void bw_related(const BRepGraph& g){
 std::vector<BRepGraph_NodeId> nodes=bw_nodes(g);
 for(int k=0;k<12;++k)nodes.push_back(BRepGraph_NodeId(BRepGraph_NodeId::Kind(k),g.Topo().Gen().Nb(BRepGraph_NodeId::Kind(k))));
 for(const BRepGraph_NodeId& n:nodes){
  try{for(BRepGraph_RelatedIterator it(g,n);it.More();it.Next()){bg_node(it.Current());br_int(int(it.CurrentRelation()));}}
  catch(const Standard_Failure& e){br_word(e.ExceptionType());}
  br_word("|");}
 br_end();}
