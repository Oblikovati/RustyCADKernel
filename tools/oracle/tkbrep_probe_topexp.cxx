// TopExp oracle: explorer sequences for every searched and avoided type, the indexed
// and plain maps, the ancestor maps and the vertex queries of a scripted graph. The
// Rust twin is crates/tkbrep/examples/br_probe/topexp.rs.
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
// One exploration: each found shape with the depth, then the state at the end, the
// restart that Next performs after the end, ReInit, and Clear.
void br_explore(const BrGraph& g,const TopoDS_Shape& root,int find,int avoid){
 TopExp_Explorer e(root,TopAbs_ShapeEnum(find),TopAbs_ShapeEnum(avoid));int n=0;
 for(;e.More()&&n<1000;e.Next(),++n){br_desc(g,e.Current());br_int(e.Depth());}
 br_word("|");br_desc(g,e.Current());br_int(e.Depth());
 e.Next();br_flag(e.More());br_desc(g,e.Value());br_int(e.Depth());
 e.ReInit();br_flag(e.More());br_desc(g,e.Current());
 br_desc(g,e.ExploredShape());e.Clear();br_flag(e.More());br_int(e.Depth());}
// `br_explore script root`: every (ToFind, ToAvoid) pair, SHAPE included.
void br_explore_case(std::istream& in){
 BrReader r{in};BrGraph g=br_graph(r);TopoDS_Shape root=g.shape(r.integer());
 for(int find=0;find<=8;++find)for(int avoid=0;avoid<=8;++avoid)br_explore(g,root,find,avoid);
 br_end();}
void br_indexed(const BrGraph& g,const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& m){
 br_int(m.Extent());for(int i=1;i<=m.Extent();++i)br_desc(g,m(i));}
void br_ancestors(const BrGraph& g,const NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher>& m){
 br_int(m.Extent());
 for(int i=1;i<=m.Extent();++i){br_desc(g,m.FindKey(i));br_int(m(i).Extent());for(const TopoDS_Shape& a:m(i))br_desc(g,a);}}
// `br_maps script root`: MapShapes by type (and accumulated), MapShapes with the
// cumulation flags into indexed and plain maps, and the three ancestor maps for every
// pair of shape types.
void br_maps_case(std::istream& in){
 BrReader r{in};BrGraph g=br_graph(r);TopoDS_Shape root=g.shape(r.integer());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> all;
 for(int t=0;t<=8;++t){
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> m;
  TopExp::MapShapes(root,TopAbs_ShapeEnum(t),m);br_indexed(g,m);TopExp::MapShapes(root,TopAbs_ShapeEnum(t),all);}
 br_indexed(g,all);
 for(int c=0;c<4;++c){
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> m;TopExp::MapShapes(root,m,c&1,c&2);br_indexed(g,m);
  NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> plain;TopExp::MapShapes(root,plain,c&1,c&2);
  br_int(plain.Extent());for(const TopoDS_Shape& s:g.shapes)br_flag(plain.Contains(s));}
 for(int ts=0;ts<=7;++ts)for(int ta=0;ta<=7;++ta){
  NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> m,u,uo;
  TopExp::MapShapesAndAncestors(root,TopAbs_ShapeEnum(ts),TopAbs_ShapeEnum(ta),m);br_ancestors(g,m);
  TopExp::MapShapesAndUniqueAncestors(root,TopAbs_ShapeEnum(ts),TopAbs_ShapeEnum(ta),u);br_ancestors(g,u);
  TopExp::MapShapesAndUniqueAncestors(root,TopAbs_ShapeEnum(ts),TopAbs_ShapeEnum(ta),uo,true);br_ancestors(g,uo);}
 br_end();}
// `br_vertices script`: the vertex queries on every edge, pair of edges and wire. The
// casts are unchecked in TopExp, so any shape of the graph may be queried; only
// non-null shapes are, since the native wire loop needs a TShape.
void br_vertices_case(std::istream& in){
 BrReader r{in};BrGraph g=br_graph(r);
 for(const TopoDS_Shape& s:g.shapes){
  if(s.IsNull())continue;
  const TopoDS_Edge& e=*(const TopoDS_Edge*)&s;
  for(int cum=0;cum<2;++cum){
   br_desc(g,TopExp::FirstVertex(e,cum));br_desc(g,TopExp::LastVertex(e,cum));
   TopoDS_Vertex f,l;TopExp::Vertices(e,f,l,cum);br_desc(g,f);br_desc(g,l);}
  for(const TopoDS_Shape& t:g.shapes){
   if(t.IsNull())continue;TopoDS_Vertex v;
   bool found=TopExp::CommonVertex(e,*(const TopoDS_Edge*)&t,v);br_flag(found);br_desc(g,v);}
  if(s.ShapeType()==TopAbs_WIRE){TopoDS_Vertex f,l;TopExp::Vertices(*(const TopoDS_Wire*)&s,f,l);br_desc(g,f);br_desc(g,l);}}
 br_end();}
