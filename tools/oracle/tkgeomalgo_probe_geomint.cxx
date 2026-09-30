// GeomInt and GeomAPI_IntSS probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom_int.rs. Geom surfaces are read with
// ga_ics_geom_surface, adaptors with ga_ics_surface, domains and lines with ga_wt_domain
// and ga_wt_line (tkgeomalgo_probe_intpatch_wltool.cxx), and the curves are printed with
// the GeomAPI printers (tkgeomalgo_probe_geomapi_fit.cxx, tkgeomalgo_probe_geom2dapi.cxx).
#include <GeomAPI_IntSS.hxx>
#include <GeomInt_IntSS.hxx>
#include <GeomInt_LineConstructor.hxx>
#include <GeomInt_LineTool.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
// A 3d curve: the GeomAPI summary, trimmed flag, then the B-spline data (of the curve or
// of the basis of a trimmed curve).
void ga_gi_curve(const occ::handle<Geom_Curve>& c){
 ga_gapi_curve_out(c);if(c.IsNull())return;
 occ::handle<Geom_TrimmedCurve> t=occ::down_cast<Geom_TrimmedCurve>(c);ga_flag(t.IsNull());
 occ::handle<Geom_Curve> b=t.IsNull()?c:t->BasisCurve();
 occ::handle<Geom_BSplineCurve> s=occ::down_cast<Geom_BSplineCurve>(b);ga_flag(s.IsNull());if(!s.IsNull())ga_gapi_bspline_out(s);}
// A 2d curve, likewise.
void ga_gi_curve2d(const occ::handle<Geom2d_Curve>& c){
 ga_gapi_curve2d_out(c);if(c.IsNull())return;
 occ::handle<Geom2d_TrimmedCurve> t=occ::down_cast<Geom2d_TrimmedCurve>(c);ga_flag(t.IsNull());
 occ::handle<Geom2d_Curve> b=t.IsNull()?c:t->BasisCurve();
 occ::handle<Geom2d_BSplineCurve> s=occ::down_cast<Geom2d_BSplineCurve>(b);ga_flag(s.IsNull());if(!s.IsNull())ga_bspline_out(s);}
// Every result of an intersection, with the indices 0..N+1.
void ga_gi_intss_report(GeomInt_IntSS& s){
 ga_flag(s.IsDone());ga_real(s.TolReached3d());ga_real(s.TolReached2d());int n=s.NbLines();ga_int(n);
 for(int i=0;i<=n+1;++i){GA_TRY(ga_gi_curve(s.Line(i)));GA_TRY(ga_flag(s.HasLineOnS1(i)));GA_TRY(ga_gi_curve2d(s.LineOnS1(i)));
  GA_TRY(ga_flag(s.HasLineOnS2(i)));GA_TRY(ga_gi_curve2d(s.LineOnS2(i)));}
 GA_TRY(int nb=s.NbBoundaries();ga_int(nb);for(int i=0;i<=nb+1;++i)GA_TRY(ga_gi_curve(s.Boundary(i))));
 GA_TRY(int np=s.NbPoints();ga_int(np);for(int i=0;i<=np+1;++i){GA_TRY(ga_pnt(s.Point(i)));
  GA_TRY(ga_pnt2d(s.Pnt2d(i,true)));GA_TRY(ga_pnt2d(s.Pnt2d(i,false)));});}
// The adaptor of a Geom surface `flag [u1 u2 v1 v2]`.
occ::handle<GeomAdaptor_Surface> ga_gi_adaptor(GaReader& r,const occ::handle<Geom_Surface>& s){
 if(!r.flag())return new GeomAdaptor_Surface(s);double a=r.real(),b=r.real(),c=r.real(),d=r.real();
 return new GeomAdaptor_Surface(s,a,b,c,d);}
// `s1 s2 tol approx approxS1 approxS2 mode ...`: GeomInt_IntSS (SetTolFixTangents and
// TolFixTangents are declared without a definition, so they are not called);
// mode 0 the constructor, 1 Perform, 2 Perform from `u1 v1 u2 v2`, 3 Perform of the
// adaptors `ad1 ad2`, 4 of the adaptors from `u1 v1 u2 v2`, 5 Perform then Perform with
// the surfaces swapped, 6 Perform of S1 with itself.
void ga_gi_intss_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Surface> s1=ga_ics_geom_surface(r);occ::handle<Geom_Surface> s2=ga_ics_geom_surface(r);
  double tol=r.real();bool a=r.flag(),a1=r.flag(),a2=r.flag();int mode=r.integer();
  if(mode==0){GeomInt_IntSS s(s1,s2,tol,a,a1,a2);ga_gi_intss_report(s);ga_end();return;}
  GeomInt_IntSS s;
  if(mode==1){GA_TRY(s.Perform(s1,s2,tol,a,a1,a2));}
  else if(mode==2){double u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();GA_TRY(s.Perform(s1,s2,tol,u1,v1,u2,v2,a,a1,a2));}
  else if(mode==3||mode==4){occ::handle<GeomAdaptor_Surface> h1=ga_gi_adaptor(r,s1),h2=ga_gi_adaptor(r,s2);
   if(mode==3){GA_TRY(s.Perform(h1,h2,tol,a,a1,a2));}
   else{double u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();GA_TRY(s.Perform(h1,h2,tol,u1,v1,u2,v2,a,a1,a2));}}
  else if(mode==5){GA_TRY(s.Perform(s1,s2,tol,a,a1,a2));GA_TRY(s.Perform(s2,s1,tol,a,a1,a2));}
  else{GA_TRY(s.Perform(s1,s1,tol,a,a1,a2));}
  ga_gi_intss_report(s));
 ga_end();
}
// `s1 s2 tol mode`: GeomAPI_IntSS; mode 0 the constructor, 1 Perform, 2 empty.
void ga_gapi_intss_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Surface> s1=ga_ics_geom_surface(r);occ::handle<Geom_Surface> s2=ga_ics_geom_surface(r);
  double tol=r.real();int mode=r.integer();std::unique_ptr<GeomAPI_IntSS> s(mode==0?new GeomAPI_IntSS(s1,s2,tol):new GeomAPI_IntSS());
  if(mode==1){GA_TRY(s->Perform(s1,s2,tol));}
  ga_flag(s->IsDone());int n=s->NbLines();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(ga_gi_curve(s->Line(i))));
 ga_end();
}
// The parts of a line constructor, indices 0..N+1.
void ga_gi_lc_report(const GeomInt_LineConstructor& lc){
 ga_flag(lc.IsDone());GA_TRY(int n=lc.NbParts();ga_int(n);
  for(int i=0;i<=n+1;++i)GA_TRY(double f=0,l=0;lc.Part(i,f,l);ga_real(f);ga_real(l)));}
// `s1 s2 dom1 dom2 line`: GeomInt_LineConstructor on a line (ga_wt_line) of the adaptor
// surfaces (ga_ics_surface); the parts, then the parts after a second Perform.
void ga_gi_lconst_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);
  occ::handle<Adaptor3d_TopolTool> d1=ga_wt_domain(r,fx,s1);occ::handle<Adaptor3d_TopolTool> d2=ga_wt_domain(r,fx,s2);
  occ::handle<IntPatch_Line> l=ga_wt_line(r,fx);GeomInt_LineConstructor lc;ga_gi_lc_report(lc);
  lc.Load(d1,d2,occ::down_cast<GeomAdaptor_Surface>(s1),occ::down_cast<GeomAdaptor_Surface>(s2));
  GA_TRY(lc.Perform(l));ga_gi_lc_report(lc);GA_TRY(lc.Perform(l));ga_gi_lc_report(lc));
 ga_end();
}
// The lines of the intersection of two adaptors `s1 s2 tol` with their own domains.
struct GaGiInter{occ::handle<Adaptor3d_Surface> s1,s2;occ::handle<Adaptor3d_TopolTool> d1,d2;IntPatch_Intersection in;};
void ga_gi_inter(GaReader& r,GaGiInter& g){
 g.s1=ga_ics_surface(r);g.s2=ga_ics_surface(r);double tol=r.real();
 g.d1=new Adaptor3d_TopolTool(g.s1);g.d2=new Adaptor3d_TopolTool(g.s2);
 g.in.Perform(g.s1,g.d1,g.s2,g.d2,tol,tol);}
// `s1 s2 tol`: the line constructor and the line tool on every line of the intersection
// of the adaptors, then the decomposition of the walking lines (tolerance 2e-7).
void ga_gi_lines_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(GaGiInter g;ga_gi_inter(r,g);ga_flag(g.in.IsDone());int n=g.in.NbLines();ga_int(n);
  occ::handle<GeomAdaptor_Surface> h1=occ::down_cast<GeomAdaptor_Surface>(g.s1),h2=occ::down_cast<GeomAdaptor_Surface>(g.s2);
  for(int i=1;i<=n;++i){const occ::handle<IntPatch_Line>& l=g.in.Line(i);
   GA_TRY(int nv=GeomInt_LineTool::NbVertex(l);ga_int(nv);for(int k=0;k<=nv+1;++k)GA_TRY(ga_ip_point(fx,GeomInt_LineTool::Vertex(l,k))));
   GA_TRY(ga_real(GeomInt_LineTool::FirstParameter(l)));GA_TRY(ga_real(GeomInt_LineTool::LastParameter(l)));
   GeomInt_LineConstructor lc;lc.Load(g.d1,g.d2,h1,h2);GA_TRY(lc.Perform(l));ga_gi_lc_report(lc);
   occ::handle<IntPatch_WLine> w=occ::down_cast<IntPatch_WLine>(l);if(w.IsNull())continue;
   NCollection_Sequence<occ::handle<IntPatch_Line>> nl;
   GA_TRY(ga_flag(GeomInt_LineTool::DecompositionOfWLine(w,h1,h2,2.e-7,lc,nl)));ga_int(nl.Length());
   for(int k=1;k<=nl.Length();++k)ga_wt_report(fx,nl(k));});
 ga_end();
}
// `line`: the line tool on a line (ga_wt_line).
void ga_gi_ltool_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<IntPatch_Line> l=ga_wt_line(r,fx);
  GA_TRY(int nv=GeomInt_LineTool::NbVertex(l);ga_int(nv);for(int k=0;k<=nv+1;++k)GA_TRY(ga_ip_point(fx,GeomInt_LineTool::Vertex(l,k))));
  GA_TRY(ga_real(GeomInt_LineTool::FirstParameter(l)));GA_TRY(ga_real(GeomInt_LineTool::LastParameter(l))));
 ga_end();
}
