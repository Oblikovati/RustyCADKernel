// GccAna probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/gcc_ana.rs and gcc_ana_lines.rs.
// Every solver request is `kind detail arguments... tolerance`: `kind` selects the
// constructor overload, `detail` the consultations printed per solution (bit 1
// qualifiers, bit 2 first argument, bit 4 second, bit 8 third or centre). A
// tangency is read only when IsTheSame is false, so no request reads a slot that
// native leaves uninitialized; the corpus clears the bits of the few other such
// slots (tools/tkgeomalgo_corpus_gccana.py).
#include <GccAna_Circ2d2TanOn.hxx>
#include <GccAna_Circ2d2TanRad.hxx>
#include <GccAna_Circ2d3Tan.hxx>
#include <GccAna_Circ2dBisec.hxx>
#include <GccAna_Circ2dTanCen.hxx>
#include <GccAna_Circ2dTanOnRad.hxx>
#include <GccAna_CircLin2dBisec.hxx>
#include <GccAna_CircPnt2dBisec.hxx>
#include <GccAna_Lin2d2Tan.hxx>
#include <GccAna_Lin2dBisec.hxx>
#include <GccAna_Lin2dTanObl.hxx>
#include <GccAna_LinPnt2dBisec.hxx>
#include <GccAna_Pnt2dBisec.hxx>
// `position x y dx dy radius` and `position x y dx dy`.
GccEnt_QualifiedCirc ga_read_qcirc(GaReader& r){GccEnt_Position p=GccEnt_Position(r.integer());return GccEnt_QualifiedCirc(ga_read_circ2d(r),p);}
GccEnt_QualifiedLin ga_read_qlin(GaReader& r){GccEnt_Position p=GccEnt_Position(r.integer());return GccEnt_QualifiedLin(ga_read_lin2d(r),p);}
// One tangency (ParSol, ParArg, PntSol).
void ga_tangency(double s,double a,const gp_Pnt2d& p){ga_real(s);ga_real(a);ga_pnt2d(p);}
#define GA_TANGENCY(CALL) GA_TRY(double s_=0.,a_=0.;gp_Pnt2d p_;CALL(i,s_,a_,p_);ga_tangency(s_,a_,p_))
// A tangency guarded by IsTheSame (printed first).
#define GA_SAME_TANGENCY(SAME,CALL) {bool same_=true;GA_TRY(same_=SAME(i);ga_flag(same_));if(!same_)GA_TANGENCY(CALL);}
// The per-solution loop from index 0 to NbSolutions + 1.
#define GA_SOLUTIONS(S,...) ga_flag(S.IsDone());ga_int(S.NbSolutions());for(int i=0;i<=S.NbSolutions()+1;i++){__VA_ARGS__}
void ga_circ3_report(const GccAna_Circ2d3Tan& s,int detail){
 GA_SOLUTIONS(s,
  GA_TRY(ga_circ2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a,b,c;s.WhichQualifier(i,a,b,c);ga_int(a);ga_int(b);ga_int(c));
  if(detail&2)GA_SAME_TANGENCY(s.IsTheSame1,s.Tangency1);
  if(detail&4)GA_SAME_TANGENCY(s.IsTheSame2,s.Tangency2);
  if(detail&8)GA_SAME_TANGENCY(s.IsTheSame3,s.Tangency3);)
}
// kind: 0 ccc, 1 ccl, 2 cll, 3 lll, 4 ccp, 5 clp, 6 llp, 7 cpp, 8 lpp, 9 ppp.
void ga_gccana_circ3tan_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(
  if(kind==0){auto a=ga_read_qcirc(r);auto b=ga_read_qcirc(r);auto c=ga_read_qcirc(r);ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else if(kind==1){auto a=ga_read_qcirc(r);auto b=ga_read_qcirc(r);auto c=ga_read_qlin(r);ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else if(kind==2){auto a=ga_read_qcirc(r);auto b=ga_read_qlin(r);auto c=ga_read_qlin(r);ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else if(kind==3){auto a=ga_read_qlin(r);auto b=ga_read_qlin(r);auto c=ga_read_qlin(r);ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else if(kind==4){auto a=ga_read_qcirc(r);auto b=ga_read_qcirc(r);gp_Pnt2d c=r.pnt2d();ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else if(kind==5){auto a=ga_read_qcirc(r);auto b=ga_read_qlin(r);gp_Pnt2d c=r.pnt2d();ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else if(kind==6){auto a=ga_read_qlin(r);auto b=ga_read_qlin(r);gp_Pnt2d c=r.pnt2d();ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else if(kind==7){auto a=ga_read_qcirc(r);gp_Pnt2d b=r.pnt2d(),c=r.pnt2d();ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else if(kind==8){auto a=ga_read_qlin(r);gp_Pnt2d b=r.pnt2d(),c=r.pnt2d();ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);}
  else{gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();ga_circ3_report(GccAna_Circ2d3Tan(a,b,c,r.real()),detail);})
 ga_end();
}
void ga_2tanrad_report(const GccAna_Circ2d2TanRad& s,int detail){
 GA_SOLUTIONS(s,
  GA_TRY(ga_circ2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a,b;s.WhichQualifier(i,a,b);ga_int(a);ga_int(b));
  if(detail&2)GA_SAME_TANGENCY(s.IsTheSame1,s.Tangency1);
  if(detail&4)GA_SAME_TANGENCY(s.IsTheSame2,s.Tangency2);)
}
// kind: 0 cc, 1 cl, 2 cp, 3 lp, 4 ll, 5 pp; then the radius and the tolerance.
void ga_gccana_2tanrad_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(
  if(kind==0){auto a=ga_read_qcirc(r);auto b=ga_read_qcirc(r);double rad=r.real();ga_2tanrad_report(GccAna_Circ2d2TanRad(a,b,rad,r.real()),detail);}
  else if(kind==1){auto a=ga_read_qcirc(r);auto b=ga_read_qlin(r);double rad=r.real();ga_2tanrad_report(GccAna_Circ2d2TanRad(a,b,rad,r.real()),detail);}
  else if(kind==2){auto a=ga_read_qcirc(r);gp_Pnt2d b=r.pnt2d();double rad=r.real();ga_2tanrad_report(GccAna_Circ2d2TanRad(a,b,rad,r.real()),detail);}
  else if(kind==3){auto a=ga_read_qlin(r);gp_Pnt2d b=r.pnt2d();double rad=r.real();ga_2tanrad_report(GccAna_Circ2d2TanRad(a,b,rad,r.real()),detail);}
  else if(kind==4){auto a=ga_read_qlin(r);auto b=ga_read_qlin(r);double rad=r.real();ga_2tanrad_report(GccAna_Circ2d2TanRad(a,b,rad,r.real()),detail);}
  else{gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();double rad=r.real();ga_2tanrad_report(GccAna_Circ2d2TanRad(a,b,rad,r.real()),detail);})
 ga_end();
}
void ga_2tanon_report(const GccAna_Circ2d2TanOn& s,int detail){
 GA_SOLUTIONS(s,
  GA_TRY(ga_circ2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a,b;s.WhichQualifier(i,a,b);ga_int(a);ga_int(b));
  if(detail&2)GA_SAME_TANGENCY(s.IsTheSame1,s.Tangency1);
  if(detail&4)GA_SAME_TANGENCY(s.IsTheSame2,s.Tangency2);
  if(detail&8)GA_TRY(double p=0.;gp_Pnt2d q;s.CenterOn3(i,p,q);ga_real(p);ga_pnt2d(q));)
}
// Two arguments then the locus: `on` 0 line, 1 circle; `args` 0 cc, 1 cl, 2 ll, 3 cp,
// 4 lp, 5 pp (kind = 6 * on + args).
void ga_gccana_2tanon_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();int on=kind/6,args=kind%6;
 GA_TRY(
  auto run=[&](auto a,auto b){if(on==0){gp_Lin2d l=ga_read_lin2d(r);ga_2tanon_report(GccAna_Circ2d2TanOn(a,b,l,r.real()),detail);}
                             else{gp_Circ2d c=ga_read_circ2d(r);ga_2tanon_report(GccAna_Circ2d2TanOn(a,b,c,r.real()),detail);}};
  if(args==0){auto a=ga_read_qcirc(r);auto b=ga_read_qcirc(r);run(a,b);}
  else if(args==1){auto a=ga_read_qcirc(r);auto b=ga_read_qlin(r);run(a,b);}
  else if(args==2){auto a=ga_read_qlin(r);auto b=ga_read_qlin(r);run(a,b);}
  else if(args==3){auto a=ga_read_qcirc(r);gp_Pnt2d b=r.pnt2d();run(a,b);}
  else if(args==4){auto a=ga_read_qlin(r);gp_Pnt2d b=r.pnt2d();run(a,b);}
  else{gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();run(a,b);})
 ga_end();
}
void ga_tanonrad_report(const GccAna_Circ2dTanOnRad& s,int detail){
 GA_SOLUTIONS(s,
  GA_TRY(ga_circ2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a;s.WhichQualifier(i,a);ga_int(a));
  if(detail&2)GA_SAME_TANGENCY(s.IsTheSame1,s.Tangency1);
  if(detail&8)GA_TRY(double p=0.;gp_Pnt2d q;s.CenterOn3(i,p,q);ga_real(p);ga_pnt2d(q));)
}
// `on` 0 line, 1 circle; `arg` 0 circle, 1 line, 2 point (kind = 3 * on + arg).
void ga_gccana_tanonrad_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();int on=kind/3,arg=kind%3;
 GA_TRY(
  auto run=[&](auto a){if(on==0){gp_Lin2d l=ga_read_lin2d(r);double rad=r.real();ga_tanonrad_report(GccAna_Circ2dTanOnRad(a,l,rad,r.real()),detail);}
                       else{gp_Circ2d c=ga_read_circ2d(r);double rad=r.real();ga_tanonrad_report(GccAna_Circ2dTanOnRad(a,c,rad,r.real()),detail);}};
  if(arg==0)run(ga_read_qcirc(r));else if(arg==1)run(ga_read_qlin(r));else run(r.pnt2d());)
 ga_end();
}
// kind 0 `qcirc centre tol`, 1 `line centre`, 2 `point centre`.
void ga_gccana_tancen_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 auto report=[&](const GccAna_Circ2dTanCen& s){GA_SOLUTIONS(s,
  GA_TRY(ga_circ2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a;s.WhichQualifier(i,a);ga_int(a));
  if(detail&2)GA_SAME_TANGENCY(s.IsTheSame1,s.Tangency1);)};
 GA_TRY(
  if(kind==0){auto a=ga_read_qcirc(r);gp_Pnt2d c=r.pnt2d();report(GccAna_Circ2dTanCen(a,c,r.real()));}
  else if(kind==1){gp_Lin2d l=ga_read_lin2d(r);report(GccAna_Circ2dTanCen(l,r.pnt2d()));}
  else{gp_Pnt2d p=r.pnt2d();report(GccAna_Circ2dTanCen(p,r.pnt2d()));})
 ga_end();
}
// kind 0 `p1 p2 tol`, 1 `qcirc point tol`, 2 `qcirc qcirc tol`.
void ga_gccana_lin2tan_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 auto report=[&](const GccAna_Lin2d2Tan& s){GA_SOLUTIONS(s,
  GA_TRY(ga_lin2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a,b;s.WhichQualifier(i,a,b);ga_int(a);ga_int(b));
  if(detail&2)GA_TANGENCY(s.Tangency1);
  if(detail&4)GA_TANGENCY(s.Tangency2);)};
 GA_TRY(
  if(kind==0){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();report(GccAna_Lin2d2Tan(a,b,r.real()));}
  else if(kind==1){auto a=ga_read_qcirc(r);gp_Pnt2d b=r.pnt2d();report(GccAna_Lin2d2Tan(a,b,r.real()));}
  else{auto a=ga_read_qcirc(r);auto b=ga_read_qcirc(r);report(GccAna_Lin2d2Tan(a,b,r.real()));})
 ga_end();
}
// kind 0 `point line angle`, 1 `qcirc line angle`.
void ga_gccana_lintanobl_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 auto report=[&](const GccAna_Lin2dTanObl& s){ga_flag(s.IsDone());int n=-1;GA_TRY(n=s.NbSolutions();ga_int(n));
  for(int i=0;i<=n+1;i++){GA_TRY(ga_lin2d(s.ThisSolution(i)));
   if(detail&1)GA_TRY(GccEnt_Position a;s.WhichQualifier(i,a);ga_int(a));
   if(detail&2)GA_TANGENCY(s.Tangency1);
   if(detail&4)GA_TANGENCY(s.Intersection2);}};
 GA_TRY(
  if(kind==0){gp_Pnt2d p=r.pnt2d();gp_Lin2d l=ga_read_lin2d(r);report(GccAna_Lin2dTanObl(p,l,r.real()));}
  else{auto a=ga_read_qcirc(r);gp_Lin2d l=ga_read_lin2d(r);report(GccAna_Lin2dTanObl(a,l,r.real()));})
 ga_end();
}
// `line line`: the bisectors and their intersections with both lines.
void ga_gccana_linbisec_case(std::istream& input){
 GaReader r{input};gp_Lin2d a=ga_read_lin2d(r),b=ga_read_lin2d(r);
 GA_TRY(GccAna_Lin2dBisec s(a,b);ga_flag(s.IsDone());int n=-1;GA_TRY(n=s.NbSolutions();ga_int(n));
  for(int i=0;i<=n+1;i++){GA_TRY(ga_lin2d(s.ThisSolution(i)));GA_TANGENCY(s.Intersection1);GA_TANGENCY(s.Intersection2);})
 ga_end();
}
// A bisecting locus: its accessors, or `null` for a null handle.
void ga_bisec_report(const occ::handle<GccInt_Bisec>& b){if(b.IsNull()){std::cout<<"null ";return;}ga_gccint_accessors(b);}
// kind 0 `circ circ`, 1 `circ line`, 2 `circ point`, 3 `circ point tol`, 4 `line point`,
// 5 `point point`.
void ga_gccana_bisec_case(std::istream& input){
 GaReader r{input};int kind=r.integer();
 GA_TRY(
  if(kind==0){gp_Circ2d a=ga_read_circ2d(r),b=ga_read_circ2d(r);GccAna_Circ2dBisec s(a,b);ga_flag(s.IsDone());int n=-1;GA_TRY(n=s.NbSolutions();ga_int(n));
   for(int i=0;i<=n+1;i++)GA_TRY(ga_bisec_report(s.ThisSolution(i)));}
  else if(kind==1){gp_Circ2d a=ga_read_circ2d(r);gp_Lin2d b=ga_read_lin2d(r);GccAna_CircLin2dBisec s(a,b);ga_flag(s.IsDone());ga_int(s.NbSolutions());
   for(int i=0;i<=s.NbSolutions()+1;i++)GA_TRY(ga_bisec_report(s.ThisSolution(i)));}
  else if(kind==2||kind==3){gp_Circ2d a=ga_read_circ2d(r);gp_Pnt2d b=r.pnt2d();
   GccAna_CircPnt2dBisec s=kind==2?GccAna_CircPnt2dBisec(a,b):GccAna_CircPnt2dBisec(a,b,r.real());ga_flag(s.IsDone());ga_int(s.NbSolutions());
   for(int i=0;i<=s.NbSolutions()+1;i++)GA_TRY(ga_bisec_report(s.ThisSolution(i)));}
  else if(kind==4){gp_Lin2d a=ga_read_lin2d(r);GccAna_LinPnt2dBisec s(a,r.pnt2d());ga_flag(s.IsDone());GA_TRY(ga_bisec_report(s.ThisSolution()));}
  else{gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();GccAna_Pnt2dBisec s(a,b);ga_flag(s.IsDone());ga_flag(s.HasSolution());GA_TRY(ga_lin2d(s.ThisSolution()));})
 ga_end();
}
