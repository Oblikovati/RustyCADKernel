// Geom2dInt curve-curve probe operations (tkgeomalgo_probe.cxx): the polygon, the exact
// intersection point, the polygon-polygon intersector, Geom2dInt_GInter and
// Geom2dAPI_InterCurveCurve. The Rust twin is crates/tkgeomalgo/examples/ga_probe/ginter.rs.
// Curves are read by ga_curve2d (tkgeomalgo_probe_geom2dint.cxx), domains by ga_read_domain
// (tkgeomalgo_probe_intcurve.cxx) with their points on the curve. The computations run under
// GaMute: an index past the last polygon point makes ApproxParamOnCurve print
// "OutOfRange Polygon2d::ApproxParamOnCurve" on std::cout, which the port does not.
#include <Geom2dAPI_InterCurveCurve.hxx>
#include <Geom2dInt_ExactIntersectionPointOfTheIntPCurvePCurveOfGInter.hxx>
#include <Geom2dInt_GInter.hxx>
#include <Geom2dInt_TheIntPCurvePCurveOfGInter.hxx>
#include <Geom2dInt_ThePolygon2dOfTheIntPCurvePCurveOfGInter.hxx>
typedef Geom2dInt_ThePolygon2dOfTheIntPCurvePCurveOfGInter GaPoly;
IntRes2d_Domain ga_curve_domain(const GaDomainSpec& s,const Geom2dAdaptor_Curve& a){
 return ga_domain(s,[&](double u){return a.Value(u);});
}
void ga_box2d(const Bnd_Box2d& b){
 ga_flag(b.IsVoid());if(b.IsVoid())return;double x0,y0,x1,y1;b.Get(x0,y0,x1,y1);
 ga_real(x0);ga_real(y0);ga_real(x1);ga_real(y1);
}
// Every public member of the polygon, on the indices whose arrays are defined natively (the
// index slots past the point count may be uninitialized).
void ga_poly(const GaPoly& p){
 int nbseg=p.NbSegments();int npin=p.Closed()?nbseg:nbseg+1;
 ga_real(p.DeflectionOverEstimation());ga_int(nbseg);ga_flag(p.Closed());ga_box2d(p.Bounding());
 GA_TRY(ga_real(p.InfParameter()));GA_TRY(ga_real(p.SupParameter()));GA_TRY(ga_flag(p.AutoIntersectionIsPossible()));
 for(int i=0;i<=npin;i++){GA_TRY(gp_Pnt2d b,e;p.Segment(i,b,e);ga_pnt2d(b);ga_pnt2d(e));}
 for(int i=0;i<npin;i++)GA_TRY(ga_real(p.ApproxParamOnCurve(i,0.5)));
 GA_TRY(ga_real(p.ApproxParamOnCurve(npin,0.0)));
 ga_int(p.CalculRegion(-2.,3.,-1.,1.,-1.,1.));ga_int(p.CalculRegion(0.5,-3.,-1.,1.,-1.,1.));
}
// `curve domain n tol curve2 domain2 n2 tol2 deflection closed`: a polygon, the same after
// ComputeWithBox on the box of the second one, then after SetDeflectionOverEstimation and
// Closed.
void ga_g2int_polygon_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=ga_curve2d(r);GaDomainSpec s=ga_read_domain(r);int n=r.integer();double tol=r.real();
  occ::handle<Geom2d_Curve> c2=ga_curve2d(r);GaDomainSpec s2=ga_read_domain(r);int n2=r.integer();double tol2=r.real();
  double dfl=r.real();bool closed=r.flag();Geom2dAdaptor_Curve a(c),a2(c2);
  GaPoly p(a,n,ga_curve_domain(s,a),tol);ga_poly(p);
  GA_TRY(GaPoly q(a2,n2,ga_curve_domain(s2,a2),tol2);ga_box2d(q.Bounding());p.ComputeWithBox(a,q.Bounding());ga_poly(p));
  p.SetDeflectionOverEstimation(dfl);p.Closed(closed);ga_poly(p);p.Dump());
 ga_end();
}
void ga_eip_out(Geom2dInt_ExactIntersectionPointOfTheIntPCurvePCurveOfGInter& e){
 ga_int(e.NbRoots());if(e.NbRoots()>0){double u,v;e.Roots(u,v);ga_real(u);ga_real(v);}ga_flag(e.AnErrorOccurred());
}
// `curve1 domain1 n1 curve2 domain2 n2 tol seg1 par1 seg2 par2 uo vo uinf vinf usup vsup`: the
// exact point from a polygon seat (the seats printed after), then from explicit bounds.
void ga_g2int_exact_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c1=ga_curve2d(r);GaDomainSpec s1=ga_read_domain(r);int n1=r.integer();
  occ::handle<Geom2d_Curve> c2=ga_curve2d(r);GaDomainSpec s2=ga_read_domain(r);int n2=r.integer();double tol=r.real();
  int seg1=r.integer();double par1=r.real();int seg2=r.integer();double par2=r.real();
  double uo=r.real(),vo=r.real(),ui=r.real(),vi=r.real(),us=r.real(),vs=r.real();
  Geom2dAdaptor_Curve a1(c1),a2(c2);GaPoly p1(a1,n1,ga_curve_domain(s1,a1),tol);GaPoly p2(a2,n2,ga_curve_domain(s2,a2),tol);
  Geom2dInt_ExactIntersectionPointOfTheIntPCurvePCurveOfGInter e(a1,a2,tol);
  GA_TRY({GaMute m;e.Perform(p1,p2,seg1,seg2,par1,par2);}ga_int(seg1);ga_real(par1);ga_int(seg2);ga_real(par2);ga_eip_out(e));
  GA_TRY({GaMute m;e.Perform(uo,vo,ui,vi,us,vs);}ga_eip_out(e)));
 ga_end();
}
// The protected Perform overloads of the polygon-polygon intersector.
struct GaPolyPoly:Geom2dInt_TheIntPCurvePCurveOfGInter{using Geom2dInt_TheIntPCurvePCurveOfGInter::Perform;};
// `curve1 domain1 curve2 domain2 tolconf tol minnb reversed mode nbiter`: mode 0 the pair
// Perform, 1 the self Perform on the first curve, 2 and 3 their protected overloads at
// iteration nbiter (deltas 1).
void ga_g2int_polypoly_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c1=ga_curve2d(r);GaDomainSpec s1=ga_read_domain(r);
  occ::handle<Geom2d_Curve> c2=ga_curve2d(r);GaDomainSpec s2=ga_read_domain(r);double tc=r.real(),t=r.real();
  int minnb=r.integer();bool rev=r.flag();int mode=r.integer(),nbiter=r.integer();
  Geom2dAdaptor_Curve a1(c1),a2(c2);IntRes2d_Domain d1=ga_curve_domain(s1,a1),d2=ga_curve_domain(s2,a2);
  GaPolyPoly i;ga_int(i.GetMinNbSamples());i.SetMinNbSamples(minnb);ga_int(i.GetMinNbSamples());i.SetReversedParameters(rev);
  {GaMute m;if(mode==0)i.Perform(a1,d1,a2,d2,tc,t);else if(mode==1)i.Perform(a1,d1,tc,t);
  else if(mode==2)i.Perform(a1,d1,a2,d2,tc,t,nbiter,1.,1.);else i.Perform(a1,d1,tc,t,nbiter,1.,1.);}
  ga_inter(i));
 ga_end();
}
void ga_domain_out(const IntRes2d_Domain& d){
 ga_flag(d.HasFirstPoint());if(d.HasFirstPoint()){ga_real(d.FirstParameter());ga_pnt2d(d.FirstPoint());ga_real(d.FirstTolerance());}
 ga_flag(d.HasLastPoint());if(d.HasLastPoint()){ga_real(d.LastParameter());ga_pnt2d(d.LastPoint());ga_real(d.LastTolerance());}
 ga_flag(d.IsClosed());if(d.IsClosed()){double a,b;d.EquivalentParameters(a,b);ga_real(a);ga_real(b);}
}
// `curve1 domain1 curve2 domain2 tolconf tol minnb reversed mode ctor`: Geom2dInt_GInter by
// the constructor (ctor 1) or the Perform (after SetMinNbSamples and SetReversedParameters)
// of mode 0 (C1, C2), 1 (C1, D1, C2), 2 (C1, C2, D2), 3 (C1, D1, C2, D2), 4 (C) or 5 (C, D);
// then ComputeDomain of both curves.
void ga_g2int_ginter_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c1=ga_curve2d(r);GaDomainSpec s1=ga_read_domain(r);
  occ::handle<Geom2d_Curve> c2=ga_curve2d(r);GaDomainSpec s2=ga_read_domain(r);double tc=r.real(),t=r.real();
  int minnb=r.integer();bool rev=r.flag();int mode=r.integer();bool ctor=r.flag();
  Geom2dAdaptor_Curve a1(c1),a2(c2);IntRes2d_Domain d1=ga_curve_domain(s1,a1),d2=ga_curve_domain(s2,a2);
  if(ctor){
   GA_TRY(Geom2dInt_GInter i;{GaMute m;switch(mode){case 0:i=Geom2dInt_GInter(a1,a2,tc,t);break;
    case 1:i=Geom2dInt_GInter(a1,d1,a2,tc,t);break;case 2:i=Geom2dInt_GInter(a1,a2,d2,tc,t);break;
    case 3:i=Geom2dInt_GInter(a1,d1,a2,d2,tc,t);break;case 4:i=Geom2dInt_GInter(a1,tc,t);break;
    default:i=Geom2dInt_GInter(a1,d1,tc,t);}}ga_inter(i));
  }else{
   Geom2dInt_GInter i;ga_int(i.GetMinNbSamples());i.SetMinNbSamples(minnb);ga_int(i.GetMinNbSamples());i.SetReversedParameters(rev);
   GA_TRY({GaMute m;switch(mode){case 0:i.Perform(a1,a2,tc,t);break;case 1:i.Perform(a1,d1,a2,tc,t);break;
    case 2:i.Perform(a1,a2,d2,tc,t);break;case 3:i.Perform(a1,d1,a2,d2,tc,t);break;case 4:i.Perform(a1,tc,t);break;
    default:i.Perform(a1,d1,tc,t);}}ga_inter(i));
   GA_TRY(ga_domain_out(i.ComputeDomain(a1,tc)));GA_TRY(ga_domain_out(i.ComputeDomain(a2,t)));
  });
 ga_end();
}
void ga_trimmed_out(const occ::handle<Geom2d_Curve>& c){
 ga_real(c->FirstParameter());ga_real(c->LastParameter());GA_TRY(ga_pnt2d(c->Value(c->FirstParameter())));
 GA_TRY(ga_pnt2d(c->Value(c->LastParameter())));
}
// `curve1 self curve2 tol`: Geom2dAPI_InterCurveCurve of both curves or (self 1) of the first
// one: the points, the trimmed segment curves and the out-of-range errors.
void ga_g2api_intercc_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c1=ga_curve2d(r);bool self=r.flag();occ::handle<Geom2d_Curve> c2=ga_curve2d(r);double tol=r.real();
  Geom2dAPI_InterCurveCurve e;ga_int(e.NbPoints());ga_int(e.NbSegments());
  {GaMute m;if(self)e.Init(c1,tol);else e.Init(c1,c2,tol);}
  int np=e.NbPoints();ga_int(np);for(int i=0;i<=np+1;i++)GA_TRY(ga_pnt2d(e.Point(i)));
  int ns=e.NbSegments();ga_int(ns);
  for(int i=0;i<=ns+1;i++)GA_TRY(occ::handle<Geom2d_Curve> t1,t2;e.Segment(i,t1,t2);ga_trimmed_out(t1);ga_trimmed_out(t2));
  ga_inter(e.Intersector()));
 ga_end();
}
