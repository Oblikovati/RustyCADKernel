// HelixBRep probe operations; the Rust twin is crates/tkhelix/examples/hx_probe/helix_brep.rs
// and tools/tkhelix_corpus_helixbrep.py spells the requests. It needs the HelixGeom
// printers (hx_bspline, hx_shape) of tkhelix_probe_helixgeom.cxx, included before it.
#include <BRep_Tool.hxx>
#include <HelixBRep_BuilderHelix.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax3.hxx>
// `lo n (v)...`: an array of n values from the lower bound lo.
template<class T> NCollection_Array1<T> hxb_array(GaReader& r){
 int lo=r.integer(),n=r.integer();NCollection_Array1<T> a(lo,lo+n-1);
 for(int i=lo;i<lo+n;++i)a(i)=T(r.real());return a;}
// A vertex: its point and tolerance (nothing for a null vertex, after the null flag).
void hxb_vertex(const TopoDS_Vertex& v){
 ga_flag(v.IsNull());if(v.IsNull())return;ga_pnt(BRep_Tool::Pnt(v));ga_real(BRep_Tool::Tolerance(v));}
// An edge: orientation, tolerance, 3D curve range and B-spline, then its vertices.
void hxb_edge(const TopoDS_Edge& e){
 ga_int(int(e.Orientation()));ga_real(BRep_Tool::Tolerance(e));double f,l;
 occ::handle<Geom_Curve> c=BRep_Tool::Curve(e,f,l);ga_real(f);ga_real(l);
 hx_bspline(occ::down_cast<Geom_BSplineCurve>(c));
 TopoDS_Vertex v1,v2;TopExp::Vertices(e,v1,v2);hxb_vertex(v1);hxb_vertex(v2);}
// The result: null flag, type, orientation, then every edge of TopExp_Explorer.
void hxb_shape(const TopoDS_Shape& s){
 ga_flag(s.IsNull());if(s.IsNull())return;ga_int(int(s.ShapeType()));ga_int(int(s.Orientation()));
 int n=0;for(TopExp_Explorer x(s,TopAbs_EDGE);x.More();x.Next())++n;ga_int(n);
 for(TopExp_Explorer x(s,TopAbs_EDGE);x.More();x.Next())hxb_edge(TopoDS::Edge(x.Current()));}
// `mode data`: one SetParameters overload (mode 0 general, 1 one diameter, 2 spiral,
// 3 general by turns, 4 one diameter by turns, 5 spiral by turns).
void hxb_set(GaReader& r,HelixBRep_BuilderHelix& b,const gp_Ax3& ax){
 int mode=r.integer();
 if(mode==0){auto d=hxb_array<double>(r);auto h=hxb_array<double>(r);auto p=hxb_array<double>(r);auto f=hxb_array<bool>(r);b.SetParameters(ax,d,h,p,f);}
 else if(mode==1){double d=r.real();auto h=hxb_array<double>(r);auto p=hxb_array<double>(r);auto f=hxb_array<bool>(r);b.SetParameters(ax,d,h,p,f);}
 else if(mode==2){double d1=r.real(),d2=r.real();auto h=hxb_array<double>(r);auto p=hxb_array<double>(r);auto f=hxb_array<bool>(r);b.SetParameters(ax,d1,d2,h,p,f);}
 else if(mode==3){auto d=hxb_array<double>(r);auto p=hxb_array<double>(r);auto t=hxb_array<double>(r);b.SetParameters(ax,d,p,t);}
 else if(mode==4){double d=r.real();auto p=hxb_array<double>(r);auto t=hxb_array<double>(r);b.SetParameters(ax,d,p,t);}
 else{double d1=r.real(),d2=r.real();auto p=hxb_array<double>(r);auto t=hxb_array<double>(r);b.SetParameters(ax,d1,d2,p,t);}}
// The state after a step: statuses, reached tolerance and the shape.
void hxb_state(const HelixBRep_BuilderHelix& b){
 ga_int(b.ErrorStatus());ga_int(b.WarningStatus());ga_real(b.ToleranceReached());hxb_shape(b.Shape());}
// `P N X indirect nsteps (step)...` with a step `0 mode data` (SetParameters), `1 tol
// deg cont` (SetApproxParameters) or `2` (Perform); every step prints its exception
// type, if any, then the state. The frame is gp_Ax3(P, N, X), Y reversed when indirect.
void hxb_build_case(std::istream& input){
 GaReader r{input};gp_Pnt o=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();bool indirect=r.flag();
 gp_Ax3 ax(o,n,x);if(indirect)ax.YReverse();HelixBRep_BuilderHelix b;hxb_state(b);
 int steps=r.integer();
 for(int k=0;k<steps;++k){int kind=r.integer();
  if(kind==0){GA_TRY(hxb_set(r,b,ax))}
  else if(kind==1){double tol=r.real();int deg=r.integer(),cont=r.integer();b.SetApproxParameters(tol,deg,hx_shape(cont));}
  else{GA_TRY(b.Perform())}
  hxb_state(b);}
 ga_end();}
