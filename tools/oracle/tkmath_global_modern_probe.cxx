// Modern global strategy dispatch at OCCT 3d097a0.
#include <MathOpt_GlobOpt.hxx>
void global_modern_result(const MathUtils::VectorResult& r,SwarmFixture& f){
 std::cout<<int(r.Status)<<' '<<r.NbIterations<<' ';sys_vector(r.Solution);integ_optional(r.Value);sys_vector(r.Gradient);
 std::cout<<f.calls.size()<<' ';for(const auto& x:f.calls)for(double v:x)std::cout<<bits(v)<<' ';f.calls.clear();
}
void modern_global_case(std::istream& in){
 int mode,n,fail,strategy,pop,starts,limit,polish,custom,seeded;unsigned seed;double mutation,cross,tol;
 in>>mode>>n>>fail>>seed>>strategy>>pop>>starts>>limit>>mutation>>cross>>polish>>tol>>custom>>seeded;
 MathOpt::GlobalConfig cfg;cfg.Strategy=MathOpt::GlobalStrategy(strategy);cfg.NbPopulation=pop;cfg.NbStarts=starts;cfg.MaxIterations=limit;cfg.Seed=seed;cfg.MutationScale=mutation;cfg.CrossoverProb=cross;cfg.PolishBudgetPerDim=polish;cfg.Tolerance=tol;
 math_Vector lower(-2,n-3,-2.0),upper(5,n+4,2.0);SwarmFixture f{mode,n,fail,{}};
 global_modern_result(MathOpt::DifferentialEvolution(f,lower,upper,cfg),f);
 global_modern_result(MathOpt::MultiStart(f,lower,upper,cfg),f);
 global_modern_result(MathOpt::GlobalMinimum(f,lower,upper,cfg),f);
 MathOpt::PSOConfig p;p.NbParticles=4;p.MaxIterations=3;p.Omega=1.5;p.BoundaryMode=MathOpt::PSOBoundaryMode::Reflect;p.PolishBudgetPerDim=0;
 if(seeded)p.InitMode=MathOpt::PSOInitMode::SeededOnly;
 NCollection_DynamicArray<MathOpt::PSOSeedParticle> seeds;math_Vector position(3,n+2,0.25);seeds.Append(MathOpt::PSOSeedParticle(position,-100.0));
 MathOpt::PSOStats stats;stats.NbFunctionEvals=999;
 global_modern_result(MathOpt::GlobalMinimum(f,lower,upper,cfg,custom?&p:nullptr,seeded?&seeds:nullptr,&stats),f);
 std::cout<<stats.NbFunctionEvals<<' '<<stats.NbIterations<<' '<<stats.NbBoundaryCorrections<<' '<<stats.NbStagnationEvents<<' '<<stats.NbRestarts<<' ';integ_optional(stats.InitialBest);integ_optional(stats.FinalBest);std::cout<<std::endl;
}
