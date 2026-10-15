// BRepFilletAPI and FilletSurf probes of tools/oracle/tkfillet_probe.cxx (the 3d fillets
// and chamfers through ChFi3d_FilBuilder and ChFi3d_ChBuilder, and the fillet surfaces of
// FilletSurf_Builder); the Rust twins are crates/tkfillet/examples/fl_probe/api.rs and
// fillet_surf.rs.
#include <BRep_Tool.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <FilletSurf_Builder.hxx>
#include <Geom_Surface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeWedge.hxx>
#include <GProp_GProps.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <gp_Ax2.hxx>

// The solid of a request: 0 the box 20 20 20, 1 the pyramid of the wedge 10 10 10 to
// the apex (5, 5), and the box 10 10 5 with 2 a boss (the cylinder of radius 2 and
// height 3 on its top face at (5, 5)), 3 a pocket (the box 4 4 3 at (3, 3, 3) cut off),
// 4 a notch (the box 10 10 7 at (5, 5, -1) cut off) and 5 a step (the box 5 10 3 at
// (0, 0, 5) fused), 6 the cylinder of radius 5 and height 10, 7 the box 10 10 10 with
// its vertical edge at (10, 10) filleted with radius 3.
TopoDS_Shape fl_api_box(double x,double y,double z,double dx,double dy,double dz){
 return BRepPrimAPI_MakeBox(gp_Pnt(x,y,z),dx,dy,dz).Shape();}
// The box 10 10 10 with its vertical edge at (10, 10) filleted with radius 3.
TopoDS_Shape fl_api_rounded_box(){
 TopoDS_Shape b=fl_api_box(0.,0.,0.,10.,10.,10.);
 BRepFilletAPI_MakeFillet f(b);
 for(TopExp_Explorer ex(b,TopAbs_EDGE);ex.More();ex.Next()){
  const TopoDS_Edge& e=TopoDS::Edge(ex.Current());
  gp_Pnt a=BRep_Tool::Pnt(TopExp::FirstVertex(e)),c=BRep_Tool::Pnt(TopExp::LastVertex(e));
  if(a.X()==10.&&a.Y()==10.&&c.X()==10.&&c.Y()==10.){f.Add(3.,e);break;}}
 return f.Shape();}
TopoDS_Shape fl_api_solid(int kind){
 if(kind==1)return BRepPrimAPI_MakeWedge(10.,10.,10.,5.,5.,5.,5.).Shape();
 TopoDS_Shape base=fl_api_box(0.,0.,0.,10.,10.,5.);
 if(kind==2){
  TopoDS_Shape cyl=BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5.,5.,5.),gp_Dir(0.,0.,1.)),2.,3.).Shape();
  return BRepAlgoAPI_Fuse(base,cyl).Shape();}
 if(kind==3)return BRepAlgoAPI_Cut(base,fl_api_box(3.,3.,3.,4.,4.,3.)).Shape();
 if(kind==4)return BRepAlgoAPI_Cut(base,fl_api_box(5.,5.,-1.,10.,10.,7.)).Shape();
 if(kind==5)return BRepAlgoAPI_Fuse(base,fl_api_box(0.,0.,5.,5.,10.,3.)).Shape();
 if(kind==6)return BRepPrimAPI_MakeCylinder(5.,10.).Shape();
 if(kind==7)return fl_api_rounded_box();
 return fl_api_box(0.,0.,0.,20.,20.,20.);}

// The result of an algorithm: done, the faulty contours and vertices, and when done the
// numbers of faces and edges, the volume, the area and the validity.
template<class Api> void fl_api_result(Api& api){
 ga_flag(api.IsDone());
 if(!api.IsDone())return;
 const TopoDS_Shape& s=api.Shape();
 int nf=0,ne=0;
 for(TopExp_Explorer ex(s,TopAbs_FACE);ex.More();ex.Next())nf++;
 for(TopExp_Explorer ex(s,TopAbs_EDGE);ex.More();ex.Next())ne++;
 ga_int(nf);ga_int(ne);
 GProp_GProps v;BRepGProp::VolumeProperties(s,v);ga_real(v.Mass());
 GProp_GProps a;BRepGProp::SurfaceProperties(s,a);ga_real(a.Mass());
 ga_flag(BRepCheck_Analyzer(s).IsValid());}

// `fl_make_fillet solid edge method d1 d2`: the fillet or chamfer of the `edge`-th edge
// of the map of the edges of the solid (all of them for 0). Methods: 0 fillet of radius d1,
// 1 fillet from d1 to d2, 2 chamfer d1, 3 chamfer d1 on the first face of the edge and
// d2, 4 chamfer d1 on the first face and the angle d2, 5 and 6 fillet of radius d1 with
// quasi-angular and polynomial sections.
void fl_make_fillet_case(std::istream& in){
 GaReader r{in};int kind=r.integer(),iedge=r.integer(),method=r.integer();double d1=r.real(),d2=r.real();
 GA_TRY(
  TopoDS_Shape s=fl_api_solid(kind);
  NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> ef;
  TopExp::MapShapesAndAncestors(s,TopAbs_EDGE,TopAbs_FACE,ef);
  std::vector<TopoDS_Edge> edges;
  for(int k=1;k<=ef.Extent();k++)if(iedge==0||k==iedge)edges.push_back(TopoDS::Edge(ef.FindKey(k)));
  if(method==0||method==1||method==5||method==6){
   BRepFilletAPI_MakeFillet api(s,method==5?ChFi3d_QuasiAngular:method==6?ChFi3d_Polynomial:ChFi3d_Rational);
   for(const TopoDS_Edge& e:edges){if(method==1)api.Add(d1,d2,e);else api.Add(d1,e);}
   api.Build();ga_int(api.NbFaultyContours());ga_int(api.NbFaultyVertices());fl_api_result(api);
  }else{
   BRepFilletAPI_MakeChamfer api(s);
   for(const TopoDS_Edge& e:edges){
    const TopoDS_Face& f=TopoDS::Face(ef.FindFromKey(e).First());
    if(method==2)api.Add(d1,e);else if(method==3)api.Add(d1,d2,e,f);else api.AddDA(d1,d2,e,f);}
   api.Build();fl_api_result(api);
  });
 ga_end();}

// `fl_fillet_surf solid e1 e2 e3 r simulate`: FilletSurf_Builder on the edges e1, e2 and
// e3 of the map of the edges of the solid (0 for none) with the radius r, computed or
// simulated: the statuses, the number of surfaces and, per surface, its tolerance and
// the ranges of its curves on the faces (the number of sections and the first one when
// simulated), then the parameters and statuses of the ends.
void fl_fillet_surf_case(std::istream& in){
 GaReader r{in};int kind=r.integer();int ie[3]={r.integer(),r.integer(),r.integer()};double rad=r.real();bool simul=r.flag();
 GA_TRY(
  TopoDS_Shape s=fl_api_solid(kind);
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> em;
  TopExp::MapShapes(s,TopAbs_EDGE,em);
  NCollection_List<TopoDS_Shape> E;
  for(int k=0;k<3;k++)if(ie[k]>0&&ie[k]<=em.Extent())E.Append(em(ie[k]));
  FilletSurf_Builder b(s,E,rad);
  if(simul)b.Simulate();else b.Perform();
  ga_int(b.IsDone());ga_int(b.StatusError());
  if(b.IsDone()!=FilletSurf_IsNotOk){
   int n=b.NbSurface();ga_int(n);
   for(int i=1;i<=n;i++){
    if(simul){
     int ns=b.NbSection(i);ga_int(ns);
     occ::handle<Geom_TrimmedCurve> c=b.Section(i,1);ga_real(c->FirstParameter());ga_real(c->LastParameter());
    }else{
     ga_real(b.TolApp3d(i));
     ga_real(b.CurveOnFace1(i)->FirstParameter());ga_real(b.CurveOnFace1(i)->LastParameter());
     ga_real(b.PCurveOnFace2(i)->FirstParameter());ga_real(b.PCurve2OnFillet(i)->LastParameter());
     double u1,u2,v1,v2;b.SurfaceFillet(i)->Bounds(u1,u2,v1,v2);ga_real(u1);ga_real(u2);ga_real(v1);ga_real(v2);
    }
   }
   ga_real(b.FirstParameter());ga_real(b.LastParameter());
   ga_int(b.StartSectionStatus());ga_int(b.EndSectionStatus());
  });
 ga_end();}
