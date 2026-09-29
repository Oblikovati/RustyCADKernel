// BRepTools oracle, part 3: ReShape, History, Substitution, Quilt and PurgeLocations
// driven by commands over a scene built by tkbrep_probe_brep.cxx. Each command prints
// `-` or its exception; lists whose native order follows TShape addresses are printed
// sorted by descriptor key. The Rust twin is
// crates/tkbrep/examples/br_probe/brep_tools_reshape.rs.
#include <BRepTools_History.hxx>
#include <BRepTools_PurgeLocations.hxx>
#include <BRepTools_Quilt.hxx>
#include <BRepTools_ReShape.hxx>
#include <BRepTools_Substitution.hxx>
// The descriptor of a shape as numbers: TShape id, orientation, chain (-9 for null).
std::vector<long long> bt_key(const BrGraph& g,const TopoDS_Shape& s){
 if(s.IsNull())return {-9};
 std::vector<long long> k{g.tshape_id(s),s.Orientation()};
 for(TopLoc_Location c=s.Location();!c.IsIdentity();c=c.NextLocation()){k.push_back(g.datum_id(c.FirstDatum()));k.push_back(c.FirstPower());}
 return k;}
// A list of shapes as sorted keys: the count, then each key with its length.
void bt_sorted(const BrGraph& g,const NCollection_List<TopoDS_Shape>& l){
 std::vector<std::vector<long long>> keys;for(const TopoDS_Shape& s:l)keys.push_back(bt_key(g,s));
 std::sort(keys.begin(),keys.end());br_int(keys.size());
 for(const auto& k:keys){br_int(k.size());for(long long v:k)br_int(v);}}
NCollection_List<TopoDS_Shape> bt_ids(const BrGraph& g,BrReader& r){
 NCollection_List<TopoDS_Shape> l;int n=r.integer();for(int i=0;i<n;++i)l.Append(g.shape(r.integer()));return l;}
// ReShape commands: 0 replace a b, 1 remove a, 2 merge n ids... product, 3 mode flag,
// 4 copy vertex v tol (pushed), 5 copy vertex v P tol (pushed), 6 clear.
void bt_reshape_command(BrGraph& g,BRepTools_ReShape& rs,BrReader& r){
 int code=r.integer();
 try{
  switch(code){
   case 0:{TopoDS_Shape a=g.shape(r.integer());rs.Replace(a,g.shape(r.integer()));break;}
   case 1:rs.Remove(g.shape(r.integer()));break;
   case 2:{NCollection_List<TopoDS_Shape> l=bt_ids(g,r);rs.Merge(l,g.shape(r.integer()));break;}
   case 3:rs.ModeConsiderLocation()=r.flag();break;
   case 4:{TopoDS_Shape v=g.shape(r.integer());double tol=r.real();g.shapes.push_back(rs.CopyVertex(TopoDS::Vertex(v),tol));break;}
   case 5:{TopoDS_Shape v=g.shape(r.integer());gp_Pnt p=br_point(r);double tol=r.real();g.shapes.push_back(rs.CopyVertex(TopoDS::Vertex(v),p,tol));break;}
   default:rs.Clear();break;}
  br_word("-");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
void bt_reshape_report(const BrGraph& g,BRepTools_ReShape& rs,const TopoDS_Shape& sh){
 br_flag(rs.IsRecorded(sh));br_flag(rs.IsNewShape(sh));BR_TRY(br_desc(g,rs.Value(sh)));BR_TRY(br_desc(g,rs.ValueLeaf(sh)));
 for(bool last:{false,true}){TopoDS_Shape n;BR_TRY(br_int(rs.Status(sh,n,last));br_desc(g,n));}
 BR_TRY(br_text(bt_written(rs.Apply(sh),true,true,3)));BR_TRY(br_text(bt_written(rs.Apply(sh,TopAbs_FACE),true,true,3)));}
void bt_history_report(const BrGraph& g,const occ::handle<BRepTools_History>& h){
 br_flag(h->HasGenerated());br_flag(h->HasModified());br_flag(h->HasRemoved());
 for(const TopoDS_Shape& sh:g.shapes){
  if(sh.IsNull())continue;br_flag(BRepTools_History::IsSupportedType(sh));
  BR_TRY(bt_sorted(g,h->Generated(sh)));BR_TRY(bt_sorted(g,h->Modified(sh)));BR_TRY(br_flag(h->IsRemoved(sh)));}
 std::ostringstream o;h->Dump(o);br_text(o.str());}
// `br_tools_reshape script ncmd (command)...`: the commands, the report of every scene
// shape, then the history of the replacements.
void br_tools_reshape_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);BRepTools_ReShape rs;
 int n=r.integer();for(int i=0;i<n;++i)bt_reshape_command(s.graph,rs,r);
 for(const TopoDS_Shape& sh:s.graph.shapes)if(!sh.IsNull())bt_reshape_report(s.graph,rs,sh);
 BR_TRY(bt_history_report(s.graph,rs.History()));
 br_end();}
// History commands on history h (codes below 10) or on a second history (code + 10):
// 0 generated a b, 1 modified a b, 2 remove a, 3 replace generated a b, 4 replace
// modified a b, 5 clear; 20 merge the second, 21 merge its handle.
void bt_history_command(const BrGraph& g,const occ::handle<BRepTools_History> hs[2],BrReader& r){
 int code=r.integer();
 try{
  if(code>=20){if(code==20)hs[0]->Merge(*hs[1]);else hs[0]->Merge(hs[1]);br_word("-");return;}
  BRepTools_History& h=*hs[code>=10?1:0];code%=10;
  TopoDS_Shape a=g.shape(r.integer());
  switch(code){
   case 0:h.AddGenerated(a,g.shape(r.integer()));break;
   case 1:h.AddModified(a,g.shape(r.integer()));break;
   case 2:h.Remove(a);break;
   case 3:h.ReplaceGenerated(a,g.shape(r.integer()));break;
   case 4:h.ReplaceModified(a,g.shape(r.integer()));break;
   default:h.Clear();break;}
  br_word("-");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
// `br_tools_history script ncmd (command)...`.
void br_tools_history_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);occ::handle<BRepTools_History> hs[2]={new BRepTools_History(),new BRepTools_History()};
 int n=r.integer();for(int i=0;i<n;++i)bt_history_command(s.graph,hs,r);
 bt_history_report(s.graph,hs[0]);bt_history_report(s.graph,hs[1]);
 br_end();}
// `br_tools_subst script ncmd (command)...`: 0 substitute a n ids..., 1 build a, 2 clear;
// then each scene shape's copies as text.
void br_tools_subst_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);const BrGraph& g=s.graph;BRepTools_Substitution sub;
 int n=r.integer();
 for(int i=0;i<n;++i){int code=r.integer();
  try{if(code==0){TopoDS_Shape a=g.shape(r.integer());sub.Substitute(a,bt_ids(g,r));}
   else if(code==1)sub.Build(g.shape(r.integer()));else sub.Clear();br_word("-");
  }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
 for(const TopoDS_Shape& sh:g.shapes){
  if(sh.IsNull())continue;br_flag(sub.IsCopied(sh));
  BR_TRY(const NCollection_List<TopoDS_Shape>& l=sub.Copy(sh);br_int(l.Extent());for(const TopoDS_Shape& c:l)br_text(bt_written(c,true,true,3)));}
 br_end();}
// The Shells compound: the shells in order (faces, closed, orientable), then the other
// children counted by type.
void bt_shells(const TopoDS_Shape& c){
 int others[9]={0};br_int(c.NbChildren());
 for(TopoDS_Iterator it(c);it.More();it.Next()){const TopoDS_Shape& x=it.Value();
  if(x.ShapeType()!=TopAbs_SHELL){others[x.ShapeType()]++;continue;}
  br_int(x.NbChildren());br_flag(x.Closed());br_flag(x.Orientable());br_int(x.Orientation());}
 for(int k:others)br_int(k);}
// `br_tools_quilt script ncmd (command)...`: 0 bind edges a b, 1 bind vertices a b, 2 add a;
// then each scene shape's copy and the shells.
void br_tools_quilt_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);const BrGraph& g=s.graph;BRepTools_Quilt q;
 int n=r.integer();
 for(int i=0;i<n;++i){int code=r.integer();
  try{TopoDS_Shape a=g.shape(r.integer());
   if(code==0)q.Bind(TopoDS::Edge(a),TopoDS::Edge(g.shape(r.integer())));
   else if(code==1)q.Bind(TopoDS::Vertex(a),TopoDS::Vertex(g.shape(r.integer())));else q.Add(a);br_word("-");
  }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
 for(const TopoDS_Shape& sh:g.shapes){
  if(sh.IsNull())continue;br_flag(q.IsCopied(sh));BR_TRY(br_text(bt_written(q.Copy(sh),true,true,3)));}
 BR_TRY(bt_shells(q.Shells()));
 br_end();}
// `br_tools_purge script`: PurgeLocations on every scene shape.
void br_tools_purge_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);const BrGraph& g=s.graph;
 for(const TopoDS_Shape& sh:g.shapes){
  if(sh.IsNull())continue;BRepTools_PurgeLocations p;
  BR_TRY(br_flag(p.Perform(sh));br_flag(p.IsDone());br_text(bt_written(p.GetResult(),true,true,3));
   for(const TopoDS_Shape& x:g.shapes)if(!x.IsNull())br_flag(p.ModifiedShape(x).IsSame(x)));}
 br_end();}
