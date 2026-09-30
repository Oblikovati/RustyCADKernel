// The shape program reader of the TKTopAlgo oracle: any shape built with BRep_Builder
// from numbers (no BRepBuilderAPI), so the Rust twin
// crates/tktopalgo/examples/tp_probe/shapes.rs builds the same shapes with its BRep port.
//
// A program is `nops (op)...`; each op fills one of the tables surfaces, curves (3d),
// pcurves (2d), locations (the identity at index 0), triangulations and shapes, or
// changes a shape of the table:
//  1 surface: the surface record of tp_surface (tktopalgo_probe_face.cxx)
//  2 curve: 0 line P D; 1 circle P N R; 2 ellipse P N R1 R2; 3 Bezier n P...;
//    4 trimmed c u1 u2; 5 B-spline deg n P... nk (knot mult)...
//  3 pcurve: 0 line P D; 1 circle C R sense; 2 ellipse C R1 R2; 3 Bezier n P...;
//    4 trimmed c u1 u2; 5 offset c d; 6 B-spline deg n P... nk (knot mult)...
//  4 location: 0 translation dx dy dz; 1 rotation ax ay az angle (about the origin);
//    2 product i j
//  5 framed plane P N X (Geom_Plane(gp_Ax3(P, N, X)))
//  10 vertex x y z tol; 11 empty edge; 19 container kind (0 wire, 1 shell, 2 solid,
//    3 compound, 4 compsolid); 20 face s loc tol (s = -1: a face without surface);
//    21 oriented copy i o; 22 located copy i loc
//  12 edge curve e c loc tol (c = -1: a null curve); 13 pcurve e pc s loc tol;
//  14 closed pcurves e pc1 pc2 s loc tol; 15 range e f l only3d; 16 range on surface
//    e s loc f l; 17 flags e sameparameter samerange degenerated; 18 add parent child o
//    (o = -1: the child with its own orientation);
//  23 vertex on curve v t e tol; 24 vertex on pcurve v t e s loc tol; 25 vertex on
//    face v u w f tol; 26 flag i which (0 Closed, 1 Infinite) value; 27 tolerance
//    update i tol; 28 triangulation n (x y z)... nt (i j k)...;
//  29 polygon on triangulation e tri loc (polygon); 30 on a closed triangulation
//    e tri loc (polygon) (polygon), a polygon being n (node)... hasparams (t)... deflection;
//  33 pcurve on face e pc f tol; 34 closed pcurves on face e pc1 pc2 f tol;
//  35 range on face e f first last; 36 an extra 3d curve representation e c loc
//    (appended to the curves of the TEdge, as a second 3d curve).
#include <BRep_Builder.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <BRep_Curve3D.hxx>
#include <BRep_TEdge.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Poly_Triangulation.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_CompSolid.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Trsf.hxx>
struct TpShapes{
 std::vector<occ::handle<Geom_Surface>> surfaces;std::vector<occ::handle<Geom_Curve>> curves;
 std::vector<occ::handle<Geom2d_Curve>> pcurves;std::vector<TopLoc_Location> locations{TopLoc_Location()};
 std::vector<occ::handle<Poly_Triangulation>> triangulations;std::vector<TopoDS_Shape> shapes;};
// `n (knot mult)...` and the B-spline built on them.
void tp_spline_knots(GaReader& r,NCollection_Array1<double>& k,NCollection_Array1<int>& m){
 int n=r.integer();k.Resize(1,n,false);m.Resize(1,n,false);
 for(int i=1;i<=n;++i){k(i)=r.real();m(i)=r.integer();}}
occ::handle<Geom_Curve> tp_curve3d(GaReader& r,const TpShapes& t){
 int kind=r.integer();
 if(kind==0){gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());}
 if(kind==1||kind==2){gp_Pnt p=r.pnt();gp_Dir n=r.dir();double r1=r.real();
  if(kind==1)return new Geom_Circle(gp_Ax2(p,n),r1);return new Geom_Ellipse(gp_Ax2(p,n),r1,r.real());}
 if(kind==4){int c=r.integer();double u1=r.real();return new Geom_TrimmedCurve(t.curves.at(c),u1,r.real());}
 int deg=kind==5?r.integer():0;int n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);
 for(int i=1;i<=n;++i)poles(i)=r.pnt();
 if(kind==3)return new Geom_BezierCurve(poles);
 NCollection_Array1<double> k(1,1);NCollection_Array1<int> m(1,1);tp_spline_knots(r,k,m);
 return new Geom_BSplineCurve(poles,k,m,deg);}
occ::handle<Geom2d_Curve> tp_curve2d(GaReader& r,const TpShapes& t){
 int kind=r.integer();
 if(kind==0){gp_Pnt2d p=r.pnt2d();return new Geom2d_Line(p,r.dir2d());}
 if(kind==1){gp_Pnt2d c=r.pnt2d();double radius=r.real();return new Geom2d_Circle(gp_Ax2d(c,gp_Dir2d(1.,0.)),radius,r.flag());}
 if(kind==2){gp_Pnt2d c=r.pnt2d();double r1=r.real();return new Geom2d_Ellipse(gp_Ax2d(c,gp_Dir2d(1.,0.)),r1,r.real());}
 if(kind==4){int c=r.integer();double u1=r.real();return new Geom2d_TrimmedCurve(t.pcurves.at(c),u1,r.real());}
 if(kind==5){int c=r.integer();return new Geom2d_OffsetCurve(t.pcurves.at(c),r.real());}
 int deg=kind==6?r.integer():0;int n=r.integer();NCollection_Array1<gp_Pnt2d> poles(1,n);
 for(int i=1;i<=n;++i)poles(i)=r.pnt2d();
 if(kind==3)return new Geom2d_BezierCurve(poles);
 NCollection_Array1<double> k(1,1);NCollection_Array1<int> m(1,1);tp_spline_knots(r,k,m);
 return new Geom2d_BSplineCurve(poles,k,m,deg);}
TopLoc_Location tp_location(GaReader& r,const TpShapes& t){
 int kind=r.integer();gp_Trsf T;
 if(kind==2){int i=r.integer();return t.locations.at(i)*t.locations.at(r.integer());}
 if(kind==0){double x=r.real(),y=r.real();T.SetTranslation(gp_Vec(x,y,r.real()));}
 else{gp_Dir d=r.dir();T.SetRotation(gp_Ax1(gp_Pnt(0.,0.,0.),d),r.real());}
 return TopLoc_Location(T);}
// One polygon on triangulation: n (node)... hasparams (t)... deflection.
occ::handle<Poly_PolygonOnTriangulation> tp_polygon(GaReader& r){
 int n=r.integer();NCollection_Array1<int> nodes(1,n);for(int i=1;i<=n;++i)nodes(i)=r.integer();
 occ::handle<Poly_PolygonOnTriangulation> p;
 if(r.flag()){NCollection_Array1<double> t(1,n);for(int i=1;i<=n;++i)t(i)=r.real();p=new Poly_PolygonOnTriangulation(nodes,t);}
 else p=new Poly_PolygonOnTriangulation(nodes);
 p->Deflection(r.real());return p;}
occ::handle<Poly_Triangulation> tp_triangulation(GaReader& r){
 int n=r.integer();NCollection_Array1<gp_Pnt> nodes(1,n);for(int i=1;i<=n;++i)nodes(i)=r.pnt();
 int nt=r.integer();NCollection_Array1<Poly_Triangle> tris(1,nt);
 for(int i=1;i<=nt;++i){int a=r.integer(),b=r.integer();tris(i)=Poly_Triangle(a,b,r.integer());}
 return new Poly_Triangulation(nodes,tris);}
TopoDS_Shape tp_container(int kind){
 BRep_Builder B;
 if(kind==0){TopoDS_Wire w;B.MakeWire(w);return w;}
 if(kind==1){TopoDS_Shell s;B.MakeShell(s);return s;}
 if(kind==2){TopoDS_Solid s;B.MakeSolid(s);return s;}
 if(kind==3){TopoDS_Compound c;B.MakeCompound(c);return c;}
 TopoDS_CompSolid c;B.MakeCompSolid(c);return c;}
// The ops creating a geometry or a shape.
bool tp_create_op(GaReader& r,TpShapes& t,int op){
 BRep_Builder B;
 if(op==1){t.surfaces.push_back(tp_surface(r));return true;}
 if(op==2){t.curves.push_back(tp_curve3d(r,t));return true;}
 if(op==3){t.pcurves.push_back(tp_curve2d(r,t));return true;}
 if(op==4){t.locations.push_back(tp_location(r,t));return true;}
 if(op==5){gp_Pnt p=r.pnt();gp_Dir n=r.dir();t.surfaces.push_back(new Geom_Plane(gp_Ax3(p,n,r.dir())));return true;}
 if(op==28){t.triangulations.push_back(tp_triangulation(r));return true;}
 if(op==10){gp_Pnt p=r.pnt();TopoDS_Vertex v;B.MakeVertex(v,p,r.real());t.shapes.push_back(v);return true;}
 if(op==11){TopoDS_Edge e;B.MakeEdge(e);t.shapes.push_back(e);return true;}
 if(op==19){t.shapes.push_back(tp_container(r.integer()));return true;}
 if(op==20){int s=r.integer(),l=r.integer();double tol=r.real();TopoDS_Face f;
  if(s<0)B.MakeFace(f);else B.MakeFace(f,t.surfaces.at(s),t.locations.at(l),tol);t.shapes.push_back(f);return true;}
 if(op==21){int i=r.integer();t.shapes.push_back(t.shapes.at(i).Oriented(TopAbs_Orientation(r.integer())));return true;}
 if(op==22){int i=r.integer();t.shapes.push_back(t.shapes.at(i).Located(t.locations.at(r.integer())));return true;}
 return false;}
// The ops changing the representations of an edge.
bool tp_edge_op(GaReader& r,TpShapes& t,int op){
 BRep_Builder B;
 if(op<12||op>17)return false;
 TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));
 if(op==12){int c=r.integer(),l=r.integer();double tol=r.real();
  B.UpdateEdge(e,c<0?occ::handle<Geom_Curve>():t.curves.at(c),t.locations.at(l),tol);}
 else if(op==13){int c=r.integer(),s=r.integer(),l=r.integer();double tol=r.real();
  B.UpdateEdge(e,c<0?occ::handle<Geom2d_Curve>():t.pcurves.at(c),t.surfaces.at(s),t.locations.at(l),tol);}
 else if(op==14){int c1=r.integer(),c2=r.integer(),s=r.integer(),l=r.integer();double tol=r.real();
  B.UpdateEdge(e,t.pcurves.at(c1),t.pcurves.at(c2),t.surfaces.at(s),t.locations.at(l),tol);}
 else if(op==15){double f=r.real(),l=r.real();B.Range(e,f,l,r.flag());}
 else if(op==16){int s=r.integer(),l=r.integer();double f=r.real();B.Range(e,t.surfaces.at(s),t.locations.at(l),f,r.real());}
 else{bool sp=r.flag(),sr=r.flag();B.SameParameter(e,sp);B.SameRange(e,sr);B.Degenerated(e,r.flag());}
 return true;}
// The ops on vertices, flags, tolerances, polygons and face-bound pcurves.
void tp_other_op(GaReader& r,TpShapes& t,int op){
 BRep_Builder B;int i=r.integer();
 if(op==18){int c=r.integer(),o=r.integer();B.Add(t.shapes.at(i),o<0?t.shapes.at(c):t.shapes.at(c).Oriented(TopAbs_Orientation(o)));}
 else if(op==23){double p=r.real();int e=r.integer();B.UpdateVertex(TopoDS::Vertex(t.shapes.at(i)),p,TopoDS::Edge(t.shapes.at(e)),r.real());}
 else if(op==24){double p=r.real();int e=r.integer(),s=r.integer(),l=r.integer();
  B.UpdateVertex(TopoDS::Vertex(t.shapes.at(i)),p,TopoDS::Edge(t.shapes.at(e)),t.surfaces.at(s),t.locations.at(l),r.real());}
 else if(op==25){double u=r.real(),v=r.real();int f=r.integer();
  B.UpdateVertex(TopoDS::Vertex(t.shapes.at(i)),u,v,TopoDS::Face(t.shapes.at(f)),r.real());}
 else if(op==26){int which=r.integer();bool value=r.flag();
  if(which==0)t.shapes.at(i).Closed(value);else t.shapes.at(i).Infinite(value);}
 else if(op==27){double tol=r.real();const TopoDS_Shape& s=t.shapes.at(i);
  if(s.ShapeType()==TopAbs_VERTEX)B.UpdateVertex(TopoDS::Vertex(s),tol);
  else if(s.ShapeType()==TopAbs_EDGE)B.UpdateEdge(TopoDS::Edge(s),tol);else B.UpdateFace(TopoDS::Face(s),tol);}
 else if(op==29||op==30){int tri=r.integer(),l=r.integer();occ::handle<Poly_PolygonOnTriangulation> p1=tp_polygon(r);
  if(op==29)B.UpdateEdge(TopoDS::Edge(t.shapes.at(i)),p1,t.triangulations.at(tri),t.locations.at(l));
  else B.UpdateEdge(TopoDS::Edge(t.shapes.at(i)),p1,tp_polygon(r),t.triangulations.at(tri),t.locations.at(l));}
 else if(op==33){int c=r.integer(),f=r.integer();B.UpdateEdge(TopoDS::Edge(t.shapes.at(i)),t.pcurves.at(c),TopoDS::Face(t.shapes.at(f)),r.real());}
 else if(op==34){int c1=r.integer(),c2=r.integer(),f=r.integer();
  B.UpdateEdge(TopoDS::Edge(t.shapes.at(i)),t.pcurves.at(c1),t.pcurves.at(c2),TopoDS::Face(t.shapes.at(f)),r.real());}
 else if(op==35){int f=r.integer();double first=r.real();B.Range(TopoDS::Edge(t.shapes.at(i)),TopoDS::Face(t.shapes.at(f)),first,r.real());}
 else{int c=r.integer();const TopoDS_Shape& e=t.shapes.at(i);
  occ::down_cast<BRep_TEdge>(e.TShape())->ChangeCurves().Append(new BRep_Curve3D(t.curves.at(c),t.locations.at(r.integer())));}}
TpShapes tp_shapes(GaReader& r){
 TpShapes t;int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(!tp_create_op(r,t,op)&&!tp_edge_op(r,t,op))tp_other_op(r,t,op);}
 return t;}
// The index of `s` (IsEqual) in the table, -1 for a null shape, -2 when it is not there.
int tp_shape_index(const TpShapes& t,const TopoDS_Shape& s){
 if(s.IsNull())return -1;
 for(size_t i=0;i<t.shapes.size();++i)if(t.shapes[i].IsEqual(s))return int(i);
 return -2;}
// The same with IsSame.
int tp_same_index(const TpShapes& t,const TopoDS_Shape& s){
 if(s.IsNull())return -1;
 for(size_t i=0;i<t.shapes.size();++i)if(t.shapes[i].IsSame(s))return int(i);
 return -2;}
