// Fixed-seed particle trajectories, mutable pools, ignored callback status and repeated Perform.
#include <math_PSO.hxx>
#include <math_PSOParticlesPool.hxx>
#include <math_BullardGenerator.hxx>
struct SwarmFunction:math_MultipleVarFunction {
 int dimension,mode,calls=0;SwarmFunction(int n,int m):dimension(n),mode(m){}
 int NbVariables()const override{return dimension;}
 bool Value(const math_Vector&x,double&v)override{v=0;for(int i=1;i<=dimension;i++){double t=x(i)-0.25*i;v+=i*t*t;}if(mode==1)v=3;bool good=mode!=2||x(1)>0.0;std::cout<<"v ";for(int i=1;i<=dimension;i++)std::cout<<bits(x(i))<<' ';std::cout<<bits(v)<<' '<<good<<' ';return good;}
};
void swarm_output(double value,const math_Vector&point){std::cout<<"out "<<bits(value)<<' ';for(int i=1;i<=point.Length();i++)std::cout<<bits(point(i))<<' ';}
void pool_output(math_PSOParticlesPool&pool,int count,int dimension){for(int i=1;i<=count;i++){auto*p=pool.GetParticle(i);std::cout<<bits(p->Distance)<<' '<<bits(p->BestDistance)<<' ';for(int j=0;j<dimension;j++)std::cout<<bits(p->Position[j])<<' '<<bits(p->Velocity[j])<<' '<<bits(p->BestPosition[j])<<' ';}std::cout<<bits(pool.GetBestParticle()->Distance)<<' '<<bits(pool.GetWorstParticle()->Distance)<<' ';}
void pso_case(std::istream&input){
 int n,mode,count,limit;double lower,upper,step;input>>n>>mode>>count>>limit>>lower>>upper>>step;
 SwarmFunction f(n,mode);math_Vector low(1,n),high(1,n),steps(1,n),point(1,n);double value=71;
 for(int i=1;i<=n;i++){low(i)=lower-0.1*i;high(i)=upper+0.2*i;steps(i)=step+0.07*i;}
 math_PSO solver(&f,low,high,steps,count,7);solver.Perform(steps,value,point,limit);swarm_output(value,point);
 math_PSOParticlesPool pool(count+1,n);pool_output(pool,count+1,n);
 for(int i=1;i<=count+1;i++){auto*p=pool.GetParticle(i);for(int j=0;j<n;j++){p->Position[j]=lower+(upper-lower)*(0.1+0.13*i)+0.2*j;p->BestPosition[j]=p->Position[j];point(j+1)=p->Position[j];}f.Value(point,p->Distance);p->BestDistance=p->Distance;}
 solver.Perform(pool,count,value,point,limit);swarm_output(value,point);pool_output(pool,count+1,n);
 for(int i=1;i<=n;i++)steps(i)*=1.3;solver.Perform(steps,value,point,limit);swarm_output(value,point);std::cout<<'\n';
}
void bullard_case(std::istream&input){unsigned int seed;input>>seed;math_BullardGenerator random(seed);for(int i=0;i<32;i++)std::cout<<random.NextInt()<<' '<<bits(random.NextReal())<<' ';random.SetSeed();std::cout<<random.NextInt()<<' '<<bits(random.NextReal())<<' '<<'\n';}
