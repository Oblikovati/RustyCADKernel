// Global search traces, derivative capability dispatch and retained configuration state.
#include <math_GlobOptMin.hxx>
struct GlobalFunction:math_MultipleVarFunctionWithHessian {
 int n,shape,fail,state=0;double shift;GlobalFunction(int dim,int kind,int failure,double center):n(dim),shape(kind),fail(failure),shift(center){}
 int NbVariables()const override{return n;}
 double center(int i)const{return shift+0.25*i;}
 double value(const math_Vector&x){double sum=0;for(int i=1;i<=n;i++){double t=x(i)-center(i);sum+=shape==1?std::sin(t)+0.1*t*t:shape==2?3.0:i*t*t;}return sum;}
 void gradient(const math_Vector&x,math_Vector&g){for(int i=1;i<=n;i++){double t=x(i)-center(i);g(i)=shape==1?std::cos(t)+0.2*t:shape==2?0.0:2.0*i*t;}}
 void trace(char kind,const math_Vector&x){std::cout<<kind<<' ';for(int i=1;i<=n;i++)std::cout<<bits(x(i))<<' ';}
 bool Value(const math_Vector&x,double&v)override{v=value(x);trace('v',x);std::cout<<bits(v)<<' ';return fail!=3;}
 bool Gradient(const math_Vector&x,math_Vector&g)override{gradient(x,g);trace('g',x);for(int i=1;i<=n;i++)std::cout<<bits(g(i))<<' ';return fail!=2;}
 bool Values(const math_Vector&x,double&v,math_Vector&g)override{v=value(x);gradient(x,g);trace('c',x);std::cout<<bits(v)<<' ';for(int i=1;i<=n;i++)std::cout<<bits(g(i))<<' ';return fail!=2&&fail!=3;}
 bool Values(const math_Vector&x,double&v,math_Vector&g,math_Matrix&h)override{v=value(x);gradient(x,g);h.Init(0);for(int i=1;i<=n;i++){double t=x(i)-center(i);h(i,i)=shape==1?-std::sin(t)+0.2:shape==2?0.0:2.0*i;}trace('h',x);std::cout<<bits(v)<<' ';for(int i=1;i<=n;i++)std::cout<<bits(g(i))<<' ';for(int i=1;i<=n;i++)for(int j=1;j<=n;j++)std::cout<<bits(h(i,j))<<' ';return fail!=1&&fail!=3;}
 int GetStateNumber()override{std::cout<<"s "<<++state<<' ';return state;}
};
struct GlobalValue:math_MultipleVarFunction {GlobalFunction&f;GlobalValue(GlobalFunction&v):f(v){}int NbVariables()const override{return f.NbVariables();}bool Value(const math_Vector&x,double&v)override{return f.Value(x,v);}int GetStateNumber()override{return f.GetStateNumber();}};
struct GlobalGradient:math_MultipleVarFunctionWithGradient {GlobalFunction&f;GlobalGradient(GlobalFunction&v):f(v){}int NbVariables()const override{return f.NbVariables();}bool Value(const math_Vector&x,double&v)override{return f.Value(x,v);}bool Gradient(const math_Vector&x,math_Vector&g)override{return f.Gradient(x,g);}bool Values(const math_Vector&x,double&v,math_Vector&g)override{return f.Values(x,v,g);}int GetStateNumber()override{return f.GetStateNumber();}};
void global_result(math_GlobOptMin&s,int n){double tol,same;s.GetTol(tol,same);std::cout<<"out "<<s.isDone()<<' '<<bits(s.GetF())<<' '<<s.NbExtrema()<<' '<<s.GetContinuity()<<' '<<s.GetLipConstState()<<' '<<bits(s.GetFunctionalMinimalValue())<<' '<<bits(tol)<<' '<<bits(same)<<' ';math_Vector p(1,n,91.0);for(int i=1;i<=s.NbExtrema();i++){s.Points(i,p);for(int j=1;j<=n;j++)std::cout<<bits(p(j))<<' ';}for(int i:{0,s.NbExtrema()+1}){try{s.Points(i,p);for(int j=1;j<=n;j++)std::cout<<bits(p(j))<<' ';}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}}}
void global_case(std::istream&input){
 int n,shape,fail,capability,continuity,locked,single;double shift,lo,hi,c,tol,same;input>>n>>shape>>fail>>capability>>continuity>>locked>>single>>shift>>lo>>hi>>c>>tol>>same;
 GlobalFunction f(n,shape,fail,shift),g(n,shape,fail,shift+0.3);GlobalValue fv(f),gv(g);GlobalGradient fg(f),gg(g);
 math_MultipleVarFunction* fp=capability==0?static_cast<math_MultipleVarFunction*>(&fv):capability==1?static_cast<math_MultipleVarFunction*>(&fg):static_cast<math_MultipleVarFunction*>(&f);
 math_MultipleVarFunction* gp=capability==0?static_cast<math_MultipleVarFunction*>(&gv):capability==1?static_cast<math_MultipleVarFunction*>(&gg):static_cast<math_MultipleVarFunction*>(&g);
 math_Vector lower(1,n),upper(1,n),localLo(1,n),localHi(1,n);for(int i=1;i<=n;i++){lower(i)=lo;upper(i)=hi;localLo(i)=lo+0.1*(hi-lo);localHi(i)=hi-0.1*(hi-lo);}
 math_GlobOptMin s(fp,lower,upper,c,tol,same);global_result(s,n);
 s.SetContinuity(continuity);s.SetLipConstState(locked!=0);s.SetFunctionalMinimalValue(shape==2?3.0*n:0.0);s.SetTol(tol,same);s.Perform(single!=0);global_result(s,n);
 s.SetLocalParams(localLo,localHi);s.Perform(single==0);global_result(s,n);
 s.SetGlobalParams(gp,lower,upper,c,tol,same);global_result(s,n);s.Perform(true);global_result(s,n);std::cout<<'\n';
}
