// TKGeomBase oracle: the AdvApp2Var test functions (tkgeombase_probe.cxx). Each function
// F(u,v) of R^3 is a list of separable terms c * g(u) * h(v) whose factors are powers,
// cos(a x), sin(a x) or exp(a x); partial derivatives of any order come from the factor
// derivatives, so the C++ and Rust twins (crates/tkgeombase/examples/gb_probe/adv_eval.rs)
// evaluate them with the same operations. Function id + 10 fails with code 7 when the
// constant parameter exceeds 0.5.
#include <AdvApp2Var_EvaluatorFunc2Var.hxx>
struct GbAaFactor{int kind;double a;};  // 0: x^a (a integer), 1: cos(a x), 2: sin(a x), 3: exp(a x)
struct GbAaTerm{int comp;double c;GbAaFactor g,h;};
double gb_aa_factor(const GbAaFactor& f,int k,double x){
 if(f.kind==0){const int n=int(f.a);if(k>n)return 0.;double c=1.;for(int i=0;i<k;++i)c*=double(n-i);
  double p=1.;for(int i=0;i<n-k;++i)p*=x;return c*p;}
 double s=1.;for(int i=0;i<k;++i)s*=f.a;const double t=f.a*x;
 if(f.kind==3)return s*std::exp(t);
 const int phase=(f.kind==1?k:k+3)%4;  // sin(t) = cos(t - pi/2): three quarter turns ahead
 const double v=(phase==0)?std::cos(t):(phase==1)?-std::sin(t):(phase==2)?-std::cos(t):std::sin(t);
 return s*v;
}
std::vector<GbAaTerm> gb_aa_terms(int id){
 switch(id){
  case 0:return {{0,1.,{0,1},{0,0}},{1,1.,{0,0},{0,1}},{2,1.,{0,1},{0,1}},{2,0.5,{0,2},{0,0}}};
  case 1:return {{0,1.,{1,1},{1,1}},{1,1.,{2,1},{1,1}},{2,1.,{0,0},{2,1}}};
  case 2:return {{0,1.,{0,1},{0,0}},{1,1.,{0,0},{0,1}},{2,1.,{3,0.7},{2,2}},{2,0.3,{0,2},{0,1}}};
  default:return {{0,1.,{0,1},{0,0}},{0,0.2,{0,0},{0,3}},{1,1.,{0,0},{0,1}},{1,-0.1,{0,2},{0,0}},{2,1.,{1,1},{3,-0.5}}};
 }
}
class GbAaFunction : public AdvApp2Var_EvaluatorFunc2Var{
 public:
 int id;explicit GbAaFunction(int theId):id(theId){}
 void Evaluate(int* dim,double*,double*,int* iso,double* cst,int* n,double* par,int* du,int* dv,double* res,int* ier) const override{
  const std::vector<GbAaTerm> terms=gb_aa_terms(id%10);
  for(int i=0;i<*n;++i){const double u=*iso==1?*cst:par[i],v=*iso==1?par[i]:*cst;
   for(int c=0;c<*dim;++c){double sum=0.;for(const GbAaTerm& t:terms)if(t.comp==c%3)sum+=t.c*gb_aa_factor(t.g,*du,u)*gb_aa_factor(t.h,*dv,v);
    res[c+i**dim]=sum;}}
  *ier=(id>=10&&*cst>0.5)?7:0;
 }
};
