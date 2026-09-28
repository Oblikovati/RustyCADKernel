// Generic Newton systems at pinned OCCT 3d097a0.
#include <MathSys_Newton.hxx>
struct ModernSystem{
 int mode,m,fail;double target,scale;std::vector<std::tuple<int,double,double>> calls;
 int NbVariables() const{return 2;}int NbEquations() const{return m;}
 bool Record(int kind,const math_Vector& x){calls.emplace_back(kind,x.At(0),x.At(1));return int(calls.size())!=fail;}
 void Residual(const math_Vector& x,math_Vector& f){
  double a=x.At(0),b=x.At(1);if(mode==4&&calls.size()%3==0)return;
  for(int i=0;i<m;++i){double value;
   switch(mode){case 0:value=i==0?a*a+b*b-target:a-b;break;case 2:value=(double(i)+1.0)*(a+scale*b-target);break;case 3:value=std::numeric_limits<double>::quiet_NaN();break;case 5:value=i==0?(1.0+a)*(b-1.0):b*(b-1.0);break;case 6:value=i==0?scale*(a*a-target):b/scale;break;default:value=i==0?a-target:(i==1?b-2.0*target:a+b-3.0*target);}f.ChangeAt(i)=value;
  }
 }
 void Jacobian(const math_Vector& x,math_Matrix& j){
  double a=x.At(0),b=x.At(1);if(mode==4&&calls.size()%3==0)return;
  for(int i=0;i<m;++i){double p,q;
   switch(mode){case 0:p=i==0?2.0*a:1.0;q=i==0?2.0*b:-1.0;break;case 2:p=double(i)+1.0;q=(double(i)+1.0)*scale;break;case 5:p=i==0?b-1.0:0.0;q=i==0?1.0+a:2.0*b-1.0;break;case 6:p=i==0?scale*2.0*a:0.0;q=i==0?0.0:1.0/scale;break;default:p=i==1?0.0:1.0;q=i==0?0.0:1.0;}j.ChangeAt(i,0)=p;j.ChangeAt(i,1)=q;
  }
 }
 bool Value(const math_Vector& x,math_Vector& f){if(!Record(0,x))return false;Residual(x,f);return true;}
 bool Derivatives(const math_Vector& x,math_Matrix& j){if(!Record(2,x))return false;Jacobian(x,j);return true;}
 bool Values(const math_Vector& x,math_Vector& f,math_Matrix& j){if(!Record(1,x))return false;Residual(x,f);Jacobian(x,j);return true;}
};
void sys_vector(const std::optional<math_Vector>& v){std::cout<<v.has_value()<<' ';if(v){std::cout<<v->Lower()<<' '<<v->Size()<<' ';for(size_t i=0;i<v->Size();++i)std::cout<<bits(v->At(i))<<' ';}}
void sys_result(const MathSys::SystemResult& r,ModernSystem& f){
 std::cout<<int(r.Status)<<' '<<r.NbIterations<<' ';sys_vector(r.Solution);integ_optional(r.Value);sys_vector(r.Gradient);
 std::cout<<r.Jacobian.has_value()<<' ';if(r.Jacobian){const auto& j=*r.Jacobian;std::cout<<j.LowerRow()<<' '<<j.LowerCol()<<' '<<j.RowSize()<<' '<<j.ColSize()<<' ';for(size_t i=0;i<j.RowSize();++i)for(size_t k=0;k<j.ColSize();++k)std::cout<<bits(j.At(i,k))<<' ';}
 std::cout<<bits(r.ResidualNorm)<<' '<<bits(r.StepNorm)<<' '<<r.Rank<<' '<<r.IsRoot<<' '<<r.IsStationary<<' '<<f.calls.size()<<' ';
 for(auto c:f.calls)std::cout<<std::get<0>(c)<<' '<<bits(std::get<1>(c))<<' '<<bits(std::get<2>(c))<<' ';f.calls.clear();
}
void modern_sys_case(std::istream& input){
 int mode,m,fail,limit;double target,scale,x,y,tx,ty,ft,lo,hi;input>>mode>>m>>fail>>target>>scale>>x>>y>>tx>>ty>>ft>>limit>>lo>>hi;
 ModernSystem f{mode,m,fail,target,scale,{}};math_Vector start(3,4);start(3)=x;start(4)=y;math_Vector tol(-1,0);tol(-1)=tx;tol(0)=ty;math_Vector lower(7,8,lo),upper(-3,-2,hi);
 sys_result(MathSys::Newton(f,start,tol,ft,limit),f);
 sys_result(MathSys::NewtonBounded(f,start,lower,upper,tol,ft,limit),f);
 sys_result(MathSys::Newton(f,start,tx,ft,limit),f);std::cout<<std::endl;
}

#include <MathSys_LevenbergMarquardt.hxx>
void modern_lm_case(std::istream& input){
 int mode,m,fail,limit;double target,scale,x,y,tol,xt,ft,lo,hi,init,increase,decrease,max,min,rank,rel,step;
 input>>mode>>m>>fail>>target>>scale>>x>>y>>tol>>xt>>ft>>limit>>lo>>hi>>init>>increase>>decrease>>max>>min>>rank>>rel>>step;
 ModernSystem f{mode,m,fail,target,scale,{}};math_Vector start(3,4);start(3)=x;start(4)=y;math_Vector lower(7,8,lo),upper(-3,-2,hi);
 MathSys::LMConfig cfg(tol,limit);cfg.XTolerance=xt;cfg.FTolerance=ft;cfg.RelativeTolerance=rel;cfg.StepMin=step;cfg.LambdaInit=init;cfg.LambdaIncrease=increase;cfg.LambdaDecrease=decrease;cfg.LambdaMax=max;cfg.LambdaMin=min;cfg.RankTolerance=rank;
 sys_result(MathSys::LevenbergMarquardt(f,start,cfg),f);
 sys_result(MathSys::LevenbergMarquardtBounded(f,start,lower,upper,cfg),f);std::cout<<std::endl;
}
