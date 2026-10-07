// ShapeUpgrade_UnifySameDomain and ShapeFix_FaceConnect probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/unify.rs.
//
// `sh_unify (input) init ue uf concat allow safe lintol angtol nkeep (kind index)...`:
// the input is
//  0 n (m (point)*m)*n tol: n planar polygon faces (BRepBuilderAPI_MakePolygon closed,
//    BRepBuilderAPI_MakeFace only plane) sewn with BRepBuilderAPI_Sewing(tol);
//  1 type angle count shell params: `count` lateral faces (or shells with the planar
//    sides and caps) of revolution primitives on gp_Ax2(origin, Z, X turned by k * angle),
//    each spanning `angle`, sewn with 1e-6;
//    type 0 cylinder R H, 1 cone R1 R2 H, 2 sphere R;
//  2 n (edge)*n: a wire (BRepBuilderAPI_MakeWire) of edges `0 P1 P2` (segment),
//    `1 C R a1 a2` (arc of the circle of axis Z) or `2 m (pole)*m` (Bezier curve);
//  3 dx dy dz: a box solid;
//  4 n (edge)*n: the planar face (only plane) of such a wire.
// With init the default constructor then Initialize, else the constructor with the
// shape. The kept shapes are edges (kind 0) or vertices (kind 1) by their index in the
// maps of the input. Build, then the digest of the result and, for every sub-shape of
// the input (TopExp::MapShapes), IsRemoved and the indices of its Modified images in the
// map of the result (-2 when absent).
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <ShapeFix_FaceConnect.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
TopoDS_Shape su_polygons(GaReader& r){
 int n=r.integer();std::vector<TopoDS_Shape> faces;
 for(int k=0;k<n;++k){int m=r.integer();BRepBuilderAPI_MakePolygon poly;
  for(int i=0;i<m;++i)poly.Add(r.pnt());poly.Close();
  faces.push_back(BRepBuilderAPI_MakeFace(poly.Wire(),true).Face());}
 BRepBuilderAPI_Sewing sewing(r.real());for(const TopoDS_Shape& f:faces)sewing.Add(f);
 sewing.Perform();return sewing.SewedShape();}
template<class M> TopoDS_Shape su_piece(M&& maker,bool shell){
 if(shell)return maker.Shell();return maker.Face();}
TopoDS_Shape su_lateral(int type,const gp_Ax2& ax,double angle,const std::vector<double>& p,bool shell){
 if(type==0)return su_piece(BRepPrimAPI_MakeCylinder(ax,p[0],p[1],angle),shell);
 if(type==1)return su_piece(BRepPrimAPI_MakeCone(ax,p[0],p[1],p[2],angle),shell);
 return su_piece(BRepPrimAPI_MakeSphere(ax,p[0],angle),shell);}
TopoDS_Shape su_revolved(GaReader& r){
 int type=r.integer();double angle=r.real();int count=r.integer();bool shell=r.flag();
 std::vector<double> params(type==0?2:(type==1?3:1));for(double& v:params)v=r.real();
 BRepBuilderAPI_Sewing sewing(1e-6);
 for(int k=0;k<count;++k){
  gp_Ax2 ax(gp_Pnt(0,0,0),gp_Dir(0,0,1),gp_Dir(std::cos(k*angle),std::sin(k*angle),0));
  sewing.Add(su_lateral(type,ax,angle,params,shell));}
 sewing.Perform();return sewing.SewedShape();}
TopoDS_Edge su_edge(GaReader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt a=r.pnt();return BRepBuilderAPI_MakeEdge(a,r.pnt()).Edge();}
 if(kind==1){gp_Pnt c=r.pnt();double R=r.real(),a1=r.real();
  occ::handle<Geom_Circle> circ=new Geom_Circle(gp_Ax2(c,gp_Dir(0,0,1)),R);
  return BRepBuilderAPI_MakeEdge(circ,a1,r.real()).Edge();}
 int m=r.integer();NCollection_Array1<gp_Pnt> poles(1,m);for(int i=1;i<=m;++i)poles(i)=r.pnt();
 occ::handle<Geom_BezierCurve> b=new Geom_BezierCurve(poles);return BRepBuilderAPI_MakeEdge(b).Edge();}
TopoDS_Shape su_wire(GaReader& r){
 int n=r.integer();BRepBuilderAPI_MakeWire mw;for(int k=0;k<n;++k)mw.Add(su_edge(r));return mw.Wire();}
TopoDS_Shape su_input(GaReader& r){
 int kind=r.integer();
 if(kind==0)return su_polygons(r);
 if(kind==1)return su_revolved(r);
 if(kind==2)return su_wire(r);
 if(kind==4)return BRepBuilderAPI_MakeFace(TopoDS::Wire(su_wire(r)),true).Face();
 double dx=r.real(),dy=r.real();return BRepPrimAPI_MakeBox(dx,dy,r.real()).Shape();}
void su_history(const TopoDS_Shape& input,const TopoDS_Shape& result,const occ::handle<BRepTools_History>& h){
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> in,out;TopExp::MapShapes(input,in);TopExp::MapShapes(result,out);
 ga_int(in.Extent());
 for(int i=1;i<=in.Extent();++i){const TopoDS_Shape& s=in(i);
  if(!BRepTools_History::IsSupportedType(s)){ga_int(-3);continue;}
  ga_flag(h->IsRemoved(s));const NCollection_List<TopoDS_Shape>& images=h->Modified(s);ga_int(images.Extent());
  for(NCollection_List<TopoDS_Shape>::Iterator it(images);it.More();it.Next()){int k=out.FindIndex(it.Value());ga_int(k==0?-2:k);}}}
void sh_unify_case(std::istream& input){
 GaReader r{input};TopoDS_Shape shape;GA_TRY(shape=su_input(r))
 bool init=r.flag(),ue=r.flag(),uf=r.flag(),concat=r.flag(),allow=r.flag(),safe=r.flag();
 double lin=r.real(),ang=r.real();int nkeep=r.integer();
 std::vector<std::pair<int,int>> keep;for(int k=0;k<nkeep;++k){int kind=r.integer();keep.push_back({kind,r.integer()});}
 if(shape.IsNull()){ga_end();return;}
 occ::handle<ShapeUpgrade_UnifySameDomain> u=init?new ShapeUpgrade_UnifySameDomain():new ShapeUpgrade_UnifySameDomain(shape,ue,uf,concat);
 if(init)u->Initialize(shape,ue,uf,concat);
 u->AllowInternalEdges(allow);u->SetSafeInputMode(safe);u->SetLinearTolerance(lin);u->SetAngularTolerance(ang);
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> edges,vertices;
 TopExp::MapShapes(shape,TopAbs_EDGE,edges);TopExp::MapShapes(shape,TopAbs_VERTEX,vertices);
 for(const auto& [kind,index]:keep){const auto& m=kind==0?edges:vertices;if(index>=1&&index<=m.Extent())u->KeepShape(m(index));}
 GA_TRY(u->Build();tp_shape(u->Shape());su_history(shape,u->Shape(),u->History()))
 ga_end();}
// `sh_face_connect n (m (point)*m)*n npairs (i j)* sewtol fixtol`: ShapeFix_FaceConnect on
// a shell (BRep_Builder) of n separate planar polygon faces with the pairs of face indices
// connected; the digest of the result (also a second Build on it with the same object).
void sh_face_connect_case(std::istream& input){
 GaReader r{input};BRep_Builder B;TopoDS_Shell shell;B.MakeShell(shell);std::vector<TopoDS_Face> faces;
 int n=r.integer();
 for(int k=0;k<n;++k){int m=r.integer();BRepBuilderAPI_MakePolygon poly;
  for(int i=0;i<m;++i)poly.Add(r.pnt());poly.Close();
  faces.push_back(BRepBuilderAPI_MakeFace(poly.Wire(),true).Face());B.Add(shell,faces.back());}
 ShapeFix_FaceConnect connector;int npairs=r.integer();
 for(int k=0;k<npairs;++k){int i=r.integer(),j=r.integer();ga_flag(connector.Add(faces.at(i),faces.at(j)));}
 double sewtol=r.real(),fixtol=r.real();
 GA_TRY(TopoDS_Shell result=connector.Build(shell,sewtol,fixtol);tp_shape(result);
  tp_shape(connector.Build(result,sewtol,fixtol)))
 ga_end();}
