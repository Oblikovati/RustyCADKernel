// AppCont requests (tools/tkgeombase_corpus_appcont.py): the raw tables, the matrix
// providers and the continuous least-squares fit of an analytic function. The Rust
// twin is crates/tkgeombase/examples/gb_probe/app_cont.rs.

#include <AppCont_ContMatrices.pxx>
#include <AppCont_LeastSquare.hxx>
#include <cmath>
// `acc_table kind start count`: the raw entries of table kind (0 BB, 1 InvM, 2 IBP,
// 3 IBT, 4 VB), read directly from the static arrays.
void gb_acc_table_case(std::istream& in){
 GbReader r{in};const int kind=r.integer(),start=r.integer(),count=r.integer();
 const double* tables[]={AppCont_ContMatrices::THE_BBMATRIX_DATA,AppCont_ContMatrices::THE_INV_MMATRIX_DATA,
  AppCont_ContMatrices::THE_IBPMATRIX_DATA,AppCont_ContMatrices::THE_IBTMATRIX_DATA,AppCont_ContMatrices::THE_VBMATRIX_DATA};
 for(int i=start;i<start+count;++i)gb_real(tables[kind][i]);
 gb_end();
}
// `acc_offsets classe`: MMatrixOffset, IBPMatrixOffset, IBTMatrixOffset.
void gb_acc_offsets_case(std::istream& in){
 GbReader r{in};const int classe=r.integer();
 gb_int(AppCont_ContMatrices::MMatrixOffset(classe));gb_int(AppCont_ContMatrices::IBPMatrixOffset(classe));
 gb_int(AppCont_ContMatrices::IBTMatrixOffset(classe));gb_end();
}
// `acc_matrix kind classe nbpoints rl cl rows cols`: one provider (0 MMatrix,
// 1 InvMMatrix, 2 IBPMatrix, 3 IBTMatrix, 4 VBernstein) into a matrix of `rows x cols`
// from (rl, cl) filled with -7; the exception, then the matrix (partial writes kept).
void gb_acc_matrix_case(std::istream& in){
 GbReader r{in};const int kind=r.integer(),classe=r.integer(),nbpoints=r.integer();
 const int rl=r.integer(),cl=r.integer(),rows=r.integer(),cols=r.integer();
 math_Matrix m(rl,rl+rows-1,cl,cl+cols-1,-7.);
 GB_TRY(
  if(kind==0)AppCont_ContMatrices::MMatrix(classe,m);
  else if(kind==1)AppCont_ContMatrices::InvMMatrix(classe,m);
  else if(kind==2)AppCont_ContMatrices::IBPMatrix(classe,m);
  else if(kind==3)AppCont_ContMatrices::IBTMatrix(classe,m);
  else AppCont_ContMatrices::VBernstein(classe,nbpoints,m));
 gb_fem_matrix(m);gb_end();
}
// An analytic function: coordinate d is `a0 + a1 u + a2 u^2 + a3 sin(a4 u)` (plus
// jump0 at u == U0 and jump1 at u == U1), reduced into [0, P) when periodic of P.
class GbAccFunction : public AppCont_Function{
public:
 double myU0=0.,myU1=1.,myJump0=0.,myJump1=0.;bool myD1Ok=true;
 std::vector<double> myCoef,myPeriods;
 double FirstParameter() const override{return myU0;}
 double LastParameter() const override{return myU1;}
 double Coord(int d,double u) const{
  const double* a=&myCoef[5*(d-1)];double v=a[0]+a[1]*u+a[2]*u*u+a[3]*std::sin(a[4]*u);
  if(u==myU0)v+=myJump0;
  if(u==myU1)v+=myJump1;
  const double p=myPeriods[d-1];
  if(p>0.)v-=p*std::floor(v/p);
  return v;
 }
 double Deriv(int d,double u) const{const double* a=&myCoef[5*(d-1)];return a[1]+2.*a[2]*u+a[3]*a[4]*std::cos(a[4]*u);}
 bool Value(const double u,NCollection_Array1<gp_Pnt2d>& p2d,NCollection_Array1<gp_Pnt>& p) const override{
  int d=1;
  for(int j=1;j<=myNbPnt;++j,d+=3)p(j)=gp_Pnt(Coord(d,u),Coord(d+1,u),Coord(d+2,u));
  for(int j=1;j<=myNbPnt2d;++j,d+=2)p2d(j)=gp_Pnt2d(Coord(d,u),Coord(d+1,u));
  return true;
 }
 bool D1(const double u,NCollection_Array1<gp_Vec2d>& v2d,NCollection_Array1<gp_Vec>& v) const override{
  int d=1;
  for(int j=1;j<=myNbPnt;++j,d+=3)v(j)=gp_Vec(Deriv(d,u),Deriv(d+1,u),Deriv(d+2,u));
  for(int j=1;j<=myNbPnt2d;++j,d+=2)v2d(j)=gp_Vec2d(Deriv(d,u),Deriv(d+1,u));
  return myD1Ok;
 }
 void PeriodInformation(const int d,bool& isPeriodic,double& period) const override{
  period=myPeriods[d-1];isPeriodic=period>0.;
 }
 // `u0 u1 nb3d nb2d d1ok jump0 jump1 nperiodic (dim period)... coefficients...`.
 explicit GbAccFunction(GbReader& r){
  myU0=r.real();myU1=r.real();myNbPnt=r.integer();myNbPnt2d=r.integer();
  myD1Ok=r.flag();myJump0=r.real();myJump1=r.real();
  const int dims=std::max(3*myNbPnt+2*myNbPnt2d,0);myPeriods.assign(dims,0.);
  for(int n=r.integer();n>0;--n){const int d=r.integer();myPeriods[d-1]=r.real();}
  myCoef.resize(5*dims);for(double& c:myCoef)c=r.real();
 }
};
// `acc_ls function fc lc deg nbpoints`: the counts, then IsDone, Error, Value.
void gb_acc_ls_case(std::istream& in){
 GbReader r{in};GbAccFunction f(r);
 const AppParCurves_Constraint fc=AppParCurves_Constraint(r.integer()),lc=AppParCurves_Constraint(r.integer());
 const int deg=r.integer(),nbpoints=r.integer();
 int nb3d,nb2d;f.GetNumberOfPoints(nb3d,nb2d);gb_int(nb3d);gb_int(nb2d);
 gb_int(f.GetNbOf3dPoints());gb_int(f.GetNbOf2dPoints());
 GB_TRY(
  AppCont_LeastSquare ls(f,f.FirstParameter(),f.LastParameter(),fc,lc,deg,nbpoints);
  gb_flag(ls.IsDone());double e,e3,e2;ls.Error(e,e3,e2);gb_real(e);gb_real(e3);gb_real(e2);
  gb_apc_curve(ls.Value()));
 gb_end();
}
