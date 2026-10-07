// HLRBRep_EdgeData, HLRBRep_FaceData, HLRBRep_FaceIterator, HLRBRep_EdgeFaceTool and the
// HLRBRep package functions, the Rust twin of crates/tkhlr/examples/hl_probe/hlrbrep_records.rs.
#include <BRepAdaptor_Curve.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <HLRBRep.hxx>
#include <HLRBRep_EdgeData.hxx>
#include <HLRBRep_EdgeFaceTool.hxx>
#include <HLRBRep_FaceData.hxx>
#include <HLRBRep_FaceIterator.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Vertex.hxx>
// `geometry u1 u2`: an edge with its vertices (BRepBuilderAPI_MakeEdge).
TopoDS_Edge hb_edge_v(GaReader& r){
 occ::handle<Geom_Curve> c=hb_geometry(r);double u1=r.real(),u2=r.real();
 return BRepBuilderAPI_MakeEdge(c,u1,u2).Edge();}
// An edge: null flag, then its end points and three points of its curve.
void hb_edge_summary(const TopoDS_Edge& e){
 ga_flag(e.IsNull());if(e.IsNull())return;
 TopoDS_Vertex v1,v2;TopExp::Vertices(e,v1,v2);
 for(const TopoDS_Vertex& v:{v1,v2}){ga_flag(v.IsNull());if(!v.IsNull())ga_pnt(BRep_Tool::Pnt(v));}
 GA_TRY(BRepAdaptor_Curve c(e);double a=c.FirstParameter(),b=c.LastParameter();ga_real(a);ga_real(b);
  ga_int(c.GetType());for(double t:{a,(a+b)/2,b})ga_pnt(c.Value(t)));}
// The vertex indices, tolerance and status of a default EdgeData are uninitialized
// natively, so `full` is false before Set.
void hb_edge_flags(HLRBRep_EdgeData& d,bool full=true){
 ga_flag(d.Selected());ga_flag(d.Rg1Line());ga_flag(d.RgNLine());ga_flag(d.Vertical());ga_flag(d.Simple());
 ga_flag(d.OutLVSta());ga_flag(d.OutLVEnd());ga_flag(d.CutAtSta());ga_flag(d.CutAtEnd());ga_flag(d.VerAtSta());
 ga_flag(d.VerAtEnd());ga_flag(d.AutoIntersectionDone());ga_flag(d.Used());ga_int(d.HideCount());if(!full)return;
 ga_int(d.VSta());ga_int(d.VEnd());ga_real(d.Tolerance());
 double s,e;float ts,te;d.Status().Bounds(s,ts,e,te);ga_real(s);ga_real(ts);ga_real(e);ga_real(te);}
// `projector edge flags(9) start tolstart end tolend`: Set, the flags, then MakeEdge and
// MakeEdge3d on the updated curve at the ends and on an inner range.
void hb_edgedata_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);
 GA_TRY(TopoDS_Edge edge=hb_edge_v(r);bool f[9];for(bool& b:f)b=r.flag();
  double s=r.real();float ts=(float)r.real();double e=r.real();float te=(float)r.real();
  HLRBRep_EdgeData d;hb_edge_flags(d,false);
  d.Set(f[0],f[1],edge,3,7,f[2],f[3],f[4],f[5],s,ts,e,te);hb_edge_flags(d);
  d.Vertical(f[6]);d.Simple(f[7]);d.AutoIntersectionDone(f[8]);d.Used(true);d.HideCount(4);d.Selected(false);
  d.VerAtSta(true);d.VerAtEnd(f[8]);hb_edge_flags(d);
  HLRBRep_Curve& c=d.ChangeGeometry();c.Projector(&projector);double mn[16],mx[16];HLRAlgo::InitMinMax(1e100,mn,mx);
  GA_TRY(c.Update(mn,mx);double a=HLRBRep_BCurveTool::FirstParameter(c.Curve()),b=HLRBRep_BCurveTool::LastParameter(c.Curve());
   GA_TRY(hb_edge_summary(HLRBRep::MakeEdge(c,a,b)));GA_TRY(hb_edge_summary(HLRBRep::MakeEdge(c,a+(b-a)/4,b-(b-a)/3)));
   GA_TRY(hb_edge_summary(HLRBRep::MakeEdge3d(c,a,b)));GA_TRY(hb_edge_summary(HLRBRep::MakeEdge3d(c,a+(b-a)/4,b)))));
 ga_end();}
// `face or closed nw (ne)* (wi ewi ei or outl inte dble isol)* -1`: FaceData Set, SetWire,
// SetWEdge, then the flags and a FaceIterator walk.
void hb_facedata_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TopoDS_Face face=hb_face(r);int o=r.integer();bool cl=r.flag();int nw=r.integer();
  HLRBRep_FaceData fd;ga_flag(fd.Selected());ga_real(fd.Size());
  fd.Set(face,TopAbs_Orientation(o),cl,nw);
  for(int w=1;w<=nw;++w)fd.SetWire(w,r.integer());
  for(int wi=r.integer();wi>0;wi=r.integer()){int ewi=r.integer(),ei=r.integer(),eo=r.integer();
   bool a=r.flag(),b=r.flag(),c=r.flag(),dd=r.flag();fd.SetWEdge(wi,ewi,ei,TopAbs_Orientation(eo),a,b,c,dd);}
  fd.Back(true);fd.Side(o==1);fd.Hiding(cl);fd.Simple(true);fd.Cut(false);fd.WithOutL(cl);fd.Plane(true);
  fd.Cylinder(o==2);fd.Cone(false);fd.Sphere(true);fd.Torus(cl);fd.Size(2.5);fd.Selected(false);
  for(bool v:{fd.Selected(),fd.Back(),fd.Side(),fd.Closed(),fd.Hiding(),fd.Simple(),fd.Cut(),fd.WithOutL(),
   fd.Plane(),fd.Cylinder(),fd.Cone(),fd.Sphere(),fd.Torus()})ga_flag(v);
  ga_real(fd.Size());ga_int(fd.Orientation());ga_real(fd.Tolerance());ga_int(fd.Geometry().GetType());
  HLRBRep_FaceIterator it;
  for(it.InitEdge(fd);it.MoreEdge();it.NextEdge()){
   ga_int(it.Edge());ga_int(it.Orientation());ga_flag(it.OutLine());ga_flag(it.Internal());ga_flag(it.Double());
   ga_flag(it.IsoLine());ga_flag(it.BeginningOfWire());ga_flag(it.EndOfWire());ga_int(it.Wire()->NbEdges());});
 ga_end();}
// `projector face u v dir par edge`: EdgeFaceTool on the face: the curvature along `dir`,
// UVPoint of an edge of the face (p-curve) and of a free edge (projection); then the
// polyhedral angle and deflection of `par`.
void hb_edgeface_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);
 GA_TRY(TopoDS_Face face=hb_face(r);double u=r.real(),v=r.real();gp_Dir d=r.dir();double par=r.real();
  TopoDS_Edge free=hb_edge_v(r);
  HLRBRep_Surface s;s.Projector(&projector);s.Surface(face);
  GA_TRY(ga_real(HLRBRep_EdgeFaceTool::CurvatureValue(&s,u,v,d)));
  TopExp_Explorer ex(face,TopAbs_EDGE);
  for(const TopoDS_Edge& e:{TopoDS::Edge(ex.Current()),free}){
   HLRBRep_Curve c;c.Projector(&projector);c.Curve(e);
   double a=HLRBRep_BCurveTool::FirstParameter(c.Curve()),b=HLRBRep_BCurveTool::LastParameter(c.Curve());
   GA_TRY(double uu=-1,vv=-1;ga_flag(HLRBRep_EdgeFaceTool::UVPoint(a+(b-a)*par,&c,&s,uu,vv));ga_real(uu);ga_real(vv));}
  double oa,od;HLRBRep::PolyHLRAngleAndDeflection(par,oa,od);ga_real(oa);ga_real(od));
 ga_end();}
