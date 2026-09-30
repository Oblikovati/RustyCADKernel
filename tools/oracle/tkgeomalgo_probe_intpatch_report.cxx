// The printers of the IntPatch analytic intersections (tkgeomalgo_probe.cxx): the lines,
// their vertices and the arcs of both domains, shared by
// tkgeomalgo_probe_intpatch_impimp.cxx and tkgeomalgo_probe_intpatch_impprm.cxx; the Rust
// twin is crates/tkgeomalgo/examples/ga_probe/int_patch_report.rs. Needs ga_iw_pnt2s
// (tkgeomalgo_probe_intwalk.cxx) and ga_ip_trans (tkgeomalgo_probe_intpatch.cxx).
#include <Adaptor3d_TopolTool.hxx>
#include <IntPatch_ALine.hxx>
#include <IntPatch_GLine.hxx>
#include <IntPatch_RLine.hxx>
#include <IntPatch_WLine.hxx>
// The arcs of both domains, collected before the intersection runs: an arc prints its
// rank (1.. on the first domain, 101.. on the second one; 0 null, -1 another arc).
struct GaIplDomains{
 std::vector<occ::handle<Adaptor2d_Curve2d>> a1,a2;
 static void collect(const occ::handle<Adaptor3d_TopolTool>& d,std::vector<occ::handle<Adaptor2d_Curve2d>>& v){
  if(d.IsNull())return;for(d->Init();d->More();d->Next())v.push_back(d->Value());}
 GaIplDomains(const occ::handle<Adaptor3d_TopolTool>& d1,const occ::handle<Adaptor3d_TopolTool>& d2){collect(d1,a1);collect(d2,a2);}
 int rank(const occ::handle<Adaptor2d_Curve2d>& a)const{
  if(a.IsNull())return 0;for(size_t k=0;k<a1.size();++k)if(a==a1[k])return int(k)+1;
  for(size_t k=0;k<a2.size();++k)if(a==a2[k])return int(k)+101;return -1;}
};
// A vertex of a domain: null, or its 2d point and orientation.
void ga_ipl_hvertex(const occ::handle<Adaptor3d_HVertex>& v){
 ga_flag(v.IsNull());if(!v.IsNull()){ga_pnt2d(v->Value());ga_int(v->Orientation());}}
void ga_ipl_point(const GaIplDomains& dm,const IntPatch_Point& p){
 ga_pnt(p.Value());ga_real(p.ParameterOnLine());ga_real(p.Tolerance());ga_flag(p.IsTangencyPoint());ga_flag(p.IsMultiple());
 ga_iw_pnt2s(p.PntOn2S());
 ga_flag(p.IsOnDomS1());ga_flag(p.IsVertexOnS1());GA_TRY(ga_ipl_hvertex(p.VertexOnS1()));
 GA_TRY(ga_int(dm.rank(p.ArcOnS1())));GA_TRY(ga_ip_trans(p.TransitionLineArc1()));GA_TRY(ga_ip_trans(p.TransitionOnS1()));
 GA_TRY(ga_real(p.ParameterOnArc1()));
 ga_flag(p.IsOnDomS2());ga_flag(p.IsVertexOnS2());GA_TRY(ga_ipl_hvertex(p.VertexOnS2()));
 GA_TRY(ga_int(dm.rank(p.ArcOnS2())));GA_TRY(ga_ip_trans(p.TransitionLineArc2()));GA_TRY(ga_ip_trans(p.TransitionOnS2()));
 GA_TRY(ga_real(p.ParameterOnArc2()));
}
void ga_ipl_ax2(const gp_Ax2& a){ga_pnt(a.Location());ga_dir(a.Direction());ga_dir(a.XDirection());}
// The vertices `NbVertex` and `Vertex(0..n+1)` of a line class.
template<class L> void ga_ipl_vertices(const GaIplDomains& dm,const occ::handle<L>& l){
 int n=l->NbVertex();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(ga_ipl_point(dm,l->Vertex(i)));
 ga_flag(l->HasFirstPoint());ga_flag(l->HasLastPoint());}
void ga_ipl_gline(const GaIplDomains& dm,const occ::handle<IntPatch_GLine>& g){
 switch(g->ArcType()){
  case IntPatch_Lin:ga_pnt(g->Line().Location());ga_dir(g->Line().Direction());break;
  case IntPatch_Circle:ga_ipl_ax2(g->Circle().Position());ga_real(g->Circle().Radius());break;
  case IntPatch_Ellipse:ga_ipl_ax2(g->Ellipse().Position());ga_real(g->Ellipse().MajorRadius());ga_real(g->Ellipse().MinorRadius());break;
  case IntPatch_Parabola:ga_ipl_ax2(g->Parabola().Position());ga_real(g->Parabola().Focal());break;
  default:ga_ipl_ax2(g->Hyperbola().Position());ga_real(g->Hyperbola().MajorRadius());ga_real(g->Hyperbola().MinorRadius());}
 ga_ipl_vertices(dm,g);
}
// An analytic line: its domain and 5 of its points, then its vertices.
void ga_ipl_aline(const GaIplDomains& dm,const occ::handle<IntPatch_ALine>& a){
 bool inc=false;double f=0,l=0;GA_TRY(f=a->FirstParameter(inc);ga_real(f);ga_flag(inc));GA_TRY(l=a->LastParameter(inc);ga_real(l);ga_flag(inc));
 for(int k=0;k<=4;++k)GA_TRY(ga_pnt(a->Value(f+(l-f)*k/4.)));
 GA_TRY(gp_Pnt p;gp_Vec v;bool ok=a->D1(0.5*(f+l),p,v);ga_flag(ok);ga_pnt(p);if(ok)ga_xyz(v.XYZ()));
 ga_ipl_vertices(dm,a);
}
void ga_ipl_wline(const GaIplDomains& dm,const occ::handle<IntPatch_WLine>& w){
 int n=w->NbPnts();ga_int(n);for(int i=1;i<=n;++i)ga_iw_pnt2s(w->Point(i));
 ga_real(w->U1Period());ga_real(w->V1Period());ga_real(w->U2Period());ga_real(w->V2Period());
 ga_int(w->GetCreatingWay());ga_flag(w->IsPurgingAllowed());ga_flag(w->HasArcOnS1());ga_int(dm.rank(w->GetArcOnS1()));
 ga_flag(w->HasArcOnS2());ga_int(dm.rank(w->GetArcOnS2()));ga_ipl_vertices(dm,w);
}
void ga_ipl_rline(const GaIplDomains& dm,const occ::handle<IntPatch_RLine>& r){
 ga_flag(r->IsArcOnS1());ga_int(dm.rank(r->ArcOnS1()));ga_flag(r->IsArcOnS2());ga_int(dm.rank(r->ArcOnS2()));
 ga_flag(r->HasPolygon());GA_TRY(int n=r->NbPnts();ga_int(n);for(int i=1;i<=n;++i)ga_iw_pnt2s(r->Point(i)));
 ga_ipl_vertices(dm,r);
}
// A line: its type, tangency, transitions, situations and isos, then its own members.
void ga_ipl_line(const GaIplDomains& dm,const occ::handle<IntPatch_Line>& L){
 ga_int(L->ArcType());ga_flag(L->IsTangent());ga_int(L->TransitionOnS1());ga_int(L->TransitionOnS2());
 GA_TRY(ga_int(L->SituationS1()));GA_TRY(ga_int(L->SituationS2()));
 ga_flag(L->IsUIsoOnS1());ga_flag(L->IsVIsoOnS1());ga_flag(L->IsUIsoOnS2());ga_flag(L->IsVIsoOnS2());
 switch(L->ArcType()){
  case IntPatch_Analytic:ga_ipl_aline(dm,occ::down_cast<IntPatch_ALine>(L));break;
  case IntPatch_Walking:ga_ipl_wline(dm,occ::down_cast<IntPatch_WLine>(L));break;
  case IntPatch_Restriction:ga_ipl_rline(dm,occ::down_cast<IntPatch_RLine>(L));break;
  default:ga_ipl_gline(dm,occ::down_cast<IntPatch_GLine>(L));}
}
