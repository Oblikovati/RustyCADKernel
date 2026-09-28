// Modern optimization oracle at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathOpt_Powell.hxx>
#include <MathOpt_BFGS.hxx>
struct ModernOptFunction {
  int mode,fail;double center;std::vector<double> calls;
  bool Record(int kind,const math_Vector& x) {
    calls.push_back(kind);calls.push_back(x(size_t(0)));calls.push_back(x(size_t(1)));
    return int(calls.size()/3)!=fail;
  }
  bool Value(const math_Vector& x,double& f) {
    if(!Record(0,x))return false;
    double a=x(size_t(0))-center,b=x(size_t(1))+center;
    f=a*a+3*b*b;
    if(mode==1)f=1;
    if(mode==2)f=x(size_t(0));
    if(mode==3)f=std::numeric_limits<double>::infinity();
    return true;
  }
  bool Gradient(const math_Vector& x,math_Vector& g) {
    if(!Record(1,x))return false;
    g(size_t(0))=2*(x(size_t(0))-center);g(size_t(1))=6*(x(size_t(1))+center);
    if(mode==1)g.Init(0);
    if(mode==2){g(size_t(0))=1;g(size_t(1))=0;}
    if(mode==4)g(size_t(1))=std::numeric_limits<double>::infinity();
    return true;
  }
  bool Hessian(const math_Vector& x,math_Matrix& h) {
    if(!Record(2,x))return false;
    h.ChangeAt(0,0)=mode==5?-2:2;h.ChangeAt(1,1)=6;
    h.ChangeAt(0,1)=mode==6?1:0;h.ChangeAt(1,0)=0;
    if(mode==7)h.ChangeAt(0,0)=std::numeric_limits<double>::infinity();
    return true;
  }
};
void modern_opt_result(const MathUtils::VectorResult& r,ModernOptFunction& f) {
  std::cout<<int(r.Status)<<' '<<r.NbIterations<<' '<<r.Value.has_value()<<' '<<bits(r.Value.value_or(91))<<' ';
  for(const auto* v:{&r.Solution,&r.Gradient}) {
    std::cout<<v->has_value()<<' ';if(*v)for(int i=0;i<(*v)->Length();++i)std::cout<<bits((**v)(size_t(i)))<<' ';
  }
  std::cout<<f.calls.size()<<' ';for(double c:f.calls)std::cout<<bits(c)<<' ';f.calls.clear();
}
void modern_opt_case(std::istream& input) {
  int mode,fail,limit;double center,x,y,tol,step;input>>mode>>fail>>limit>>center>>x>>y>>tol>>step;
  ModernOptFunction f{mode,fail,center,{}};math_Vector start(-2,-1);start(-2)=x;start(-1)=y;
  MathUtils::Config cfg;cfg.MaxIterations=limit;cfg.XTolerance=tol;cfg.FTolerance=tol;
  math_Matrix directions(-1,0,3,4);directions(-1,3)=1;directions(-1,4)=0.5;directions(0,3)=-0.5;directions(0,4)=1;
  modern_opt_result(MathOpt::Powell(f,start,cfg),f);
  modern_opt_result(MathOpt::PowellWithDirections(f,start,directions,cfg),f);
  modern_opt_result(MathOpt::BFGS(f,start,cfg),f);
  modern_opt_result(MathOpt::BFGSNumerical(f,start,step,cfg),f);std::cout<<'\n';
}

#include <MathOpt_Brent.hxx>
void scalar_opt_case(std::istream& input) {
 int mode,fail,limit;double target,guess,lower,upper,xtol,ftol;
 input>>mode>>target>>guess>>lower>>upper>>xtol>>ftol>>limit>>fail;
 MathUtils::Config config;config.XTolerance=xtol;config.FTolerance=ftol;config.MaxIterations=limit;
 ModernRootFunction f{mode,fail,target,{}};
 modern_root_result(MathOpt::Brent(f,lower,upper,config),f);f.calls.clear();
 modern_root_result(MathOpt::Golden(f,lower,upper,config),f);f.calls.clear();
 modern_root_result(MathOpt::BrentWithBracket(f,guess,upper-lower,config),f);std::cout<<'\n';
}

#include <MathOpt_FRPR.hxx>
void frpr_case(std::istream& input) {
  int mode,fail,limit;double center,x,y,tol,step;input>>mode>>fail>>limit>>center>>x>>y>>tol>>step;
  ModernOptFunction f{mode,fail,center,{}};math_Vector start(-2,-1);start(-2)=x;start(-1)=y;
  MathUtils::Config cfg;cfg.MaxIterations=limit;cfg.XTolerance=tol;cfg.FTolerance=tol;
  math_Matrix directions(-1,0,3,4);directions(-1,3)=1;directions(-1,4)=0.5;directions(0,3)=-0.5;directions(0,4)=1;
  for(int formula=0;formula<5;++formula) {
    MathOpt::FRPRConfig options;static_cast<MathUtils::Config&>(options)=cfg;
    options.Formula=static_cast<MathOpt::ConjugateGradientFormula>(formula);
    options.RestartInterval=mode;
    modern_opt_result(MathOpt::FRPR(f,start,options),f);
    modern_opt_result(MathOpt::FRPRNumerical(f,start,step,options),f);
  }
  std::cout<<'\n';
}

#include <MathOpt_Newton.hxx>
void modern_newton_case(std::istream& input) {
 int mode,fail,limit;double center,x,y,tol,step;input>>mode>>fail>>limit>>center>>x>>y>>tol>>step;
 ModernOptFunction f{mode,fail,center,{}};math_Vector start(-2,-1);start(-2)=x;start(-1)=y;
 MathOpt::NewtonConfig cfg;cfg.MaxIterations=limit;cfg.XTolerance=tol;cfg.FTolerance=tol;cfg.UseLineSearch=mode%2==0;
 cfg.Regularization=mode==8?0:1e-8;
 modern_opt_result(MathOpt::Newton(f,start,cfg),f);
 modern_opt_result(MathOpt::NewtonModified(f,start,cfg),f);
 modern_opt_result(MathOpt::NewtonNumericalHessian(f,start,step,cfg),f);
 modern_opt_result(MathOpt::NewtonNumerical(f,start,1e-8,step,cfg),f);
 math_Vector lo(-1,0,-1.0),hi(3,4,1.0);if(mode==9)hi(3)=-1.0;
 modern_opt_result(MathOpt::NewtonBounded(f,start,lo,hi,cfg),f);
 modern_opt_result(MathOpt::BFGSBounded(f,start,lo,hi,cfg),f);std::cout<<std::endl;
}

void modern_lbfgs_case(std::istream& input) {
 int mode,fail,limit;double center,x,y,tol,step;input>>mode>>fail>>limit>>center>>x>>y>>tol>>step;
 ModernOptFunction f{mode,fail,center,{}};math_Vector start(-2,-1);start(-2)=x;start(-1)=y;
 MathUtils::Config cfg;cfg.MaxIterations=limit;cfg.XTolerance=tol;cfg.FTolerance=tol;
 for(size_t memory:{size_t(0),size_t(1),size_t(2),size_t(10),std::numeric_limits<size_t>::max()})
   modern_opt_result(MathOpt::LBFGS(f,start,memory,cfg),f);
 std::cout<<std::endl;
}
