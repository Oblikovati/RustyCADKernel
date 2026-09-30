// The shape reader of the TKTopAlgo oracle: vertices, edges, faces, shells and a solid
// (or a compound) built with BRep_Builder from numbers, with shared edges, 3d curves,
// pcurves, 3d polygons, triangulations and polygons on them, so both probes build the
// same shape without BRepBuilderAPI or BRepPrimAPI. The Rust twin is
// crates/tktopalgo/examples/tp_probe/shape.rs; tools/tktopalgo_corpus_shapes.py spells
// the records.
//
// A shape is `nv (x y z tol)... ne (edge)... nf (face)... ns (shell)... top`:
// - an edge is `curve vfirst vlast degenerated tol first last np (x y z)... [defl]`:
//   curve 0 none, 1 line P D, 2 circle P N X R (gp_Ax2(P, N, X)), 3 Bezier n (x y z)...,
//   4 B-spline degree n (x y z)... nk (knot mult)...; vfirst and vlast are vertex indices
//   (-1 for none, added FORWARD and REVERSED); np > 0 adds a 3d polygon of deflection
//   defl (the polygon alone makes the edge when there is no curve); the range [first,
//   last] is set on every curve of the edge once all the faces are built;
// - a face is `surface tol natural nw (wire)... nt [(x y z)... ntri (a b c)... defl
//   npol (edge n (node)... defl)...] orientation`: surface -1 none (the face is its
//   triangulation), 0 plane P N X, 1 cylinder P N X R, 2 sphere P N X R, 3 cone
//   P N X R semi-angle, 4 torus P N X R r (gp_Ax3(P, N, X) frames), 5 Bezier surface
//   nu nv (x y z)... (U rows); a wire is `orientation n (edge orientation pcurve)...`
//   with pcurve 0 none, 1 line u v du dv, 2 circle cu cv r sense, 3 Bezier n (u v)...;
//   an edge used twice in a face (a seam) gets the pcurves of its FORWARD and REVERSED
//   uses; nt > 0 adds a triangulation of nt nodes and polygons on it for some edges;
// - a shell is `orientation closed n (face orientation location)...`;
// - top is `n (kind index)... location`: kind 0 the solid of all the shells, 1 a shell,
//   2 a face, 3 an edge, 4 a vertex; one item is the shape itself, several make a
//   compound; a location is 0 identity, 1 translation dx dy dz, 2 rotation P D angle.
#include <BRep_Builder.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Line.hxx>
#include <Poly_Polygon3D.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Poly_Triangulation.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Wire.hxx>
struct TpShape{TopoDS_Shape shape;std::vector<TopoDS_Vertex> vertices;std::vector<TopoDS_Edge> edges;
 std::vector<TopoDS_Face> faces;std::vector<TopoDS_Shell> shells;};
occ::handle<Geom_Curve> tp_shape_curve(GaReader& r){
 int kind=r.integer();
 if(kind==1){gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());}
 if(kind==2){gp_Pnt p=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();return new Geom_Circle(gp_Ax2(p,n,x),r.real());}
 if(kind==3){int n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);for(int i=1;i<=n;++i)poles(i)=r.pnt();
  return new Geom_BezierCurve(poles);}
 if(kind==4){int degree=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);
  for(int i=1;i<=n;++i)poles(i)=r.pnt();NCollection_Array1<double> knots(1,1);NCollection_Array1<int> mults(1,1);
  tp_knots(r,knots,mults);return new Geom_BSplineCurve(poles,knots,mults,degree);}
 return occ::handle<Geom_Curve>();}
// One edge; its range and record tolerance (the tolerance of its pcurves) are kept.
TopoDS_Edge tp_shape_edge(GaReader& r,const std::vector<TopoDS_Vertex>& vertices,std::vector<std::pair<double,double>>& ranges,
                          std::vector<double>& tols){
 BRep_Builder B;TopoDS_Edge e;occ::handle<Geom_Curve> c=tp_shape_curve(r);
 int vf=r.integer(),vl=r.integer();bool degenerated=r.flag();double tol=r.real();tols.push_back(tol);
 double first=r.real(),last=r.real();ranges.emplace_back(first,last);
 int np=r.integer();occ::handle<Poly_Polygon3D> polygon;
 if(np>0){NCollection_Array1<gp_Pnt> nodes(1,np);for(int i=1;i<=np;++i)nodes(i)=r.pnt();
  polygon=new Poly_Polygon3D(nodes);polygon->Deflection(r.real());}
 if(!c.IsNull()){B.MakeEdge(e,c,tol);if(!polygon.IsNull())B.UpdateEdge(e,polygon);}
 else if(!polygon.IsNull())B.MakeEdge(e,polygon);
 else B.MakeEdge(e);
 B.UpdateEdge(e,tol);
 if(vf>=0)B.Add(e,vertices.at(vf).Oriented(TopAbs_FORWARD));
 if(vl>=0)B.Add(e,vertices.at(vl).Oriented(TopAbs_REVERSED));
 B.Degenerated(e,degenerated);return e;}
occ::handle<Geom_Surface> tp_shape_surface(GaReader& r,int kind){
 if(kind==5){int nu=r.integer(),nv=r.integer();return new Geom_BezierSurface(tp_poles(r,nu,nv));}
 gp_Pnt p=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();gp_Ax3 frame(p,n,x);
 if(kind==0)return new Geom_Plane(frame);
 if(kind==1)return new Geom_CylindricalSurface(frame,r.real());
 if(kind==2)return new Geom_SphericalSurface(frame,r.real());
 if(kind==3){double radius=r.real();return new Geom_ConicalSurface(frame,r.real(),radius);}
 double major=r.real();return new Geom_ToroidalSurface(frame,major,r.real());}
occ::handle<Geom2d_Curve> tp_shape_pcurve(GaReader& r){
 int kind=r.integer();
 if(kind==1){gp_Pnt2d p=r.pnt2d();return new Geom2d_Line(p,r.dir2d());}
 if(kind==2){gp_Pnt2d c=r.pnt2d();double radius=r.real();return new Geom2d_Circle(gp_Ax2d(c,gp_Dir2d(1.,0.)),radius,r.flag());}
 if(kind==3){int n=r.integer();NCollection_Array1<gp_Pnt2d> poles(1,n);for(int i=1;i<=n;++i)poles(i)=r.pnt2d();
  return new Geom2d_BezierCurve(poles);}
 return occ::handle<Geom2d_Curve>();}
// The pcurves of one face, per edge in order of first use: FORWARD and REVERSED uses.
struct TpUse{int edge;occ::handle<Geom2d_Curve> forward,reversed;};
void tp_shape_pcurves(const TopoDS_Face& face,const std::vector<TopoDS_Edge>& edges,const std::vector<TpUse>& uses,
                      const std::vector<double>& tols){
 BRep_Builder B;
 for(const TpUse& u:uses){const TopoDS_Edge& e=edges.at(u.edge);
  if(!u.forward.IsNull()&&!u.reversed.IsNull())B.UpdateEdge(e,u.forward,u.reversed,face,tols.at(u.edge));
  else if(!u.forward.IsNull())B.UpdateEdge(e,u.forward,face,tols.at(u.edge));
  else if(!u.reversed.IsNull())B.UpdateEdge(e,u.reversed,face,tols.at(u.edge));}}
TopoDS_Wire tp_shape_wire(GaReader& r,const std::vector<TopoDS_Edge>& edges,std::vector<TpUse>& uses){
 BRep_Builder B;TopoDS_Wire w;B.MakeWire(w);TopAbs_Orientation o=TopAbs_Orientation(r.integer());
 int n=r.integer();
 for(int j=0;j<n;++j){int k=r.integer();TopAbs_Orientation eo=TopAbs_Orientation(r.integer());
  occ::handle<Geom2d_Curve> c=tp_shape_pcurve(r);B.Add(w,edges.at(k).Oriented(eo));
  TpUse* use=nullptr;for(TpUse& u:uses)if(u.edge==k)use=&u;
  if(use==nullptr){uses.push_back(TpUse{k,{},{}});use=&uses.back();}
  if(eo==TopAbs_REVERSED)use->reversed=c;else use->forward=c;}
 w.Orientation(o);return w;}
// `nt (x y z)... ntri (a b c)... defl npol (edge n (node)... defl)...` after nt > 0.
occ::handle<Poly_Triangulation> tp_shape_mesh(GaReader& r,int nt,const std::vector<TopoDS_Edge>& edges){
 NCollection_Array1<gp_Pnt> nodes(1,nt);for(int i=1;i<=nt;++i)nodes(i)=r.pnt();
 int ntri=r.integer();NCollection_Array1<Poly_Triangle> triangles(1,ntri);
 for(int i=1;i<=ntri;++i){int a=r.integer(),b=r.integer();triangles(i)=Poly_Triangle(a,b,r.integer());}
 occ::handle<Poly_Triangulation> mesh=new Poly_Triangulation(nodes,triangles);mesh->Deflection(r.real());
 int npol=r.integer();BRep_Builder B;
 for(int i=0;i<npol;++i){int k=r.integer(),n=r.integer();NCollection_Array1<int> indices(1,n);
  for(int j=1;j<=n;++j)indices(j)=r.integer();
  occ::handle<Poly_PolygonOnTriangulation> polygon=new Poly_PolygonOnTriangulation(indices);
  polygon->Deflection(r.real());B.UpdateEdge(edges.at(k),polygon,mesh,TopLoc_Location());}
 return mesh;}
TopoDS_Face tp_shape_face(GaReader& r,const std::vector<TopoDS_Edge>& edges,const std::vector<double>& tols){
 BRep_Builder B;TopoDS_Face face;int kind=r.integer();occ::handle<Geom_Surface> s;
 if(kind>=0)s=tp_shape_surface(r,kind);
 double tol=r.real();bool natural=r.flag();
 if(!s.IsNull())B.MakeFace(face,s,tol);else{B.MakeFace(face);B.UpdateFace(face,tol);}
 B.NaturalRestriction(face,natural);
 std::vector<TpUse> uses;int nw=r.integer();
 for(int i=0;i<nw;++i){TopoDS_Wire w=tp_shape_wire(r,edges,uses);B.Add(face,w);}
 tp_shape_pcurves(face,edges,uses,tols);
 int nt=r.integer();if(nt>0)B.UpdateFace(face,tp_shape_mesh(r,nt,edges));
 face.Orientation(TopAbs_Orientation(r.integer()));return face;}
// The identity is the empty location (a location built on an identity gp_Trsf is not).
TopLoc_Location tp_shape_location(GaReader& r){
 int kind=r.integer();gp_Trsf t;
 if(kind==0)return TopLoc_Location();
 if(kind==1){double x=r.real(),y=r.real();t.SetTranslation(gp_Vec(x,y,r.real()));}
 else{gp_Pnt p=r.pnt();gp_Dir d=r.dir();t.SetRotation(gp_Ax1(p,d),r.real());}
 return TopLoc_Location(t);}
TopoDS_Shell tp_shape_shell(GaReader& r,const std::vector<TopoDS_Face>& faces){
 BRep_Builder B;TopoDS_Shell shell;B.MakeShell(shell);TopAbs_Orientation o=TopAbs_Orientation(r.integer());
 bool closed=r.flag();int n=r.integer();
 for(int i=0;i<n;++i){int k=r.integer();TopAbs_Orientation fo=TopAbs_Orientation(r.integer());
  TopLoc_Location l=tp_shape_location(r);B.Add(shell,faces.at(k).Oriented(fo).Located(l));}
 shell.Closed(closed);shell.Orientation(o);return shell;}
TopoDS_Shape tp_shape_item(const TpShape& s,int kind,int index){
 if(kind==0){BRep_Builder B;TopoDS_Solid solid;B.MakeSolid(solid);for(const TopoDS_Shell& x:s.shells)B.Add(solid,x);return solid;}
 if(kind==1)return s.shells.at(index);
 if(kind==2)return s.faces.at(index);
 if(kind==3)return s.edges.at(index);
 return s.vertices.at(index);}
TpShape tp_shape(GaReader& r){
 BRep_Builder B;TpShape s;s.vertices.resize(r.integer());
 for(TopoDS_Vertex& v:s.vertices){gp_Pnt p=r.pnt();B.MakeVertex(v,p,r.real());}
 int ne=r.integer();std::vector<std::pair<double,double>> ranges;std::vector<double> tols;
 for(int i=0;i<ne;++i)s.edges.push_back(tp_shape_edge(r,s.vertices,ranges,tols));
 int nf=r.integer();for(int i=0;i<nf;++i)s.faces.push_back(tp_shape_face(r,s.edges,tols));
 for(int i=0;i<ne;++i)B.Range(s.edges[i],ranges[i].first,ranges[i].second);
 int ns=r.integer();for(int i=0;i<ns;++i)s.shells.push_back(tp_shape_shell(r,s.faces));
 int n=r.integer();std::vector<TopoDS_Shape> items;
 for(int i=0;i<n;++i){int kind=r.integer();items.push_back(tp_shape_item(s,kind,r.integer()));}
 if(items.size()==1)s.shape=items[0];
 else{TopoDS_Compound c;B.MakeCompound(c);for(const TopoDS_Shape& x:items)B.Add(c,x);s.shape=c;}
 s.shape.Location(tp_shape_location(r));return s;}
