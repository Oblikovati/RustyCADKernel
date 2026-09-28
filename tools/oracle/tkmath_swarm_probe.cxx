// Seeded PSO and exact random stream oracle at OCCT 3d097a0.
#include <MathOpt_PSO.hxx>
struct SwarmFixture {
 int mode,n,fail;std::vector<std::vector<double>> calls;
 bool Value(const math_Vector& x,double& out){
  std::vector<double> point;for(int i=0;i<n;++i)point.push_back(x.At(i));calls.push_back(point);
  if(int(calls.size())==fail||mode==2||(mode==4&&x.At(0)<0))return false;
  if(mode==3){out=std::numeric_limits<double>::quiet_NaN();return true;}
  if(mode==6&&calls.size()%3==0)return true;
  out=0.0;if(mode==5){out=1.0;return true;}
  if(mode==1&&n>=2){double a=x.At(0),b=x.At(1);out=100.0*(b-a*a)*(b-a*a)+(1.0-a)*(1.0-a);return true;}
  for(int i=0;i<n;++i){double d=x.At(i)-0.25;out+=d*d;}return true;
 }
};
void modern_swarm_case(std::istream& in){
 int mode,n,fail,particles,limit,boundary,init,schedule,miniter,stagnation,maxrestart,polish,targetflag,seedkind,velocity;unsigned seed;
 double omega,vclamp,tol,restart,target,pos;
 in>>mode>>n>>fail>>seed>>particles>>limit>>boundary>>init>>schedule>>omega>>vclamp>>tol>>miniter>>stagnation>>restart>>maxrestart>>polish>>targetflag>>target>>seedkind>>velocity>>pos;
 MathOpt::PSOConfig cfg;cfg.NbParticles=particles;cfg.MaxIterations=limit;cfg.BoundaryMode=MathOpt::PSOBoundaryMode(boundary);cfg.InitMode=MathOpt::PSOInitMode(init);cfg.InertiaSchedule=MathOpt::PSOInertiaSchedule(schedule);
 cfg.Seed=seed;cfg.Omega=omega;cfg.VelocityClamp=vclamp;cfg.Tolerance=tol;cfg.MinIterations=miniter;cfg.NoImproveIters=stagnation;cfg.RestartFraction=restart;cfg.MaxRestarts=maxrestart;cfg.PolishBudgetPerDim=polish;if(targetflag)cfg.TargetValue=target;
 math_Vector lower(-3,n-4,-2.0),upper(5,n+4,2.0);NCollection_DynamicArray<MathOpt::PSOSeedParticle> seeds;
 for(int k=0;k<(seedkind==0?0:(seedkind==2?3:1));++k){int dims=seedkind==6?0:(seedkind==3?n+1:n);math_Vector p(7,6+dims,pos+double(k));MathOpt::PSOSeedParticle s(p,-100.0);if(seedkind==4)s.Value=std::numeric_limits<double>::quiet_NaN();if((velocity||seedkind==5)&&seedkind!=7){math_Vector v(-1,n-2,3.0);if(seedkind==5)v.ChangeAt(0)=std::numeric_limits<double>::infinity();s.Velocity=v;}if(seedkind==7){math_Vector bad(size_t(n+1));s.Velocity=bad;}seeds.Append(s);}
 SwarmFixture f{mode,n,fail,{}};MathOpt::PSOStats stats;stats.NbFunctionEvals=999;
 const auto r=MathOpt::PSO(f,lower,upper,cfg,&seeds,&stats);
 std::cout<<int(r.Status)<<' '<<r.NbIterations<<' ';sys_vector(r.Solution);integ_optional(r.Value);sys_vector(r.Gradient);
 std::cout<<stats.NbFunctionEvals<<' '<<stats.NbIterations<<' '<<stats.NbBoundaryCorrections<<' '<<stats.NbStagnationEvents<<' '<<stats.NbRestarts<<' ';integ_optional(stats.InitialBest);integ_optional(stats.FinalBest);
 std::cout<<f.calls.size()<<' ';for(const auto& x:f.calls)for(double v:x)std::cout<<bits(v)<<' ';std::cout<<std::endl;
}
