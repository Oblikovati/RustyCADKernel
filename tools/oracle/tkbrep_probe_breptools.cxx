// BRepTools oracle, part 1: the text format, the queries and the in-place tools over a
// scene built by tkbrep_probe_brep.cxx. Written text is printed as `t<hex>` tokens and
// compared byte for byte; shapes found in the scene are printed as descriptors. Edge
// and face pairs are taken from the faces' own edges, which carry p-curves, so no query
// reaches the planar projection. The Rust twin is crates/tkbrep/examples/br_probe/brep_tools.rs.
#include <BRepTools.hxx>
#include <BRepTools_ShapeSet.hxx>
#include <Bnd_Box2d.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
// BRepTools::Write into a string (the options of the long overload).
std::string bt_written(const TopoDS_Shape& sh,bool tri,bool normals,int version){
 std::ostringstream o;BRepTools::Write(sh,o,tri,normals,TopTools_FormatVersion(version));return o.str();}
// The shape read back from a text: the console output, then `N` or its type and text.
void bt_reread(const std::string& w){
 TopoDS_Shape r;
 std::string console=br_captured([&]{std::istringstream i(w);BRepTools::Read(r,i,BRep_Builder());});
 br_text(console);if(r.IsNull()){br_word("N");return;}
 br_int(r.ShapeType());br_text(bt_written(r,true,true,3));}
void bt_text_shape(const TopoDS_Shape& sh){
 static const int options[][3]={{1,1,0},{2,1,0},{3,1,1},{3,0,0},{2,0,1}};
 for(const auto& o:options){std::string w;BR_TRY(w=bt_written(sh,o[1]!=0,o[2]!=0,o[0]);br_text(w);bt_reread(w));}
 std::ostringstream d;BR_TRY(BRepTools::Dump(sh,d);br_text(br_masked(d.str())));}
// One shape set holding every scene shape: the set text, each shape reference, the
// dump, then the shapes read back through one reading set.
void bt_whole_set(const BrScene& s,int version){
 BRepTools_ShapeSet set(true,true);set.SetFormatNb(version);
 for(const TopoDS_Shape& sh:s.graph.shapes)BR_TRY(set.Add(sh));
 std::ostringstream o;BR_TRY(set.Write(o);for(const TopoDS_Shape& sh:s.graph.shapes)set.Write(sh,o);br_text(o.str()));
 std::ostringstream d;BR_TRY(set.Dump(d);br_text(br_masked(d.str())));
 BRepTools_ShapeSet in;std::istringstream i(o.str());std::vector<int> nulls;int nb=-1;std::string error;
 std::string console=br_captured([&]{try{in.Read(i);nb=in.NbShapes();
  for(size_t k=0;k<s.graph.shapes.size();++k){TopoDS_Shape r;in.Read(r,i);nulls.push_back(r.IsNull());}}
  catch(const Standard_Failure& e){error=e.ExceptionType();}});
 br_text(console);br_int(nb);for(int n:nulls)br_flag(n!=0);if(!error.empty())br_word(error.c_str());}
// `br_tools_text script`: every shape through each Write option and back, then the sets.
void br_tools_text_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);
 for(const TopoDS_Shape& sh:s.graph.shapes)bt_text_shape(sh);
 for(int v=1;v<=3;++v)bt_whole_set(s,v);
 br_end();}
void bt_uv(double a,double b,double c,double d){br_real(a);br_real(b);br_real(c);br_real(d);}
void bt_box(const Bnd_Box2d& b){br_flag(b.IsVoid());double a,c,d,e;BR_TRY(b.Get(a,c,d,e);bt_uv(a,c,d,e));}
// The queries of one face: its bounds, outer wire and closedness, then each of its own
// wires and edges.
void bt_face_queries(const BrGraph& g,const TopoDS_Face& f){
 double a,b,c,d;BR_TRY(BRepTools::UVBounds(f,a,b,c,d);bt_uv(a,b,c,d));
 {Bnd_Box2d x;BR_TRY(BRepTools::AddUVBounds(f,x);bt_box(x));}
 BR_TRY(br_desc(g,BRepTools::OuterWire(f)));
 {bool u=false,v=false;BR_TRY(BRepTools::DetectClosedness(f,u,v);br_flag(u);br_flag(v));}
 for(TopExp_Explorer w(f,TopAbs_WIRE);w.More();w.Next()){const TopoDS_Wire& wire=TopoDS::Wire(w.Current());
  BR_TRY(BRepTools::UVBounds(f,wire,a,b,c,d);bt_uv(a,b,c,d));Bnd_Box2d x;BR_TRY(BRepTools::AddUVBounds(f,wire,x);bt_box(x));}
 for(TopExp_Explorer e(f,TopAbs_EDGE);e.More();e.Next()){const TopoDS_Edge& edge=TopoDS::Edge(e.Current());
  BR_TRY(BRepTools::UVBounds(f,edge,a,b,c,d);bt_uv(a,b,c,d));Bnd_Box2d x;BR_TRY(BRepTools::AddUVBounds(f,edge,x);bt_box(x));
  BR_TRY(br_flag(BRepTools::IsReallyClosed(edge,f)));BR_TRY(br_int(BRepTools::OriEdgeInFace(edge,f)));}}
// The queries of one shape: its 3D edges, bad locations and mesh state.
void bt_shape_queries(const BrGraph& g,const TopoDS_Shape& sh){
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> edges;BR_TRY(BRepTools::Map3DEdges(sh,edges);br_int(edges.Extent());
  for(int i=1;i<=edges.Extent();++i)br_desc(g,edges(i)));
 NCollection_List<TopoDS_Shape> bad;BR_TRY(BRepTools::CheckLocations(sh,bad);br_int(bad.Extent());for(const TopoDS_Shape& b:bad)br_desc(g,b));
 BR_TRY(br_flag(BRepTools::Triangulation(sh,0.1,false)));BR_TRY(br_flag(BRepTools::Triangulation(sh,1e-3,true)));}
// `br_tools_query script`: face, shape, vertex pair and edge pair queries.
void br_tools_query_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);const BrGraph& g=s.graph;
 for(const TopoDS_Shape& sh:g.shapes){
  if(sh.IsNull())continue;
  if(sh.ShapeType()==TopAbs_FACE)bt_face_queries(g,TopoDS::Face(sh));
  bt_shape_queries(g,sh);}
 for(const TopoDS_Shape& a:g.shapes)for(const TopoDS_Shape& b:g.shapes){
  if(a.IsNull()||b.IsNull()||a.ShapeType()!=b.ShapeType())continue;
  if(a.ShapeType()==TopAbs_VERTEX)BR_TRY(br_flag(BRepTools::Compare(TopoDS::Vertex(a),TopoDS::Vertex(b))));
  if(a.ShapeType()==TopAbs_EDGE)br_flag(BRepTools::Compare(TopoDS::Edge(a),TopoDS::Edge(b)));}
 br_end();}
// The tolerance EvalAndUpdateTol measures for each edge of a face with its curves.
void bt_eval_tolerances(const TopoDS_Shape& sh){
 for(TopExp_Explorer f(sh,TopAbs_FACE);f.More();f.Next()){const TopoDS_Face& face=TopoDS::Face(f.Current());
  for(TopExp_Explorer e(face,TopAbs_EDGE);e.More();e.Next()){const TopoDS_Edge& edge=TopoDS::Edge(e.Current());
   double first,last,f2,l2;occ::handle<Geom_Curve> c3=BRep_Tool::Curve(edge,first,last);
   occ::handle<Geom2d_Curve> c2=BRep_Tool::CurveOnSurface(edge,face,f2,l2);occ::handle<Geom_Surface> sf=BRep_Tool::Surface(face);
   if(c3.IsNull()||c2.IsNull())continue;BR_TRY(br_real(BRepTools::EvalAndUpdateTol(edge,c3,c2,sf,first,last)));}}}
// The triangulation list tools; each prints its flag.
void bt_triangulations(const TopoDS_Shape& sh){
 BR_TRY(br_flag(BRepTools::LoadAllTriangulations(sh)));BR_TRY(br_flag(BRepTools::UnloadAllTriangulations(sh)));
 BR_TRY(br_flag(BRepTools::LoadTriangulation(sh,-1,false)));BR_TRY(br_flag(BRepTools::UnloadTriangulation(sh,0)));
 BR_TRY(br_flag(BRepTools::ActivateTriangulation(sh,1,false)));BR_TRY(br_flag(BRepTools::ActivateTriangulation(sh,1,true)));
 BR_TRY(br_flag(BRepTools::ActivateTriangulation(sh,0,true)));BR_TRY(br_flag(BRepTools::LoadTriangulation(sh,-2,false)));}
// One in-place tool on a scene shape (codes of `br_tools_mutate`).
void bt_mutate(TopoDS_Shape& sh,int code){
 switch(code){
  case 0:BRepTools::Update(sh);break;
  case 1:BRepTools::Clean(sh,false);break;
  case 2:BRepTools::Clean(sh,true);break;
  case 3:BRepTools::CleanGeometry(sh);break;
  case 4:BRepTools::RemoveUnusedPCurves(sh);break;
  case 5:BRepTools::RemoveInternals(sh,false);break;
  case 6:BRepTools::RemoveInternals(sh,true);break;
  case 7:if(sh.ShapeType()==TopAbs_FACE)BRepTools::UpdateFaceUVPoints(TopoDS::Face(sh));break;
  case 8:bt_eval_tolerances(sh);break;
  default:bt_triangulations(sh);break;}}
// `br_tools_mutate script code`: the tool on every non-null scene shape in order, then
// every shape written in the current format.
void br_tools_mutate_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);int code=r.integer();
 for(TopoDS_Shape& sh:s.graph.shapes){if(sh.IsNull())continue;BR_TRY(bt_mutate(sh,code);br_word("-"));}
 for(const TopoDS_Shape& sh:s.graph.shapes)BR_TRY(br_text(bt_written(sh,true,true,3)));
 br_end();}
