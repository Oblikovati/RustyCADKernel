// The static members of GeomInt and GeomInt_IntSS and GeomInt_ParameterAndOrientation
// (tkgeomalgo_probe.cxx); the Rust twin is crates/tkgeomalgo/examples/ga_probe/geom_int_statics.rs.
#include <Bnd_Box2d.hxx>
#include <GeomInt.hxx>
#include <GeomInt_ParameterAndOrientation.hxx>
#include <IntPatch_RLine.hxx>
// `curve surface first last tol bounds [umin umax vmin vmax] preset [curve2d]`:
// GeomInt_IntSS::BuildPCurves (with the given bounds, or over the surface bounds), with
// a null or preset 2d curve; the 2d curve and the tolerance.
void ga_gi_pcurves_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);occ::handle<Geom_Surface> s=ga_ics_geom_surface(r);
  double f=r.real(),l=r.real(),tol=r.real();bool bounded=r.flag();double b[4]={0,0,0,0};if(bounded)for(double& x:b)x=r.real();
  occ::handle<Geom2d_Curve> c2d;if(r.flag())c2d=ga_curve2d(r);
  if(bounded){GA_TRY(GeomInt_IntSS::BuildPCurves(f,l,b[0],b[1],b[2],b[3],tol,s,c,c2d));}
  else{GA_TRY(GeomInt_IntSS::BuildPCurves(f,l,tol,s,c,c2d));}
  ga_gi_curve2d(c2d);ga_real(tol));
 ga_end();
}
// `has1 [curve2d] has2 [curve2d] box1 (u1 v1 u2 v2) box2 (...) n (param)*`:
// GeomInt_IntSS::TrimILineOnSurfBoundaries; the parameters.
void ga_gi_trim_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c1,c2;if(r.flag())c1=ga_curve2d(r);if(r.flag())c2=ga_curve2d(r);Bnd_Box2d b1,b2;
  for(Bnd_Box2d* b:{&b1,&b2}){gp_Pnt2d p=r.pnt2d();b->Add(p);p=r.pnt2d();b->Add(p);}
  NCollection_DynamicArray<double> prm;int n=r.integer();for(int i=0;i<n;++i)prm.Append(r.real());
  GA_TRY(GeomInt_IntSS::TrimILineOnSurfBoundaries(c1,c2,b1,b2,prm));ga_int(int(prm.Size()));for(double x:prm)ga_real(x));
 ga_end();
}
// `line ideb ifin`: GeomInt_IntSS::MakeBSpline and MakeBSpline2d (both surfaces) on a
// walking line (ga_wt_line).
void ga_gi_mkbsp_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<IntPatch_WLine> w=occ::down_cast<IntPatch_WLine>(ga_wt_line(r,fx));int a=r.integer(),b=r.integer();
  GA_TRY(ga_gi_curve(GeomInt_IntSS::MakeBSpline(w,a,b)));GA_TRY(ga_gi_curve2d(GeomInt_IntSS::MakeBSpline2d(w,a,b,true)));
  GA_TRY(ga_gi_curve2d(GeomInt_IntSS::MakeBSpline2d(w,a,b,false))));
 ga_end();
}
// `s1 s2 line preset1 preset2`: GeomInt_IntSS::TreatRLine on a restriction line (ga_wt_line)
// of the adaptors (ga_ics_surface), with null 2d curves or preset ones (a copy of the arc).
void ga_gi_rline_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);
  occ::handle<IntPatch_RLine> l=occ::down_cast<IntPatch_RLine>(ga_wt_line(r,fx));bool p1=r.flag(),p2=r.flag();
  occ::handle<Geom_Curve> c3d;occ::handle<Geom2d_Curve> c1,c2;
  if(p1)c1=new Geom2d_Line(gp_Pnt2d(0.25,0),gp_Dir2d(0,1));if(p2)c2=new Geom2d_Line(gp_Pnt2d(0,0.25),gp_Dir2d(1,0));
  double tol=-1;
  GA_TRY(GeomInt_IntSS::TreatRLine(l,occ::down_cast<GeomAdaptor_Surface>(s1),occ::down_cast<GeomAdaptor_Surface>(s2),c3d,c1,c2,tol));
  ga_gi_curve(c3d);ga_gi_curve2d(c1);ga_gi_curve2d(c2);ga_real(tol));
 ga_end();
}
// `n (par min max period eps)*`: GeomInt::AdjustPeriodic.
void ga_gi_adjust_case(std::istream& input){
 GaReader r{input};int n=r.integer();
 for(int i=0;i<n;++i){double p=r.real(),a=r.real(),b=r.real(),t=r.real(),e=r.real();double np=-1,o=-1;
  ga_flag(GeomInt::AdjustPeriodic(p,a,b,t,np,o,e));ga_real(np);ga_real(o);}
 ga_end();
}
// `p or1 or2 set1 set2`: GeomInt_ParameterAndOrientation (default and valued).
void ga_gi_porient_case(std::istream& input){
 GaReader r{input};double p=r.real();int a=r.integer(),b=r.integer(),c=r.integer(),d=r.integer();
 GeomInt_ParameterAndOrientation e;ga_real(e.Parameter());ga_int(e.Orientation1());ga_int(e.Orientation2());
 GeomInt_ParameterAndOrientation v(p,TopAbs_Orientation(a),TopAbs_Orientation(b));
 ga_real(v.Parameter());ga_int(v.Orientation1());ga_int(v.Orientation2());
 v.SetOrientation1(TopAbs_Orientation(c));v.SetOrientation2(TopAbs_Orientation(d));ga_int(v.Orientation1());ga_int(v.Orientation2());
 ga_end();
}
