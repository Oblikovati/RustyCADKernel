// TKG3d oracle: the AdvApprox package (tkg3d_probe.cxx). The evaluators below are
// defined identically in crates/tkg3d/examples/g3_probe/advapprox.rs.
// Coordinate c of function `kind`, derivative order d (0..2).
double g3_aa_value(int kind,int c,double t,int d){
 double s=c+1.0;
 if(kind==0){if(d==0)return ((s*t-2.0)*t+0.5)*t+c;if(d==1)return (3.0*s*t-4.0)*t+0.5;return 6.0*s*t-4.0;}
 if(kind==1){if(d==0)return std::sin(s*t)+c*std::cos(0.5*t);if(d==1)return s*std::cos(s*t)-0.5*c*std::sin(0.5*t);
  return -s*s*std::sin(s*t)-0.25*c*std::cos(0.5*t);}
 if(kind==2){double q=1.0+s*t*t;if(d==0)return 1.0/q;if(d==1)return -2.0*s*t/(q*q);return (6.0*s*s*t*t-2.0*s)/(q*q*q);}
 if(kind==3){double r=2.0+(c/3);int m=c%3;
  if(m==0){if(d==0)return r*std::cos(t);if(d==1)return -r*std::sin(t);return -r*std::cos(t);}
  if(m==1){if(d==0)return r*std::sin(t);if(d==1)return r*std::cos(t);return -r*std::sin(t);}
  if(d==0)return 0.3*t;if(d==1)return 0.3;return 0.0;}
 if(kind==4){double x=t-0.3,a=std::fabs(x);if(d==0)return s*std::pow(a,2.5);
  if(d==1)return s*2.5*std::pow(a,1.5)*(x<0?-1.0:1.0);return s*3.75*std::pow(a,0.5);}
 if(d==0)return std::exp(0.5*s*t);if(d==1)return 0.5*s*std::exp(0.5*s*t);return 0.25*s*s*std::exp(0.5*s*t);
}
// The probe evaluator: `kind`, and a failure mode: 0 none, 1 on derivative 2,
// 2 on the call number `failAt` (counting from 1).
class G3AdvEvaluator : public AdvApprox_EvaluatorFunction{
public:
 int kind,mode,failAt,calls=0;
 G3AdvEvaluator(int k,int m,int f):kind(k),mode(m),failAt(f){}
 void Evaluate(int* Dimension,double[2],double* Parameter,int* DerivativeRequest,double* Result,int* ErrorCode) override{
  ++calls;*ErrorCode=0;
  if((mode==1&&*DerivativeRequest==2)||(mode==2&&calls==failAt)){*ErrorCode=1;return;}
  for(int c=0;c<*Dimension;++c)Result[c]=g3_aa_value(kind,c,*Parameter,*DerivativeRequest);
 }
};
// Reads: kind mode failAt.
G3AdvEvaluator g3_aa_evaluator(G3Reader& r){int k=r.integer(),m=r.integer();return G3AdvEvaluator(k,m,r.integer());}
// Reads a cutting tool: 0 none (default), 1 DichoCutting, 2 PrefAndRec nrec rec... npref pref... weight.
std::unique_ptr<AdvApprox_Cutting> g3_aa_cutting(G3Reader& r){
 int kind=r.integer();
 if(kind==1)return std::make_unique<AdvApprox_DichoCutting>();
 if(kind!=2)return nullptr;
 std::vector<double> rec=r.reals(),pref=r.reals();double w=r.real();
 NCollection_Array1<double> ra(1,int(rec.size())),pa(1,int(pref.size()));
 for(size_t i=0;i<rec.size();++i)ra(int(i)+1)=rec[i];
 for(size_t i=0;i<pref.size();++i)pa(int(i)+1)=pref[i];
 return std::make_unique<AdvApprox_PrefAndRec>(ra,pa,w);
}
GeomAbs_Shape g3_aa_shape(int c){return c<=0?GeomAbs_C0:c==1?GeomAbs_C1:c==2?GeomAbs_C2:c==3?GeomAbs_C3:GeomAbs_G1;}
void g3_aa_result(const AdvApprox_ApproxAFunction& a){
 g3_flag(a.IsDone());g3_flag(a.HasResult());for(int d=0;d<=4;++d)G3_TRY(g3_int(a.NumSubSpaces(d)));g3_int(a.NbPoles());
 if(!a.HasResult()){std::cout<<"noresult ";return;}
 g3_int(a.Degree());g3_int(a.NbKnots());for(double k:a.Knots()->Array1())g3_real(k);for(int m:a.Multiplicities()->Array1())g3_int(m);
 if(!a.Poles1d().IsNull()){for(double v:a.Poles1d()->Array2())g3_real(v);
  for(int i=1;i<=a.NumSubSpaces(1);++i){NCollection_Array1<double> p(1,a.NbPoles());a.Poles1d(i,p);for(double v:p)g3_real(v);}}
 if(!a.Poles2d().IsNull()){for(const gp_Pnt2d& v:a.Poles2d()->Array2()){g3_real(v.X());g3_real(v.Y());}
  for(int i=1;i<=a.NumSubSpaces(2);++i){NCollection_Array1<gp_Pnt2d> p(1,a.NbPoles());a.Poles2d(i,p);for(const gp_Pnt2d& v:p){g3_real(v.X());g3_real(v.Y());}}}
 if(!a.Poles().IsNull()){for(const gp_Pnt& v:a.Poles()->Array2())g3_pnt(v);
  for(int i=1;i<=a.NumSubSpaces(3);++i){NCollection_Array1<gp_Pnt> p(1,a.NbPoles());a.Poles(i,p);for(const gp_Pnt& v:p)g3_pnt(v);}}
 for(int d=1;d<=3;++d){if(a.NumSubSpaces(d)==0){g3_flag(a.MaxError(d).IsNull());g3_flag(a.AverageError(d).IsNull());continue;}
  for(int i=1;i<=a.NumSubSpaces(d);++i){g3_real(a.MaxError(d,i));g3_real(a.AverageError(d,i));}
  for(double v:a.MaxError(d)->Array1())g3_real(v);for(double v:a.AverageError(d)->Array1())g3_real(v);}
 G3_TRY(a.MaxError(0));G3_TRY(a.AverageError(4));
 std::ostringstream s;a.Dump(s);std::string t=s.str();for(char& ch:t)if(ch=='\n')ch='|';std::cout<<'['<<t<<"] ";
}
// Reads n values of a tolerance table (null when n == 0).
occ::handle<NCollection_HArray1<double>> g3_aa_tolerances(G3Reader& r,int n){
 double t=r.real();if(n==0)return nullptr;occ::handle<NCollection_HArray1<double>> h=new NCollection_HArray1<double>(1,n);
 for(int i=1;i<=n;++i)h->SetValue(i,t*i);return h;
}
// g3_advapprox: evaluator, n1 n2 n3, tol1 tol2 tol3, first last continuity maxdeg maxseg, cutting.
void g3_advapprox_case(std::istream& in){
 G3Reader r{in};G3AdvEvaluator f=g3_aa_evaluator(r);int n1=r.integer(),n2=r.integer(),n3=r.integer();
 auto t1=g3_aa_tolerances(r,n1),t2=g3_aa_tolerances(r,n2),t3=g3_aa_tolerances(r,n3);
 double first=r.real(),last=r.real();GeomAbs_Shape cont=g3_aa_shape(r.integer());int deg=r.integer(),seg=r.integer();
 try{std::unique_ptr<AdvApprox_Cutting> cut=g3_aa_cutting(r);
  if(cut){AdvApprox_ApproxAFunction a(n1,n2,n3,t1,t2,t3,first,last,cont,deg,seg,f,*cut);g3_aa_result(a);}
  else{AdvApprox_ApproxAFunction a(n1,n2,n3,t1,t2,t3,first,last,cont,deg,seg,f);g3_aa_result(a);}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 g3_int(f.calls);std::cout<<'\n';
}
// g3_advapprox_static: evaluator, nss (dims...), first last order maxcoeffs maxseg tol code, cutting.
void g3_advapprox_static_case(std::istream& in){
 G3Reader r{in};G3AdvEvaluator f=g3_aa_evaluator(r);int nss=r.integer();NCollection_Array1<int> dims(1,std::max(nss,1));int td=0;
 for(int i=1;i<=nss;++i){dims(i)=r.integer();td+=dims(i);}int given=r.integer();
 double first=r.real(),last=r.real();int order=r.integer(),coeffs=r.integer(),seg=r.integer();double tol=r.real();int code=r.integer();
 NCollection_Array1<double> tols(1,std::max(nss,1));for(int i=1;i<=nss;++i)tols(i)=tol*i;
 int size=std::max(1,seg*coeffs*given);NCollection_Array1<int> ncoeff(1,std::max(seg,1));ncoeff.Init(-7);
 NCollection_Array1<double> coef(1,size),ints(1,seg+1),emax(1,std::max(1,seg*nss)),eavg(1,std::max(1,seg*nss));
 int num=0,err=0;
 try{std::unique_ptr<AdvApprox_Cutting> cut=g3_aa_cutting(r);AdvApprox_DichoCutting dicho;
  AdvApprox_ApproxAFunction::Approximation(given,nss,dims,first,last,f,cut?*cut:dicho,order,coeffs,seg,tols,code,num,ncoeff,coef,ints,emax,eavg,err);
  g3_int(num);g3_int(err);for(int i=1;i<=num;++i)g3_int(ncoeff(i));if(num>0)for(int i=1;i<=num+1;++i)g3_real(ints(i));
  for(double v:coef)g3_real(v);for(int i=1;i<=num*nss;++i){g3_real(emax(i));g3_real(eavg(i));}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 (void)td;g3_int(f.calls);std::cout<<'\n';
}
// g3_advapprox_simple: evaluator, nss (dims...), continuity code maxdeg, first last maxdegree tol.
void g3_advapprox_simple_case(std::istream& in){
 G3Reader r{in};G3AdvEvaluator f=g3_aa_evaluator(r);int nss=r.integer();NCollection_Array1<int> dims(1,nss);int td=0;
 for(int i=1;i<=nss;++i){dims(i)=r.integer();td+=dims(i);}
 GeomAbs_Shape cont=g3_aa_shape(r.integer());int code=r.integer(),maxcoef=r.integer();double first=r.real(),last=r.real();
 int maxdeg=r.integer();double tol=r.real();NCollection_Array1<double> tols(1,nss);for(int i=1;i<=nss;++i)tols(i)=tol*i;
 try{int ng,wd;PLib::JacobiParameters(cont,maxcoef,code,ng,wd);g3_int(ng);g3_int(wd);PLib_JacobiPolynomial jac(wd,cont);
  AdvApprox_SimpleApprox s(td,nss,cont,wd,ng,jac,f);s.Perform(dims,tols,first,last,maxdeg);g3_flag(s.IsDone());
  if(s.IsDone()){for(double v:s.FirstConstr()->Array2())g3_real(v);g3_int(s.Degree());for(double v:s.Coefficients()->Array1())g3_real(v);for(double v:s.LastConstr()->Array2())g3_real(v);
   for(int i=td;i<=s.SomTab()->Upper();++i){g3_real(s.SomTab()->Value(i));g3_real(s.DifTab()->Value(i));}
   if(ng%2==1)for(int i=0;i<td;++i){g3_real(s.SomTab()->Value(i));g3_real(s.DifTab()->Value(i));}
   for(int i=1;i<=nss;++i){g3_real(s.MaxError(i));g3_real(s.AverageError(i));}
   std::ostringstream o;s.Dump(o);std::string t=o.str();for(char& ch:t)if(ch=='\n')ch='|';std::cout<<'['<<t<<"] ";
   s.Perform(dims,tols,first,0.5*(first+last),maxdeg);g3_flag(s.IsDone());g3_int(s.Degree());g3_real(s.MaxError(1));}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 g3_int(f.calls);std::cout<<'\n';
}
// g3_advapprox_cut: a b, then a cutting spec: Dicho and the given tool (tables are non-empty).
void g3_advapprox_cut_case(std::istream& in){
 G3Reader r{in};double a=r.real(),b=r.real();double c=0;AdvApprox_DichoCutting d;g3_flag(d.Value(a,b,c));g3_real(c);
 try{std::unique_ptr<AdvApprox_Cutting> p=g3_aa_cutting(r);if(p){g3_flag(p->Value(a,b,c));g3_real(c);}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 std::cout<<'\n';
}
