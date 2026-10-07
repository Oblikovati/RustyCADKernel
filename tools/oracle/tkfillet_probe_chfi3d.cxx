// The ChFi3d operations of the TKFillet probe that run without the TKBool data structure.
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <ChFi3d.hxx>
#include <ChFi3d_Builder_0.hxx>
#include <ChFi3d_SearchSing.hxx>
#include <math_FunctionRoot.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <NCollection_List.hxx>
#include <TopTools_ShapeMapHasher.hxx>
using FlEdgeFaces=NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher>;
#include <GC_MakeSegment2d.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2dHatch_Hatcher.hxx>
#include <Geom2dHatch_Intersector.hxx>
#include <Geom2d_Line.hxx>
#include <HatchGen_Domain.hxx>
#include <gp_Ax2d.hxx>

// `fl_hatching_domains n P1..Pn P D tol keepSegments`: ChFi3d_ComputeHatchingDomains on
// the hatching line (P, D) trimmed by the closed polygon of the 2d points: the result,
// the points of the hatching (parameter, segment flags) and the domains.
void fl_hatching_domains_case(std::istream& in){
 GaReader r{in};int n=r.integer();std::vector<gp_Pnt2d> pts;
 for(int i=0;i<n;++i)pts.push_back(r.pnt2d());
 gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();double tol=r.real();bool keep=r.flag();
 Geom2dHatch_Intersector intersector(tol,tol);Geom2dHatch_Hatcher hatcher(intersector,tol,tol,false,keep);
 for(int i=0;i<n;++i)hatcher.AddElement(GC_MakeSegment2d(pts[i],pts[(i+1)%n]).Value(),TopAbs_FORWARD);
 occ::handle<Geom2d_Line> line=new Geom2d_Line(gp_Ax2d(p,d));
 int ih=hatcher.Trim(Geom2dAdaptor_Curve(line));
 GA_TRY(ga_flag(ChFi3d_ComputeHatchingDomains(hatcher,ih)));
 ga_int(hatcher.NbPoints(ih));
 for(int i=1;i<=hatcher.NbPoints(ih);++i){const HatchGen_PointOnHatching& pt=hatcher.Point(ih,i);
  ga_real(pt.Parameter());ga_flag(pt.SegmentBeginning());ga_flag(pt.SegmentEnd());}
 GA_TRY(ga_int(hatcher.NbDomains(ih));
  for(int i=1;i<=hatcher.NbDomains(ih);++i){const HatchGen_Domain& dom=hatcher.Domain(ih,i);
   ga_flag(dom.HasFirstPoint());if(dom.HasFirstPoint())ga_real(dom.FirstPoint().Parameter());
   ga_flag(dom.HasSecondPoint());if(dom.HasSecondPoint())ga_real(dom.SecondPoint().Parameter());});
 ga_end();}

// The face of the polygon of the 2d points (x, y) in z = 0.
TopoDS_Face fl_polygon_face(const std::vector<gp_Pnt2d>& pts){
 BRepBuilderAPI_MakePolygon poly;for(const gp_Pnt2d& p:pts)poly.Add(gp_Pnt(p.X(),p.Y(),0.0));poly.Close();
 return BRepBuilderAPI_MakeFace(poly.Wire()).Face();}

// `fl_solid kind a b c d`: 0 box (a, b, c), 1 L-shaped prism (legs a and b, thickness d,
// height c; its inner edge is concave), 2 cylinder (a, b), 3 cone (a, b, c), 4 sphere (a),
// 5 torus (a, b), 6 the rectangle [a, a + b] x [0, c] revolved by d around the Y axis.
TopoDS_Shape fl_solid(GaReader& r){
 int kind=r.integer();double a=r.real(),b=r.real(),c=r.real(),d=r.real();
 if(kind==0)return BRepPrimAPI_MakeBox(a,b,c).Shape();
 if(kind==1){TopoDS_Face f=fl_polygon_face({gp_Pnt2d(0,0),gp_Pnt2d(a,0),gp_Pnt2d(a,d),gp_Pnt2d(d,d),gp_Pnt2d(d,b),gp_Pnt2d(0,b)});
  return BRepPrimAPI_MakePrism(f,gp_Vec(0,0,c)).Shape();}
 if(kind==2)return BRepPrimAPI_MakeCylinder(a,b).Shape();
 if(kind==3)return BRepPrimAPI_MakeCone(a,b,c).Shape();
 if(kind==4)return BRepPrimAPI_MakeSphere(a).Shape();
 if(kind==5)return BRepPrimAPI_MakeTorus(a,b).Shape();
 TopoDS_Face f=fl_polygon_face({gp_Pnt2d(a,0),gp_Pnt2d(a+b,0),gp_Pnt2d(a+b,c),gp_Pnt2d(a,c)});
 return BRepPrimAPI_MakeRevol(f,gp_Ax1(gp_Pnt(0,0,0),gp_Dir(0,1,0)),d).Shape();}

// The faces of each edge of the solid, in the order of the edge map.
FlEdgeFaces fl_edge_faces(const TopoDS_Shape& s){
 FlEdgeFaces m;TopExp::MapShapesAndAncestors(s,TopAbs_EDGE,TopAbs_FACE,m);return m;}

// One ChFi3d::ConcaveSide call: the choice and both orientations.
void fl_concave(const TopoDS_Face& f1,const TopoDS_Face& f2,const TopoDS_Edge& e){
 GA_TRY(BRepAdaptor_Surface s1(f1),s2(f2);TopAbs_Orientation o1=TopAbs_INTERNAL,o2=TopAbs_INTERNAL;
  ga_int(ChFi3d::ConcaveSide(s1,s2,e,o1,o2));ga_int(int(o1));ga_int(int(o2)));}

// `fl_concave_side SOLID`: for each edge of the solid between two faces, ConcaveSide of
// the faces in both orders and of the reversed edge.
void fl_concave_side_case(std::istream& in){
 GaReader r{in};TopoDS_Shape s=fl_solid(r);FlEdgeFaces m=fl_edge_faces(s);
 ga_int(m.Extent());
 for(int i=1;i<=m.Extent();++i){const NCollection_List<TopoDS_Shape>& faces=m(i);ga_int(faces.Extent());
  if(faces.Extent()<2)continue;
  TopoDS_Face f1=TopoDS::Face(faces.First()),f2=TopoDS::Face(*std::next(faces.begin()));TopoDS_Edge e=TopoDS::Edge(m.FindKey(i));
  fl_concave(f1,f2,e);fl_concave(f2,f1,e);fl_concave(f1,f2,TopoDS::Edge(e.Reversed()));}
 ga_end();}

// `fl_search_sing GUIDE1 GUIDE2 n x1..xn x0 tol a b`: ChFi3d_SearchSing values at the
// parameters, and math_FunctionRoot from x0 in [a, b].
void fl_search_sing_case(std::istream& in){
 GaReader r{in};occ::handle<Adaptor3d_Curve> g1=fl_guide(r),g2=fl_guide(r);
 occ::handle<Geom_Curve> c1=occ::down_cast<GeomAdaptor_Curve>(g1)->Curve(),c2=occ::down_cast<GeomAdaptor_Curve>(g2)->Curve();
 ChFi3d_SearchSing f(c1,c2);int n=r.integer();
 for(int i=0;i<n;++i){double x=r.real(),v=0,d=0,v2=0,d2=0;
  GA_TRY(ga_flag(f.Value(x,v));ga_real(v);ga_flag(f.Derivative(x,d));ga_real(d);ga_flag(f.Values(x,v2,d2));ga_real(v2);ga_real(d2));}
 double x0=r.real(),tol=r.real(),a=r.real(),b=r.real();
 GA_TRY(math_FunctionRoot root(f,x0,tol,a,b);ga_flag(root.IsDone());
  if(root.IsDone()){ga_real(root.Root());ga_real(root.Value());ga_real(root.Derivative());ga_int(root.NbIterations());});
 ga_end();}
