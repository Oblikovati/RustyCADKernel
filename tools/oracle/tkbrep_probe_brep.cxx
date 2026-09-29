// BRep oracle, part 1: the geometry registry and the BRep_Builder script. A request is
// `nloc (location)... ngeom (geometry)... ncmd (command)...`; the locations are those
// of tkbrep_probe_graph.cxx, the geometries fill one registry per kind, and the
// commands build and update BRep shapes (codes below 100) or run a graph command
// (code 100 + graph code). Each command prints `-` or the exception it raised. The
// Rust twins are crates/tkbrep/examples/br_probe/brep.rs and brep_commands.rs.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Line.hxx>
#include <Poly_Polygon2D.hxx>
#include <Poly_Polygon3D.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Poly_Triangulation.hxx>
// The geometry registries; a handle is printed as its index, -1 when null and -2
// when it is not in the registry (a transformed copy).
struct BrGeometry{
 std::vector<occ::handle<Geom_Curve>> curves;std::vector<occ::handle<Geom_Surface>> surfaces;
 std::vector<occ::handle<Geom2d_Curve>> pcurves;std::vector<occ::handle<Poly_Polygon3D>> polygons3d;
 std::vector<occ::handle<Poly_Polygon2D>> polygons2d;std::vector<occ::handle<Poly_Triangulation>> triangulations;
 std::vector<occ::handle<Poly_PolygonOnTriangulation>> nodes;
};
template<class T> const occ::handle<T>& br_at(const std::vector<occ::handle<T>>& v,int i){
 static const occ::handle<T> none;return i>=0&&i<int(v.size())?v[i]:none;}
template<class T,class U> int br_id(const std::vector<occ::handle<T>>& v,const occ::handle<U>& h){
 if(h.IsNull())return -1;for(size_t i=0;i<v.size();++i)if(v[i].get()==h.get())return int(i);return -2;}
gp_Pnt br_point(BrReader& r){double x=r.real(),y=r.real();return gp_Pnt(x,y,r.real());}
gp_Dir br_direction(BrReader& r){double x=r.real(),y=r.real();return gp_Dir(x,y,r.real());}
gp_Pnt2d br_point2d(BrReader& r){double x=r.real();return gp_Pnt2d(x,r.real());}
// Geometry kinds: 0 line P D, 1 circle P N R, 2 Bezier n P..., 3 trimmed curve i u1 u2,
// 10 plane P N, 11 cylinder P N R, 12 rectangular trimmed surface i u1 u2 v1 v2,
// 13 offset surface i d, 14 Bezier surface nu nv P... (U major), 20 2D line P D, 21 2D circle C R, 30 3D polygon n P...,
// 31 2D polygon n P..., 32 triangulation n P... m (i j k)... purpose, 33 node array n i....
void br_curve_kind(BrGeometry& g,BrReader& r,int kind){
 occ::handle<Geom_Curve> c;
 try{
  if(kind==0){gp_Pnt p=br_point(r);c=new Geom_Line(p,br_direction(r));}
  else if(kind==1){gp_Pnt p=br_point(r);gp_Dir n=br_direction(r);c=new Geom_Circle(gp_Ax2(p,n),r.real());}
  else if(kind==2){int n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);for(int i=1;i<=n;++i)poles(i)=br_point(r);c=new Geom_BezierCurve(poles);}
  else{int i=r.integer();double u1=r.real();c=new Geom_TrimmedCurve(br_at(g.curves,i),u1,r.real());}
  br_word("-");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 g.curves.push_back(c);}
void br_surface_kind(BrGeometry& g,BrReader& r,int kind){
 occ::handle<Geom_Surface> s;
 try{
  if(kind==10){gp_Pnt p=br_point(r);s=new Geom_Plane(gp_Ax3(p,br_direction(r)));}
  else if(kind==11){gp_Pnt p=br_point(r);gp_Dir n=br_direction(r);s=new Geom_CylindricalSurface(gp_Ax3(p,n),r.real());}
  else if(kind==12){int i=r.integer();double u1=r.real(),u2=r.real(),v1=r.real();s=new Geom_RectangularTrimmedSurface(br_at(g.surfaces,i),u1,u2,v1,r.real());}
  else if(kind==13){int i=r.integer();s=new Geom_OffsetSurface(br_at(g.surfaces,i),r.real());}
  else{int nu=r.integer(),nv=r.integer();NCollection_Array2<gp_Pnt> poles(1,nu,1,nv);
   for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)poles(i,j)=br_point(r);s=new Geom_BezierSurface(poles);}
  br_word("-");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 g.surfaces.push_back(s);}
void br_pcurve_kind(BrGeometry& g,BrReader& r,int kind){
 occ::handle<Geom2d_Curve> c;
 try{
  gp_Pnt2d p=br_point2d(r);
  if(kind==20){double x=r.real();c=new Geom2d_Line(p,gp_Dir2d(x,r.real()));}
  else c=new Geom2d_Circle(gp_Ax2d(p,gp_Dir2d(1.0,0.0)),r.real());
  br_word("-");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 g.pcurves.push_back(c);}
occ::handle<Poly_Triangulation> br_read_triangulation(BrReader& r){
 int n=r.integer();NCollection_Array1<gp_Pnt> nodes(1,n);for(int i=1;i<=n;++i)nodes(i)=br_point(r);
 int m=r.integer();NCollection_Array1<Poly_Triangle> tris(1,m);
 for(int i=1;i<=m;++i){int a=r.integer(),b=r.integer();tris(i)=Poly_Triangle(a,b,r.integer());}
 occ::handle<Poly_Triangulation> t=new Poly_Triangulation(nodes,tris);t->SetMeshPurpose(Poly_MeshPurpose(r.integer()));return t;}
void br_mesh_kind(BrGeometry& g,BrReader& r,int kind){
 if(kind==30){int n=r.integer();NCollection_Array1<gp_Pnt> p(1,n);for(int i=1;i<=n;++i)p(i)=br_point(r);g.polygons3d.push_back(new Poly_Polygon3D(p));}
 else if(kind==31){int n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);for(int i=1;i<=n;++i)p(i)=br_point2d(r);g.polygons2d.push_back(new Poly_Polygon2D(p));}
 else if(kind==32)g.triangulations.push_back(br_read_triangulation(r));
 else{int n=r.integer();NCollection_Array1<int> p(1,n);for(int i=1;i<=n;++i)p(i)=r.integer();g.nodes.push_back(new Poly_PolygonOnTriangulation(p));}
 br_word("-");}
void br_read_geometry(BrGeometry& g,BrReader& r){
 int kind=r.integer();
 if(kind<10)br_curve_kind(g,r,kind);else if(kind<20)br_surface_kind(g,r,kind);
 else if(kind<30)br_pcurve_kind(g,r,kind);else br_mesh_kind(g,r,kind);}
// The shape arguments of the commands, cast with the checked TopoDS casts.
const TopoDS_Vertex& br_v(const BrGraph& g,BrReader& r){return TopoDS::Vertex(g.shapes.at(r.integer()));}
const TopoDS_Edge& br_e(const BrGraph& g,BrReader& r){return TopoDS::Edge(g.shapes.at(r.integer()));}
const TopoDS_Face& br_f(const BrGraph& g,BrReader& r){return TopoDS::Face(g.shapes.at(r.integer()));}
struct BrScene{BrGraph graph;BrGeometry geometry;};
// Commands that make a new shape: 1 vertex P tol, 2 vertex, 3 edge, 4 edge c loc tol,
// 5 face s loc tol, 6 face, 7 face t, 8 edge polygon3d p, 9 edge nodes n t loc,
// 47 face with triangulations (count ids... active).
bool br_make_command(BrScene& s,BrReader& r,int code){
 BRep_Builder B;BrGraph& g=s.graph;BrGeometry& G=s.geometry;
 switch(code){
  case 1:{TopoDS_Vertex v;gp_Pnt p=br_point(r);B.MakeVertex(v,p,r.real());g.shapes.push_back(v);return true;}
  case 2:{TopoDS_Vertex v;B.MakeVertex(v);g.shapes.push_back(v);return true;}
  case 3:{TopoDS_Edge e;B.MakeEdge(e);g.shapes.push_back(e);return true;}
  case 4:{TopoDS_Edge e;int c=r.integer(),l=r.integer();B.MakeEdge(e,br_at(G.curves,c),g.loc(l),r.real());g.shapes.push_back(e);return true;}
  case 5:{TopoDS_Face f;int c=r.integer(),l=r.integer();B.MakeFace(f,br_at(G.surfaces,c),g.loc(l),r.real());g.shapes.push_back(f);return true;}
  case 6:{TopoDS_Face f;B.MakeFace(f);g.shapes.push_back(f);return true;}
  case 7:{TopoDS_Face f;B.MakeFace(f,br_at(G.triangulations,r.integer()));g.shapes.push_back(f);return true;}
  case 8:{TopoDS_Edge e;B.MakeEdge(e,br_at(G.polygons3d,r.integer()));g.shapes.push_back(e);return true;}
  case 9:{TopoDS_Edge e;int n=r.integer(),t=r.integer();B.MakeEdge(e,br_at(G.nodes,n),br_at(G.triangulations,t),g.loc(r.integer()));g.shapes.push_back(e);return true;}
  case 47:{TopoDS_Face f;NCollection_List<occ::handle<Poly_Triangulation>> list;int n=r.integer();
   for(int i=0;i<n;++i)list.Append(br_at(G.triangulations,r.integer()));
   B.MakeFace(f,list,br_at(G.triangulations,r.integer()));g.shapes.push_back(f);return true;}
 }
 return false;}
// Edge commands: 10 curve e c loc tol, 11 pcurve e c s loc tol uv(0|1 u1 v1 u2 v2),
// 12 pcurves e c1 c2 s loc tol uv, 13 pcurve on face e c f tol, 14 pcurves on face
// e c1 c2 f tol, 15 polygon3d e p loc, 16 nodes e n t loc, 17 two nodes e n1 n2 t loc,
// 18 polygon2d e p s loc, 19 two polygon2d e p1 p2 s loc, 20 polygon2d on face e p f,
// 21 two polygon2d on face e p1 p2 f, 22 tolerance e tol.
bool br_edge_command(BrScene& s,BrReader& r,int code){
 BRep_Builder B;BrGraph& g=s.graph;BrGeometry& G=s.geometry;
 switch(code){
  case 10:{const TopoDS_Edge& e=br_e(g,r);int c=r.integer(),l=r.integer();B.UpdateEdge(e,br_at(G.curves,c),g.loc(l),r.real());return true;}
  case 11:case 12:{const TopoDS_Edge& e=br_e(g,r);int c1=r.integer(),c2=code==12?r.integer():-1,sf=r.integer(),l=r.integer();double tol=r.real();
   bool uv=r.flag();gp_Pnt2d p1,p2;if(uv){p1=br_point2d(r);p2=br_point2d(r);}
   if(code==11){if(uv)B.UpdateEdge(e,br_at(G.pcurves,c1),br_at(G.surfaces,sf),g.loc(l),tol,p1,p2);else B.UpdateEdge(e,br_at(G.pcurves,c1),br_at(G.surfaces,sf),g.loc(l),tol);}
   else if(uv)B.UpdateEdge(e,br_at(G.pcurves,c1),br_at(G.pcurves,c2),br_at(G.surfaces,sf),g.loc(l),tol,p1,p2);
   else B.UpdateEdge(e,br_at(G.pcurves,c1),br_at(G.pcurves,c2),br_at(G.surfaces,sf),g.loc(l),tol);return true;}
  case 13:{const TopoDS_Edge& e=br_e(g,r);int c=r.integer();const TopoDS_Face& f=br_f(g,r);B.UpdateEdge(e,br_at(G.pcurves,c),f,r.real());return true;}
  case 14:{const TopoDS_Edge& e=br_e(g,r);int c1=r.integer(),c2=r.integer();const TopoDS_Face& f=br_f(g,r);B.UpdateEdge(e,br_at(G.pcurves,c1),br_at(G.pcurves,c2),f,r.real());return true;}
  case 15:{const TopoDS_Edge& e=br_e(g,r);int p=r.integer();B.UpdateEdge(e,br_at(G.polygons3d,p),g.loc(r.integer()));return true;}
  case 16:{const TopoDS_Edge& e=br_e(g,r);int n=r.integer(),t=r.integer();B.UpdateEdge(e,br_at(G.nodes,n),br_at(G.triangulations,t),g.loc(r.integer()));return true;}
  case 17:{const TopoDS_Edge& e=br_e(g,r);int n1=r.integer(),n2=r.integer(),t=r.integer();B.UpdateEdge(e,br_at(G.nodes,n1),br_at(G.nodes,n2),br_at(G.triangulations,t),g.loc(r.integer()));return true;}
  case 18:{const TopoDS_Edge& e=br_e(g,r);int p=r.integer(),sf=r.integer();B.UpdateEdge(e,br_at(G.polygons2d,p),br_at(G.surfaces,sf),g.loc(r.integer()));return true;}
  case 19:{const TopoDS_Edge& e=br_e(g,r);int p1=r.integer(),p2=r.integer(),sf=r.integer();B.UpdateEdge(e,br_at(G.polygons2d,p1),br_at(G.polygons2d,p2),br_at(G.surfaces,sf),g.loc(r.integer()));return true;}
  case 20:{const TopoDS_Edge& e=br_e(g,r);int p=r.integer();B.UpdateEdge(e,br_at(G.polygons2d,p),br_f(g,r));return true;}
  case 21:{const TopoDS_Edge& e=br_e(g,r);int p1=r.integer(),p2=r.integer();B.UpdateEdge(e,br_at(G.polygons2d,p1),br_at(G.polygons2d,p2),br_f(g,r));return true;}
  case 22:{const TopoDS_Edge& e=br_e(g,r);B.UpdateEdge(e,r.real());return true;}
 }
 return false;}
// Edge flags and ranges: 23 continuity e s1 s2 l1 l2 c, 24 continuity on faces e f1 f2 c,
// 25 same parameter e b, 26 same range e b, 27 degenerated e b, 28 range e f l only3d,
// 29 range on surface e s loc f l, 30 range on face e face f l, 31 transfert e1 e2,
// 32 transfert vertex e1 e2 v1 v2.
bool br_flag_command(BrScene& s,BrReader& r,int code){
 BRep_Builder B;BrGraph& g=s.graph;BrGeometry& G=s.geometry;
 switch(code){
  case 23:{const TopoDS_Edge& e=br_e(g,r);int s1=r.integer(),s2=r.integer(),l1=r.integer(),l2=r.integer();
   B.Continuity(e,br_at(G.surfaces,s1),br_at(G.surfaces,s2),g.loc(l1),g.loc(l2),GeomAbs_Shape(r.integer()));return true;}
  case 24:{const TopoDS_Edge& e=br_e(g,r);const TopoDS_Face& f1=br_f(g,r);const TopoDS_Face& f2=br_f(g,r);B.Continuity(e,f1,f2,GeomAbs_Shape(r.integer()));return true;}
  case 25:{const TopoDS_Edge& e=br_e(g,r);B.SameParameter(e,r.flag());return true;}
  case 26:{const TopoDS_Edge& e=br_e(g,r);B.SameRange(e,r.flag());return true;}
  case 27:{const TopoDS_Edge& e=br_e(g,r);B.Degenerated(e,r.flag());return true;}
  case 28:{const TopoDS_Edge& e=br_e(g,r);double f=r.real(),l=r.real();B.Range(e,f,l,r.flag());return true;}
  case 29:{const TopoDS_Edge& e=br_e(g,r);int sf=r.integer(),lo=r.integer();double f=r.real();B.Range(e,br_at(G.surfaces,sf),g.loc(lo),f,r.real());return true;}
  case 30:{const TopoDS_Edge& e=br_e(g,r);const TopoDS_Face& fc=br_f(g,r);double f=r.real();B.Range(e,fc,f,r.real());return true;}
  case 31:{const TopoDS_Edge& e1=br_e(g,r);B.Transfert(e1,br_e(g,r));return true;}
  case 32:{const TopoDS_Edge& e1=br_e(g,r);const TopoDS_Edge& e2=br_e(g,r);const TopoDS_Vertex& v1=br_v(g,r);B.Transfert(e1,e2,v1,br_v(g,r));return true;}
 }
 return false;}
// Vertex and face commands: 33 point v P tol, 34 on edge v p e tol, 35 on edge and face
// v p e f tol, 36 on edge and surface v p e s loc tol, 37 on face v u v f tol,
// 38 tolerance v tol, 39 face surface f s loc tol, 40 face triangulation f t reset,
// 41 face tolerance f tol, 42 natural restriction f b, 49 set UV points e s loc P P,
// 50 set UV points on face e f P P.
bool br_vertex_command(BrScene& s,BrReader& r,int code){
 BRep_Builder B;BrGraph& g=s.graph;BrGeometry& G=s.geometry;
 switch(code){
  case 33:{const TopoDS_Vertex& v=br_v(g,r);gp_Pnt p=br_point(r);B.UpdateVertex(v,p,r.real());return true;}
  case 34:{const TopoDS_Vertex& v=br_v(g,r);double p=r.real();const TopoDS_Edge& e=br_e(g,r);B.UpdateVertex(v,p,e,r.real());return true;}
  case 35:{const TopoDS_Vertex& v=br_v(g,r);double p=r.real();const TopoDS_Edge& e=br_e(g,r);const TopoDS_Face& f=br_f(g,r);B.UpdateVertex(v,p,e,f,r.real());return true;}
  case 36:{const TopoDS_Vertex& v=br_v(g,r);double p=r.real();const TopoDS_Edge& e=br_e(g,r);int sf=r.integer(),l=r.integer();B.UpdateVertex(v,p,e,br_at(G.surfaces,sf),g.loc(l),r.real());return true;}
  case 37:{const TopoDS_Vertex& v=br_v(g,r);double u=r.real(),w=r.real();const TopoDS_Face& f=br_f(g,r);B.UpdateVertex(v,u,w,f,r.real());return true;}
  case 38:{const TopoDS_Vertex& v=br_v(g,r);B.UpdateVertex(v,r.real());return true;}
  case 39:{const TopoDS_Face& f=br_f(g,r);int sf=r.integer(),l=r.integer();B.UpdateFace(f,br_at(G.surfaces,sf),g.loc(l),r.real());return true;}
  case 40:{const TopoDS_Face& f=br_f(g,r);int t=r.integer();B.UpdateFace(f,br_at(G.triangulations,t),r.flag());return true;}
  case 41:{const TopoDS_Face& f=br_f(g,r);B.UpdateFace(f,r.real());return true;}
  case 42:{const TopoDS_Face& f=br_f(g,r);B.NaturalRestriction(f,r.flag());return true;}
  case 49:{const TopoDS_Edge& e=br_e(g,r);int sf=r.integer(),l=r.integer();gp_Pnt2d p1=br_point2d(r);BRep_Tool::SetUVPoints(e,br_at(G.surfaces,sf),g.loc(l),p1,br_point2d(r));return true;}
  case 50:{const TopoDS_Edge& e=br_e(g,r);const TopoDS_Face& f=br_f(g,r);gp_Pnt2d p1=br_point2d(r);BRep_Tool::SetUVPoints(e,f,p1,br_point2d(r));return true;}
 }
 return false;}
void br_brep_command(BrScene& s,BrReader& r){
 int code=r.integer();
 if(code>=100){br_apply_command(s.graph,r,code-100);return;}
 try{
  bool known=br_make_command(s,r,code)||br_edge_command(s,r,code)||br_flag_command(s,r,code)||br_vertex_command(s,r,code);
  br_word(known?"-":"?");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
// The whole script of a BRep request.
BrScene br_scene(BrReader& r){
 BrScene s;int nl=r.integer();for(int i=0;i<nl;++i)br_read_location(s.graph,r);
 int ng=r.integer();for(int i=0;i<ng;++i)br_read_geometry(s.geometry,r);
 int nc=r.integer();for(int i=0;i<nc;++i)br_brep_command(s,r);return s;}
