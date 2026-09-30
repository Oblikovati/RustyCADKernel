// GeomFill filling-part probe operations: the generator, the filling schemes and the
// fillings of two to four curves (tkgeomalgo_probe.cxx). The Rust twin is
// crates/tkgeomalgo/examples/ga_probe/gfill_fill.rs; the curve reader is in
// tkgeomalgo_probe_gfill.cxx.
#include <GeomFill_BSplineCurves.hxx>
#include <GeomFill_BezierCurves.hxx>
#include <GeomFill_Coons.hxx>
#include <GeomFill_Curved.hxx>
#include <GeomFill_Stretch.hxx>
#include <Geom_BezierSurface.hxx>
// The fields of a B-spline or Bezier surface (poles and weights by index).
template<class S> void ga_gf_dump_surface_poles(const S& s){
 ga_int(s->UDegree());ga_int(s->VDegree());ga_flag(s->IsUPeriodic());ga_flag(s->IsVPeriodic());
 ga_flag(s->IsURational());ga_flag(s->IsVRational());ga_int(s->NbUPoles());ga_int(s->NbVPoles());
 for(int i=1;i<=s->NbUPoles();i++)for(int j=1;j<=s->NbVPoles();j++){ga_pnt(s->Pole(i,j));ga_real(s->Weight(i,j));}
}
void ga_gf_dump_bsurface(const occ::handle<Geom_BSplineSurface>& s){
 if(s.IsNull()){std::cout<<"null ";return;}
 ga_gf_dump_surface_poles(s);
 for(double k:s->UKnots())ga_real(k);for(int m:s->UMultiplicities())ga_int(m);
 for(double k:s->VKnots())ga_real(k);for(int m:s->VMultiplicities())ga_int(m);
}
void ga_gf_dump_bzsurface(const occ::handle<Geom_BezierSurface>& s){
 if(s.IsNull()){std::cout<<"null ";return;}
 ga_gf_dump_surface_poles(s);
}
// `ptol curves...`: GeomFill_Generator on the sections.
void ga_gfill_generator_case(std::istream& input){
 GaReader r{input};double ptol=r.real();GeomFill_Generator g;ga_gf_add_curves(r,g);
 ga_gf_dump_bsurface(occ::down_cast<Geom_BSplineSurface>(g.Surface()));
 GA_TRY(g.Perform(ptol));GA_TRY(ga_int(g.Degree()));
 ga_gf_dump_bsurface(occ::down_cast<Geom_BSplineSurface>(g.Surface()));
 ga_end();
}
// A pole row `lower n points` and a weight row `lower n values`.
NCollection_Array1<gp_Pnt> ga_gf_pnt_row(GaReader& r){
 int lower=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt> a(lower,lower+n-1);for(int i=lower;i<lower+n;i++)a(i)=r.pnt();return a;
}
NCollection_Array1<double> ga_gf_real_row(GaReader& r){
 int lower=r.integer(),n=r.integer();NCollection_Array1<double> a(lower,lower+n-1);for(int i=lower;i<lower+n;i++)a(i)=r.real();return a;
}
// The result of a scheme: rationality, sizes, poles (assigned to a 0-based grid) and
// weights (assigned when `weights`).
void ga_gf_dump_filling(const GeomFill_Filling& f,bool weights){
 ga_flag(f.isRational());ga_int(f.NbUPoles());ga_int(f.NbVPoles());
 NCollection_Array2<gp_Pnt> p(0,0,0,0);f.Poles(p);ga_int(p.LowerRow());ga_int(p.LowerCol());
 for(int i=p.LowerRow();i<=p.UpperRow();i++)for(int j=p.LowerCol();j<=p.UpperCol();j++)ga_pnt(p(i,j));
 if(!weights)return;
 NCollection_Array2<double> w(0,0,0,0);f.Weights(w);ga_int(w.ColLength());ga_int(w.RowLength());
 for(int i=w.LowerRow();i<=w.UpperRow();i++)for(int j=w.LowerCol();j<=w.UpperCol();j++)ga_real(w(i,j));
}
// `scheme weighted rows... [weights...] again`: scheme 0 Stretch, 1 Coons, 2 Curved
// (four rows), 3 Curved (two rows); rows and weights as `ga_gf_pnt_row`. When
// `again`, the same object is initialized a second time without weights (the stale
// weights of the first Init stay readable).
void ga_gfill_scheme_case(std::istream& input){
 GaReader r{input};int scheme=r.integer();bool weighted=r.flag();int n=scheme==3?2:4;
 std::vector<NCollection_Array1<gp_Pnt>> p;std::vector<NCollection_Array1<double>> w;
 for(int k=0;k<n;k++)p.push_back(ga_gf_pnt_row(r));
 if(weighted)for(int k=0;k<n;k++)w.push_back(ga_gf_real_row(r));
 bool again=r.flag();
 GeomFill_Stretch st;GeomFill_Coons co;GeomFill_Curved cu;
 GeomFill_Filling* f=scheme==0?(GeomFill_Filling*)&st:scheme==1?(GeomFill_Filling*)&co:(GeomFill_Filling*)&cu;
 auto init=[&](bool withW){
  if(scheme==0){if(withW)st.Init(p[0],p[1],p[2],p[3],w[0],w[1],w[2],w[3]);else st.Init(p[0],p[1],p[2],p[3]);}
  else if(scheme==1){if(withW)co.Init(p[0],p[1],p[2],p[3],w[0],w[1],w[2],w[3]);else co.Init(p[0],p[1],p[2],p[3]);}
  else if(scheme==2){if(withW)cu.Init(p[0],p[1],p[2],p[3],w[0],w[1],w[2],w[3]);else cu.Init(p[0],p[1],p[2],p[3]);}
  else {if(withW)cu.Init(p[0],p[1],w[0],w[1]);else cu.Init(p[0],p[1]);}
 };
 // A Standard_DomainError leaves the grids null (never read: a null dereference); any
 // later failure leaves the partial pole grid, dumped without the weights.
 std::string error;try{init(weighted);}catch(const Standard_Failure& e){error=e.ExceptionType();std::cout<<error<<' ';}
 bool ok=error.empty();
 if(ok||error!="Standard_DomainError")GA_TRY(ga_gf_dump_filling(*f,ok&&weighted));
 if(again&&ok){GA_TRY(init(false));GA_TRY(ga_gf_dump_filling(*f,weighted));}
 ga_end();
}
// The poles of a Bezier curve (the caller's curves, to see the Init(C1, C2) reversal).
void ga_gf_dump_bezier_poles(const occ::handle<Geom_BezierCurve>& c){for(const gp_Pnt& p:c->Poles())ga_pnt(p);}
// `style n curves...`: GeomFill_BezierCurves on 2, 3 or 4 Bezier curves.
void ga_gfill_bezier_case(std::istream& input){
 GaReader r{input};GeomFill_FillingStyle style=GeomFill_FillingStyle(r.integer());int n=r.integer();
 std::vector<occ::handle<Geom_BezierCurve>> c;for(int k=0;k<n;k++){r.integer();c.push_back(ga_gf_bezier(r));}
 GeomFill_BezierCurves f;ga_gf_dump_bzsurface(f.Surface());
 GA_TRY(if(n==4)f.Init(c[0],c[1],c[2],c[3],style);else if(n==3)f.Init(c[0],c[1],c[2],style);else f.Init(c[0],c[1],style));
 ga_gf_dump_bzsurface(f.Surface());ga_gf_dump_bezier_poles(c[0]);
 ga_end();
}
// `style n curves...`: GeomFill_BSplineCurves on 2, 3 or 4 B-spline curves.
void ga_gfill_bspline_case(std::istream& input){
 GaReader r{input};GeomFill_FillingStyle style=GeomFill_FillingStyle(r.integer());int n=r.integer();
 std::vector<occ::handle<Geom_BSplineCurve>> c;for(int k=0;k<n;k++){r.integer();c.push_back(ga_gf_bspline(r));}
 GeomFill_BSplineCurves f;ga_gf_dump_bsurface(f.Surface());
 GA_TRY(if(n==4)f.Init(c[0],c[1],c[2],c[3],style);else if(n==3)f.Init(c[0],c[1],c[2],style);else f.Init(c[0],c[1],style));
 ga_gf_dump_bsurface(f.Surface());ga_gf_dump_bspline(c[0]);
 ga_end();
}
