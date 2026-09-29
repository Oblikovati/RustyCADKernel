// AppBlend probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/app_blend.rs. The template AppBlend_AppSurf.gxx
// is instantiated here on a probe line and section generator (GaAbLine,
// GaAbSections), exactly as GeomFill_AppSurf_0.cxx instantiates it, so the probe needs
// no other package.
#include <AppBlend_Approx.hxx>
#include <Approx_ParametrizationType.hxx>
#include <GeomAbs_Shape.hxx>
#include <NCollection_HArray2.hxx>
#include <NCollection_Sequence.hxx>
#include <Standard_Transient.hxx>
Standard_EXPORT void AppBlend_SetContextApproxWithNoTgt(const bool b);
Standard_EXPORT bool AppBlend_GetContextApproxWithNoTgt();
Standard_EXPORT void AppBlend_SetContextSplineApprox(const bool b);
Standard_EXPORT bool AppBlend_GetContextSplineApprox();
// `n` line points: Point(i) is the section index i.
class GaAbLine : public Standard_Transient
{
public:
  explicit GaAbLine(int n) : myN(n) {}
  int NbPoints() const { return myN; }
  int Point(int i) const { return i; }
private:
  int myN;
};
// Rational quadratic sections swept along t = (i - 1) / (n - 1) * len: poles
// (x_j + a t, y_j (1 + b t), c t^2 + z_j), weights (1, 0.7 + d t, 1), and `np2d` 2d
// curves (t, k t^2); the derivatives are reported when `deriv`.
class GaAbSections
{
public:
  int n, np2d;double len,a,b,c,d;bool deriv;
  double T(int i) const {return double(i-1)/double(n-1)*len;}
  void GetShape(int& NbPoles,int& NbKnots,int& Degree,int& NbPoles2d) const {NbPoles=3;NbKnots=2;Degree=2;NbPoles2d=np2d;}
  void Knots(NCollection_Array1<double>& K) const {K(K.Lower())=0.;K(K.Lower()+1)=1.;}
  void Mults(NCollection_Array1<int>& M) const {M(M.Lower())=3;M(M.Lower()+1)=3;}
  void Fill(int i,NCollection_Array1<gp_Pnt>& P,NCollection_Array1<gp_Pnt2d>& P2d,NCollection_Array1<double>& W) const {
   const double base[3][3]={{0.,0.,0.},{1.,1.,0.2},{2.,0.,0.}};double t=T(i);
   for(int j=0;j<3;j++)P(j+1)=gp_Pnt(base[j][0]+a*t,base[j][1]*(1.+b*t),c*t*t+base[j][2]);
   W(1)=1.;W(2)=0.7+d*t;W(3)=1.;
   for(int k=1;k<=np2d;k++)P2d(k)=gp_Pnt2d(t,k*t*t);
  }
  bool Section(int i,NCollection_Array1<gp_Pnt>& P,NCollection_Array1<gp_Vec>& DP,NCollection_Array1<gp_Pnt2d>& P2d,
               NCollection_Array1<gp_Vec2d>& DP2d,NCollection_Array1<double>& W,NCollection_Array1<double>& DW) const {
   Fill(i,P,P2d,W);const double base[3]={0.,1.,0.};double t=T(i);
   for(int j=0;j<3;j++)DP(j+1)=gp_Vec(a,base[j]*b,2.*c*t);
   DW(1)=0.;DW(2)=d;DW(3)=0.;
   for(int k=1;k<=np2d;k++)DP2d(k)=gp_Vec2d(1.,2.*k*t);
   return deriv;
  }
  void Section(int i,NCollection_Array1<gp_Pnt>& P,NCollection_Array1<gp_Pnt2d>& P2d,NCollection_Array1<double>& W) const {Fill(i,P,P2d,W);}
  double Parameter(int i) const {return T(i);}
};
// The class GeomFill_AppSurf.hxx declares, on the probe types.
class GaAbAppSurf : public AppBlend_Approx
{
public:
  DEFINE_STANDARD_ALLOC
  GaAbAppSurf();
  GaAbAppSurf(const int Degmin,const int Degmax,const double Tol3d,const double Tol2d,const int NbIt,const bool KnownParameters=false);
  void Init(const int Degmin,const int Degmax,const double Tol3d,const double Tol2d,const int NbIt,const bool KnownParameters=false);
  void SetParType(const Approx_ParametrizationType ParType);
  void SetContinuity(const GeomAbs_Shape C);
  void SetCriteriumWeight(const double W1,const double W2,const double W3);
  Approx_ParametrizationType ParType() const;
  GeomAbs_Shape Continuity() const;
  void CriteriumWeight(double& W1,double& W2,double& W3) const;
  void Perform(const occ::handle<GaAbLine>& Lin,GaAbSections& SecGen,const bool SpApprox=false);
  void PerformSmoothing(const occ::handle<GaAbLine>& Lin,GaAbSections& SecGen);
  void Perform(const occ::handle<GaAbLine>& Lin,GaAbSections& SecGen,const int NbMaxP);
  bool IsDone() const override;
  void SurfShape(int& UDegree,int& VDegree,int& NbUPoles,int& NbVPoles,int& NbUKnots,int& NbVKnots) const override;
  void Surface(NCollection_Array2<gp_Pnt>& TPoles,NCollection_Array2<double>& TWeights,NCollection_Array1<double>& TUKnots,
               NCollection_Array1<double>& TVKnots,NCollection_Array1<int>& TUMults,NCollection_Array1<int>& TVMults) const override;
  int UDegree() const override;
  int VDegree() const override;
  const NCollection_Array2<gp_Pnt>& SurfPoles() const override;
  const NCollection_Array2<double>& SurfWeights() const override;
  const NCollection_Array1<double>& SurfUKnots() const override;
  const NCollection_Array1<double>& SurfVKnots() const override;
  const NCollection_Array1<int>& SurfUMults() const override;
  const NCollection_Array1<int>& SurfVMults() const override;
  int NbCurves2d() const override;
  void Curves2dShape(int& Degree,int& NbPoles,int& NbKnots) const override;
  void Curve2d(const int Index,NCollection_Array1<gp_Pnt2d>& TPoles,NCollection_Array1<double>& TKnots,NCollection_Array1<int>& TMults) const override;
  int Curves2dDegree() const override;
  const NCollection_Array1<gp_Pnt2d>& Curve2dPoles(const int Index) const override;
  const NCollection_Array1<double>& Curves2dKnots() const override;
  const NCollection_Array1<int>& Curves2dMults() const override;
  void TolReached(double& Tol3d,double& Tol2d) const override;
  double TolCurveOnSurf(const int Index) const override;
private:
  void InternalPerform(const occ::handle<GaAbLine>& Lin,GaAbSections& SecGen,const bool SpApprox,const bool UseVariational);
  bool done;int dmin;int dmax;double tol3d;double tol2d;int nbit;int udeg;int vdeg;bool knownp;
  occ::handle<NCollection_HArray2<gp_Pnt>> tabPoles;occ::handle<NCollection_HArray2<double>> tabWeights;
  occ::handle<NCollection_HArray1<double>> tabUKnots;occ::handle<NCollection_HArray1<double>> tabVKnots;
  occ::handle<NCollection_HArray1<int>> tabUMults;occ::handle<NCollection_HArray1<int>> tabVMults;
  NCollection_Sequence<occ::handle<NCollection_HArray1<gp_Pnt2d>>> seqPoles2d;
  double tol3dreached;double tol2dreached;Approx_ParametrizationType paramtype;GeomAbs_Shape continuity;double critweights[3];
};
#define TheSectionGenerator GaAbSections
#define Handle_TheLine occ::handle<GaAbLine>
#define TheLine GaAbLine
#define AppBlend_AppSurf GaAbAppSurf
#include <AppBlend_AppSurf.lxx>
#include <AppBlend_AppSurf.gxx>
#undef TheSectionGenerator
#undef Handle_TheLine
#undef TheLine
#undef AppBlend_AppSurf
void ga_ab_report(const GaAbAppSurf& s){
 ga_flag(s.IsDone());
 GA_TRY(int ud,vd,nu,nv,nku,nkv;s.SurfShape(ud,vd,nu,nv,nku,nkv);ga_int(ud);ga_int(vd);ga_int(nu);ga_int(nv);ga_int(nku);ga_int(nkv));
 GA_TRY(const NCollection_Array2<gp_Pnt>& p=s.SurfPoles();const NCollection_Array2<double>& w=s.SurfWeights();
  for(int i=p.LowerRow();i<=p.UpperRow();i++)for(int j=p.LowerCol();j<=p.UpperCol();j++){ga_pnt(p(i,j));ga_real(w(i,j));});
 GA_TRY(const NCollection_Array1<double>& k=s.SurfVKnots();ga_int(k.Lower());for(double x:k)ga_real(x);for(int m:s.SurfVMults())ga_int(m));
 GA_TRY(for(double x:s.SurfUKnots())ga_real(x);for(int m:s.SurfUMults())ga_int(m));
 GA_TRY(double t3,t2;s.TolReached(t3,t2);ga_real(t3);ga_real(t2));ga_real(s.TolCurveOnSurf(1));
 GA_TRY(ga_int(s.NbCurves2d()));GA_TRY(int d,np,nk;s.Curves2dShape(d,np,nk);ga_int(d);ga_int(np);ga_int(nk));
 GA_TRY(ga_int(s.Curves2dDegree()));
 for(int i=1;i<=2;i++)GA_TRY(for(const gp_Pnt2d& p:s.Curve2dPoles(i))ga_pnt2d(p));
 GA_TRY(NCollection_Array1<gp_Pnt2d> p(1,1);NCollection_Array1<double> k(1,1);NCollection_Array1<int> m(1,1);s.Curve2d(1,p,k,m);
  ga_int(p.Length());ga_int(k.Length());ga_int(m.Length()));
 GA_TRY(NCollection_Array2<gp_Pnt> p(1,1,1,1);NCollection_Array2<double> w(1,1,1,1);NCollection_Array1<double> uk(1,1),vk(1,1);
  NCollection_Array1<int> um(1,1),vm(1,1);s.Surface(p,w,uk,vk,um,vm);ga_int(p.ColLength());ga_int(p.RowLength());ga_int(vk.Length()));
}
// `n len a b c d np2d deriv mode dmin dmax tol3d tol2d nbit knownp partype cont nbmaxp notgt w1 w2 w3`:
// mode 0 Perform, 1 Perform(SpApprox), 2 PerformSmoothing, 3 Perform(NbMaxP); then twice
// more to show the kept state.
void ga_ab_appsurf_case(std::istream& input){
 GaReader r{input};GaAbSections f;f.n=r.integer();f.len=r.real();f.a=r.real();f.b=r.real();f.c=r.real();f.d=r.real();
 f.np2d=r.integer();f.deriv=r.flag();int mode=r.integer(),dmin=r.integer(),dmax=r.integer();double t3=r.real(),t2=r.real();
 int nbit=r.integer();bool knownp=r.flag();int partype=r.integer(),cont=r.integer(),nbmaxp=r.integer();bool notgt=r.flag();
 double w[3];for(double& x:w)x=r.real();
 occ::handle<GaAbLine> line=new GaAbLine(f.n);GaAbAppSurf s(dmin,dmax,t3,t2,nbit,knownp);
 s.SetParType(Approx_ParametrizationType(partype));s.SetContinuity(GeomAbs_Shape(cont));GA_TRY(s.SetCriteriumWeight(w[0],w[1],w[2]));
 double c1,c2,c3;s.CriteriumWeight(c1,c2,c3);ga_real(c1);ga_real(c2);ga_real(c3);ga_int(int(s.ParType()));ga_int(int(s.Continuity()));
 AppBlend_SetContextApproxWithNoTgt(notgt);
 for(int round=0;round<2;round++){
  GA_TRY(if(mode==0)s.Perform(line,f,false);else if(mode==1)s.Perform(line,f,true);else if(mode==2)s.PerformSmoothing(line,f);
   else s.Perform(line,f,nbmaxp));
  ga_ab_report(s);
 }
 AppBlend_SetContextApproxWithNoTgt(false);
 GA_TRY(s.Perform(occ::handle<GaAbLine>(),f,false));ga_flag(s.IsDone());
 ga_end();
}
// `b1 b2`: the two context switches set, read and reset.
void ga_ab_context_case(std::istream& input){
 GaReader r{input};bool b1=r.flag(),b2=r.flag();
 AppBlend_SetContextSplineApprox(b1);AppBlend_SetContextApproxWithNoTgt(b2);
 ga_flag(AppBlend_GetContextSplineApprox());ga_flag(AppBlend_GetContextApproxWithNoTgt());
 AppBlend_SetContextSplineApprox(false);AppBlend_SetContextApproxWithNoTgt(false);
 ga_flag(AppBlend_GetContextSplineApprox());ga_flag(AppBlend_GetContextApproxWithNoTgt());
 ga_end();
}
