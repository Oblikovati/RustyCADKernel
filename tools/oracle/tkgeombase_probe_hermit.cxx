// TKGeomBase oracle: the Hermit package (tkgeombase_probe.cxx). The B-spline is read with
// the LProp builders (tkgeombase_probe_lprop_build.cxx); the result a(u) prints its poles,
// knots and multiplicities. The Rust twin is crates/tkgeombase/examples/gb_probe/hermit.rs.
#include <Hermit.hxx>
void gb_hermit_curve(const occ::handle<Geom2d_BSplineCurve>& c){
 gb_int(c->Degree());gb_int(c->NbPoles());for(int i=1;i<=c->NbPoles();++i)gb_pnt2d(c->Pole(i));
 gb_int(c->NbKnots());for(int i=1;i<=c->NbKnots();++i){gb_real(c->Knot(i));gb_int(c->Multiplicity(i));}
}
// hermit: mode 0 Solution (3D), 1 Solution (2D), 2 Solutionbis (3D, from the initial
// Knotmin and Knotmax); TolPoles, TolKnots, Knotmin, Knotmax, then the curve.
void gb_hermit_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double tp=r.real(),tk=r.real(),kmin=r.real(),kmax=r.real();
 try{
  if(mode==1){occ::handle<Geom2d_BSplineCurve> c=occ::down_cast<Geom2d_BSplineCurve>(gb_lp_curve2d(r));
   gb_hermit_curve(Hermit::Solution(c,tp,tk));}
  else{occ::handle<Geom_BSplineCurve> c=occ::down_cast<Geom_BSplineCurve>(gb_lp_curve3d(r));
   if(mode==0)gb_hermit_curve(Hermit::Solution(c,tp,tk));
   else{double a=kmin,b=kmax;Hermit::Solutionbis(c,a,b,tp,tk);gb_real(a);gb_real(b);}}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
