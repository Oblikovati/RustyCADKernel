// MathSys::Utils oracle at pinned 3d097a0: every public helper of MathSys_LinearAlgebra.hxx,
// MathSys_Newton.hxx, MathSys_LevenbergMarquardt.hxx, MathSys_NewtonTypes.hxx and
// MathSys_Newton2D/3D/4D.hxx. Reuses ModernSystem, sys_result and mpu_ld from the modern
// system and MathPoly::Utils probes; long doubles print as their 80-bit pattern.
#include <MathSys_LevenbergMarquardt.hxx>
#include <MathSys_Newton.hxx>
#include <MathSys_Newton2D.hxx>
#include <MathSys_Newton3D.hxx>
#include <MathSys_Newton4D.hxx>
// Doubles are read through strtod so the corpus can carry nan/inf tokens.
double msa_d(std::istream& in){std::string token;in>>token;return std::strtod(token.c_str(),nullptr);}
template<size_t N> void msa_array(const std::array<double,N>& x){for(double v:x)std::cout<<bits(v)<<' ';}
template<size_t N> void msa_checks(double (&j)[4][4],double (&f)[4],const std::array<double,4>& step,double steptol){
 double sj[N][N],sf[N];for(size_t r=0;r<N;++r){sf[r]=f[r];for(size_t c=0;c<N;++c)sj[r][c]=j[r][c];}
 std::array<double,N> st,left;for(size_t i=0;i<N;++i){st[i]=step[i];left[i]=f[i];}
 std::cout<<MathSys::Utils::IsFiniteSystemN(sf,sj)<<' '<<MathSys::Utils::IsFiniteArray(st)<<' '<<bits(MathSys::Utils::SafeNormN(sf))<<' ';
 std::cout<<bits(MathSys::Utils::SafeDistanceN(left,st))<<' '<<MathSys::Utils::IsStepWithinTolerance(left,st,steptol)<<' ';
 mpu_ld(MathSys::Utils::MeritDirectionalDerivative(sf,sj,st));
}
void mathsys_small_linear_case(std::istream& in){
 double j[4][4],f[4];std::array<double,4> step;double tol,init,current,trial,steptol;
 for(auto& row:j)for(double& v:row)v=msa_d(in);for(double& v:f)v=msa_d(in);for(double& v:step)v=msa_d(in);tol=msa_d(in);init=msa_d(in);current=msa_d(in);trial=msa_d(in);steptol=msa_d(in);
 double j2[2][2]={{j[0][0],j[0][1]},{j[1][0],j[1][1]}},f2[2]={f[0],f[1]};double du=init,dv=init;
 std::cout<<MathSys::Utils::Solve2x2(j2,f2,du,dv)<<' '<<bits(du)<<' '<<bits(dv)<<' ';
 double j3[3][3],f3[3],d3[3]={init,init,init};for(int r=0;r<3;++r){f3[r]=f[r];for(int c=0;c<3;++c)j3[r][c]=j[r][c];}
 std::cout<<MathSys::Utils::Solve3x3(j3,f3,d3)<<' '<<bits(d3[0])<<' '<<bits(d3[1])<<' '<<bits(d3[2])<<' ';
 double d4[4]={init,init,init,init};
 std::cout<<MathSys::Utils::Solve4x4(j,f,d4)<<' ';for(double v:d4)std::cout<<bits(v)<<' ';
 du=init;dv=init;std::cout<<MathSys::Utils::SolveSymmetric2x2SVD(j[0][0],j[0][1],j[1][1],f[0],f[1],du,dv,tol)<<' '<<bits(du)<<' '<<bits(dv)<<' ';
 du=init;dv=init;std::cout<<MathSys::Utils::SolveSymmetric2x2SVD(j[0][0],j[0][1],j[1][1],f[0],f[1],du,dv)<<' '<<bits(du)<<' '<<bits(dv)<<' ';
 msa_checks<2>(j,f,step,steptol);msa_checks<3>(j,f,step,steptol);msa_checks<4>(j,f,step,steptol);
 double j4[4][4];double f4[4];for(int r=0;r<4;++r){f4[r]=f[r];for(int c=0;c<4;++c)j4[r][c]=j[r][c];}
 const long double derivative=MathSys::Utils::MeritDirectionalDerivative(f4,j4,step);
 std::cout<<MathSys::Utils::IsArmijoAccepted(current,trial,derivative)<<' '<<MathSys::Utils::IsArmijoAccepted(current,trial,static_cast<long double>(tol))<<' '<<std::endl;
}
template<size_t N> MathSys::NewtonBoundsN<N> msa_bounds(const double* lo,const double* hi,bool has){MathSys::NewtonBoundsN<N> b;b.HasBounds=has;for(size_t i=0;i<N;++i){b.Min[i]=lo[i];b.Max[i]=hi[i];}return b;}
void msa_step(const std::optional<double>& v){std::cout<<v.has_value()<<' '<<bits(v.value_or(91.0))<<' ';}
void mathsys_small_box_case(std::istream& in){
 double lo[4],hi[4],x[4];double has,soft,ratio,step,ft,xt,limit,stepratio,ext;
 for(double& v:lo)v=msa_d(in);for(double& v:hi)v=msa_d(in);for(double& v:x)v=msa_d(in);has=msa_d(in);soft=msa_d(in);ratio=msa_d(in);step=msa_d(in);ft=msa_d(in);xt=msa_d(in);limit=msa_d(in);stepratio=msa_d(in);ext=msa_d(in);
 auto b2=msa_bounds<2>(lo,hi,has!=0);auto b3=msa_bounds<3>(lo,hi,has!=0);auto b4=msa_bounds<4>(lo,hi,has!=0);
 std::cout<<MathSys::Utils::IsBoundsValid2D(b2)<<' '<<MathSys::Utils::IsBoundsValid3D(b3)<<' '<<MathSys::Utils::IsBoundsValid4D(b4)<<' ';
 msa_step(MathSys::Utils::MaxStep2D(b2,step));msa_step(MathSys::Utils::MaxStep3D(b3,step));msa_step(MathSys::Utils::MaxStep4D(b4,step));
 std::array<double,2> x2={x[0],x[1]};std::array<double,3> x3={x[0],x[1],x[2]};std::array<double,4> x4={x[0],x[1],x[2],x[3]};
 MathSys::Utils::Clamp2D(x2,b2,soft!=0,ratio);MathSys::Utils::Clamp3D(x3,b3,soft!=0,ratio);MathSys::Utils::Clamp4D(x4,b4,soft!=0,ratio);
 msa_array(x2);msa_array(x3);msa_array(x4);
 MathSys::NewtonOptions o;o.FTolerance=ft;o.XTolerance=xt;o.MaxIterations=static_cast<uint32_t>(limit);o.MaxStepRatio=stepratio;o.SoftBoundsExtension=ext;
 std::cout<<MathSys::Utils::IsOptionsValid(o)<<' '<<MathSys::Utils::IsOptionsValid3D(o)<<' '<<MathSys::Utils::IsOptionsValid4D(o)<<' '<<std::endl;
}
void mathsys_linear_api_case(std::istream& in){
 int m,n;in>>m>>n;const double tol=msa_d(in),init=msa_d(in);math_Matrix a(1,m,1,n);math_Vector b(1,m);
 for(int r=1;r<=m;++r)for(int c=1;c<=n;++c)a(r,c)=msa_d(in);for(int r=1;r<=m;++r)b(r)=msa_d(in);
 std::cout<<bits(MathSys::Utils::SafeNorm(b))<<' '<<MathSys::Utils::IsFinite(b)<<' '<<MathSys::Utils::IsFinite(a)<<' ';
 const std::optional<size_t> rank=MathSys::Utils::ScaledRank(a,tol),defrank=MathSys::Utils::ScaledRank(a);
 std::cout<<rank.has_value()<<' '<<rank.value_or(91)<<' '<<defrank.has_value()<<' '<<defrank.value_or(91)<<' ';
 math_Vector x(1,n,init);std::cout<<MathSys::Utils::SolveQR(a,b,x,tol)<<' ';for(int i=1;i<=n;++i)std::cout<<bits(x(i))<<' ';
 math_Vector y(1,n,init);std::cout<<MathSys::Utils::SolveQR(a,b,y)<<' ';for(int i=1;i<=n;++i)std::cout<<bits(y(i))<<' ';
 const MathSys::Utils::LinearStep s=MathSys::Utils::SolveLinearized(a,b,tol);std::cout<<int(s.Status)<<' '<<s.Rank<<' ';sys_vector(s.Solution);
 const MathSys::Utils::LinearStep d=MathSys::Utils::SolveLinearized(a,b);std::cout<<int(d.Status)<<' '<<d.Rank<<' ';sys_vector(d.Solution);std::cout<<std::endl;
}
void mathsys_finish_api_case(std::istream& input){
 int mode,m,fail,limit,mask;double target,scale,x,y,ft,lo,hi,tol,initial;
 input>>mode>>m>>fail>>target>>scale>>x>>y>>ft>>limit>>lo>>hi>>mask>>tol>>initial;
 ModernSystem f{mode,m,fail,target,scale,{}};math_Vector point(3,4);point(3)=x;point(4)=y;math_Vector tolx(0,1,ft);
 math_Vector lower(7,8,lo),upper(-3,-2,hi);const math_Vector* inf=(mask&1)?&lower:nullptr;const math_Vector* sup=(mask&2)?&upper:nullptr;
 MathSys::SystemResult a;std::cout<<MathSys::Utils::FinalizeSystemResult(f,point,size_t(m),ft,a)<<' ';sys_result(a,f);
 MathSys::LMConfig cfg(tol,limit);MathSys::SystemResult b;
 std::cout<<MathSys::Utils::FinishLM(f,point,size_t(m),cfg,initial,inf,sup,b)<<' ';sys_result(b,f);
 sys_result(MathSys::Utils::NewtonImpl(f,point,tolx,ft,limit,inf,sup),f);
 sys_result(MathSys::Utils::LMImpl(f,point,cfg,inf,sup),f);
 const MathSys::LMConfig def;const MathSys::NewtonOptions opt;
 std::cout<<bits(def.Tolerance)<<' '<<bits(def.LambdaInit)<<' '<<bits(def.RankTolerance)<<' '<<def.MaxIterations<<' '<<bits(cfg.StepMin)<<' ';
 std::cout<<bits(opt.Tolerance)<<' '<<bits(opt.XTolerance)<<' '<<bits(opt.FTolerance)<<' '<<bits(opt.RelativeTolerance)<<' '<<bits(opt.StepMin)<<' '<<opt.MaxIterations<<' '<<bits(opt.MaxStepRatio)<<' '<<opt.EnableLineSearch<<' '<<opt.AllowSoftBounds<<' '<<bits(opt.SoftBoundsExtension)<<' ';
 std::cout<<MathSys::Utils::THE_LM_DAMPING_MAX<<' '<<MathSys::Utils::THE_LINE_SEARCH_MAX<<' '<<bits(MathSys::Utils::THE_ARMIJO_C1)<<' '<<bits(MathSys::Utils::THE_NEWTON_PIVOT_TOL)<<' '<<bits(MathSys::Utils::THE_RANK_TOL)<<' '<<bits(MathSys::Utils::THE_CRITICAL_GRAD)<<' '<<bits(MathSys::Utils::THE_CRITICAL_GRAD_SQ)<<' '<<std::endl;
}
