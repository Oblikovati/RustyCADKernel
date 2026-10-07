// HLRBRep_Intersector, the Rust twin of crates/tkhlr/examples/hl_probe/hlrbrep_intersector.rs.
#include <HLRBRep_Intersector.hxx>
#include <IntCurveSurface_IntersectionSegment.hxx>
// An edge record on an edge with vertices, its curve updated by the projector.
void hb_set_edge(HLRBRep_EdgeData& d,const TopoDS_Edge& e,HLRAlgo_Projector& p){
 BRepAdaptor_Curve bc(e);double a=bc.FirstParameter(),b=bc.LastParameter();
 d.Set(false,false,e,1,2,false,false,false,false,a,1e-7f,b,1e-7f);d.ChangeGeometry().Projector(&p);
 double mn[16],mx[16];HLRAlgo::InitMinMax(1e100,mn,mx);d.ChangeGeometry().Update(mn,mx);}
void hb_inter_print(const HLRBRep_Intersector& i){
 ga_flag(i.IsDone());int n=i.NbPoints();ga_int(n);for(int k=1;k<=n;++k)GA_TRY(ga_ipoint(i.Point(k)));
 int ns=i.NbSegments();ga_int(ns);for(int k=1;k<=ns;++k)GA_TRY(ga_iseg(i.Segment(k)));}
void hb_cs_print(const HLRBRep_Intersector& i){
 ga_flag(i.IsDone());int n=i.NbPoints();ga_int(n);for(int k=1;k<=n;++k)GA_TRY(hb_ics_point(i.CSPoint(k)));ga_int(i.NbSegments());}
// `projector edge1 edge2 da1 db1 da2 db2 enbout u v face (x y P)*3`: the self
// intersection of edge 1, the two-edge Perform, SimulateOnePoint, then the sight lines
// through (x, y) of the view against the face (Load once, Perform(L, P) three times).
void hb_intersector_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector p=hb_projector(r);
 GA_TRY(TopoDS_Edge e1=hb_edge_v(r),e2=hb_edge_v(r);double da1=r.real(),db1=r.real(),da2=r.real(),db2=r.real();
  bool enbout=r.flag();double u=r.real(),v=r.real();TopoDS_Face f=hb_face(r);
  HLRBRep_EdgeData d1,d2;hb_set_edge(d1,e1,p);hb_set_edge(d2,e2,p);HLRBRep_Intersector i;
  GA_TRY(i.Perform(&d1,da1,db1);hb_inter_print(i));
  GA_TRY(i.Perform(1,&d1,da1,db1,2,&d2,da2,db2,enbout);hb_inter_print(i));
  GA_TRY(i.SimulateOnePoint(&d1,u,&d2,v);hb_inter_print(i));
  HLRBRep_Surface s;s.Projector(&p);s.Surface(f);i.Load(&s);
  for(int k=0;k<3;++k){double x=r.real(),y=r.real(),pp=r.real();GA_TRY(i.Perform(p.Shoot(x,y),pp);hb_cs_print(i));});
 ga_end();}
