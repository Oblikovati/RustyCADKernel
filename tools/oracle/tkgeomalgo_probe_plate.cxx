// Plate probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/plate.rs.
#include <Plate_D1.hxx>
#include <Plate_D2.hxx>
#include <Plate_D3.hxx>
#include <Plate_FreeGtoCConstraint.hxx>
#include <Plate_GlobalTranslationConstraint.hxx>
#include <Plate_GtoCConstraint.hxx>
#include <Plate_LineConstraint.hxx>
#include <Plate_LinearScalarConstraint.hxx>
#include <Plate_LinearXYZConstraint.hxx>
#include <Plate_PinpointConstraint.hxx>
#include <Plate_PlaneConstraint.hxx>
#include <Plate_Plate.hxx>
#include <Plate_SampledCurveConstraint.hxx>
#include <gp_Lin.hxx>
#include <gp_Pln.hxx>
// `u v x y z idu idv`: a pinpoint constraint.
Plate_PinpointConstraint ga_plate_ppc(GaReader& r){double u=r.real(),v=r.real();gp_XYZ p=r.xyz();int iu=r.integer(),iv=r.integer();return Plate_PinpointConstraint(gp_XY(u,v),p,iu,iv);}
void ga_plate_print_ppc(const Plate_PinpointConstraint& c){ga_xy(c.Pnt2d());ga_xyz(c.Value());ga_int(c.Idu());ga_int(c.Idv());}
void ga_plate_print_lxyz(const Plate_LinearXYZConstraint& c){
 const NCollection_Array2<double>& k=c.Coeff();ga_int(k.ColLength());ga_int(k.RowLength());
 for(int j=1;j<=c.GetPPC().Length();j++)ga_plate_print_ppc(c.GetPPC()(j));
 for(int i=k.LowerRow();i<=k.UpperRow();i++)for(int j=k.LowerCol();j<=k.UpperCol();j++)ga_real(k(i,j));
}
void ga_plate_print_lsc(const Plate_LinearScalarConstraint& c){
 const NCollection_Array2<gp_XYZ>& k=c.Coeff();ga_int(k.ColLength());ga_int(k.RowLength());
 for(int j=1;j<=c.GetPPC().Length();j++)ga_plate_print_ppc(c.GetPPC()(j));
 for(int i=k.LowerRow();i<=k.UpperRow();i++)for(int j=k.LowerCol();j<=k.UpperCol();j++)ga_xyz(k(i,j));
}
// `rows cols ppc*cols coef*(rows*cols)`, with gp_XYZ coefficients when `scalar`.
Plate_LinearXYZConstraint ga_plate_lxyz(GaReader& r){
 int rows=r.integer(),cols=r.integer();Plate_LinearXYZConstraint c(rows,cols);
 for(int j=1;j<=cols;j++)c.SetPPC(j,ga_plate_ppc(r));
 for(int i=1;i<=rows;i++)for(int j=1;j<=cols;j++)c.SetCoeff(i,j,r.real());
 return c;
}
Plate_LinearScalarConstraint ga_plate_lsc(GaReader& r){
 int rows=r.integer(),cols=r.integer();Plate_LinearScalarConstraint c(rows,cols);
 for(int j=1;j<=cols;j++)c.SetPPC(j,ga_plate_ppc(r));
 for(int i=1;i<=rows;i++)for(int j=1;j<=cols;j++)c.SetCoeff(i,j,r.xyz());
 return c;
}
// The evaluation block of a plate: done, continuity, box, values and derivatives at the
// points, the polynomial coefficients.
void ga_plate_report(const Plate_Plate& p,const std::vector<double>& uv){
 ga_flag(p.IsDone());ga_int(p.Continuity());
 GA_TRY(double a,b,c,d;p.UVBox(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d));
 for(size_t i=0;i+1<uv.size();i+=2){gp_XY q(uv[i],uv[i+1]);
  ga_xyz(p.Evaluate(q));ga_xyz(p.EvaluateDerivative(q,1,0));ga_xyz(p.EvaluateDerivative(q,0,1));
  ga_xyz(p.EvaluateDerivative(q,1,1));ga_xyz(p.EvaluateDerivative(q,2,0));}
 if(p.IsDone()){occ::handle<NCollection_HArray2<gp_XYZ>> k=p.CoefPol();
  for(int i=k->LowerRow();i<=k->UpperRow();i++)for(int j=k->LowerCol();j<=k->UpperCol();j++)ga_xyz(k->Value(i,j));}
 NCollection_Sequence<gp_XY> seq;p.UVConstraints(seq);ga_int(seq.Length());
}
// `order aniso pponly npp ppc... nxyz lxyz... nsc lsc... uv...`: load, solve, report,
// then report a copy (copy constructor) and an assigned plate (Copy).
void ga_plate_solve_case(std::istream& input){
 GaReader r{input};int ord=r.integer();double aniso=r.real();bool pponly=r.flag();
 Plate_Plate p;int n=r.integer();for(int i=0;i<n;i++)p.Load(ga_plate_ppc(r));
 n=r.integer();for(int i=0;i<n;i++)p.Load(ga_plate_lxyz(r));
 n=r.integer();for(int i=0;i<n;i++)p.Load(ga_plate_lsc(r));
 std::vector<double> uv=r.reals();
 GA_TRY(p.SolveTI(ord,aniso));
 if(pponly)p.SetPolynomialPartOnly(true);
 ga_plate_report(p,uv);
 Plate_Plate q(p);ga_plate_report(q,uv);
 Plate_Plate a;a.Load(Plate_PinpointConstraint(gp_XY(9.,9.),gp_XYZ(1.,1.,1.)));a=p;ga_plate_report(a,uv);
 ga_end();
}
Plate_D1 ga_plate_d1(GaReader& r){gp_XYZ a=r.xyz();return Plate_D1(a,r.xyz());}
Plate_D2 ga_plate_d2(GaReader& r){gp_XYZ a=r.xyz(),b=r.xyz();return Plate_D2(a,b,r.xyz());}
Plate_D3 ga_plate_d3(GaReader& r){gp_XYZ a=r.xyz(),b=r.xyz(),c=r.xyz();return Plate_D3(a,b,c,r.xyz());}
// Loads `c` with three corner values into an order-3 plate and reports it at (0.3, 0.4).
template<class C> void ga_plate_solve_with(const C& c){
 Plate_Plate p;p.Load(c);
 p.Load(Plate_PinpointConstraint(gp_XY(0.,0.),gp_XYZ(0.,0.,0.)));
 p.Load(Plate_PinpointConstraint(gp_XY(1.,0.),gp_XYZ(1.,0.,0.5)));
 p.Load(Plate_PinpointConstraint(gp_XY(0.,1.),gp_XYZ(0.,1.,-0.5)));
 GA_TRY(p.SolveTI(3,1.));ga_plate_report(p,{0.3,0.4});
}
void ga_plate_gtoc(const Plate_GtoCConstraint& c){
 ga_int(c.nb_PPC());for(int i=0;i<9;i++)ga_plate_print_ppc(c.GetPPC(i));
 ga_xyz(c.D1SurfInit().DU());ga_xyz(c.D1SurfInit().DV());ga_plate_solve_with(c);
}
void ga_plate_free(const Plate_FreeGtoCConstraint& c){
 ga_int(c.nb_PPC());for(int i=0;i<5;i++)ga_plate_print_ppc(c.GetPPC(i));
 ga_int(c.nb_LSC());for(int i=0;i<c.nb_LSC();i++)ga_plate_print_lsc(c.LSC(i));ga_plate_solve_with(c);
}
// `kind args...`: 0 line, 1 plane, 2 global translation, 3 sampled curve, 4..9 GtoC
// (G1, G1 nP, G2, G2 nP, G3, G3 nP), 10..12 FreeGtoC (G1, G2, G3).
void ga_plate_constraint_case(std::istream& input){
 GaReader r{input};int kind=r.integer();double u=r.real(),v=r.real();gp_XY uv(u,v);
 GA_TRY(switch(kind){
  case 0:{gp_Pnt o=r.pnt();gp_Dir d=r.dir();int iu=r.integer(),iv=r.integer();Plate_LineConstraint c(uv,gp_Lin(o,d),iu,iv);ga_plate_print_lsc(c.LSC());ga_plate_solve_with(c);}break;
  case 1:{gp_Pnt o=r.pnt();gp_Dir d=r.dir();int iu=r.integer(),iv=r.integer();Plate_PlaneConstraint c(uv,gp_Pln(o,d),iu,iv);ga_plate_print_lsc(c.LSC());ga_plate_solve_with(c);}break;
  case 2:{std::vector<double> xy=r.reals();NCollection_Sequence<gp_XY> s;for(size_t i=0;i+1<xy.size();i+=2)s.Append(gp_XY(xy[i],xy[i+1]));
   Plate_GlobalTranslationConstraint c(s);ga_plate_print_lxyz(c.LXYZC());ga_plate_solve_with(c);}break;
  case 3:{int m=r.integer();NCollection_Sequence<Plate_PinpointConstraint> s;for(int i=0;i<m;i++)s.Append(ga_plate_ppc(r));int n=r.integer();
   Plate_SampledCurveConstraint c(s,n);ga_plate_print_lxyz(c.LXYZC());ga_plate_solve_with(c);}break;
  case 4:case 5:{Plate_D1 s1=ga_plate_d1(r),t1=ga_plate_d1(r);gp_XYZ np=r.xyz();
   ga_plate_gtoc(kind==4?Plate_GtoCConstraint(uv,s1,t1):Plate_GtoCConstraint(uv,s1,t1,np));}break;
  case 6:case 7:{Plate_D1 s1=ga_plate_d1(r),t1=ga_plate_d1(r);Plate_D2 s2=ga_plate_d2(r),t2=ga_plate_d2(r);gp_XYZ np=r.xyz();
   ga_plate_gtoc(kind==6?Plate_GtoCConstraint(uv,s1,t1,s2,t2):Plate_GtoCConstraint(uv,s1,t1,s2,t2,np));}break;
  case 8:case 9:{Plate_D1 s1=ga_plate_d1(r),t1=ga_plate_d1(r);Plate_D2 s2=ga_plate_d2(r),t2=ga_plate_d2(r);Plate_D3 s3=ga_plate_d3(r),t3=ga_plate_d3(r);gp_XYZ np=r.xyz();
   ga_plate_gtoc(kind==8?Plate_GtoCConstraint(uv,s1,t1,s2,t2,s3,t3):Plate_GtoCConstraint(uv,s1,t1,s2,t2,s3,t3,np));}break;
  case 10:{Plate_D1 s1=ga_plate_d1(r),t1=ga_plate_d1(r);double load=r.real();int o=r.integer();ga_plate_free(Plate_FreeGtoCConstraint(uv,s1,t1,load,o));}break;
  case 11:{Plate_D1 s1=ga_plate_d1(r),t1=ga_plate_d1(r);Plate_D2 s2=ga_plate_d2(r),t2=ga_plate_d2(r);double load=r.real();int o=r.integer();
   ga_plate_free(Plate_FreeGtoCConstraint(uv,s1,t1,s2,t2,load,o));}break;
  default:{Plate_D1 s1=ga_plate_d1(r),t1=ga_plate_d1(r);Plate_D2 s2=ga_plate_d2(r),t2=ga_plate_d2(r);Plate_D3 s3=ga_plate_d3(r),t3=ga_plate_d3(r);double load=r.real();int o=r.integer();
   ga_plate_free(Plate_FreeGtoCConstraint(uv,s1,t1,s2,t2,s3,t3,load,o));}break;
 });
 ga_end();
}
