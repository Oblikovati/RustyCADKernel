// Specialized Newton protocols at pinned OCCT 3d097a0.
#include <MathSys_Newton2D.hxx>
#include <MathSys_Newton3D.hxx>
#include <MathSys_Newton4D.hxx>
template<size_t N> struct SmallFixture {
 int mode,fail;double scale;mutable std::vector<std::pair<int,std::array<double,N>>> calls;
 bool Record(int kind,const std::array<double,N>& x) const {calls.emplace_back(kind,x);return int(calls.size())!=fail;}
 double Entry(size_t r,size_t c) const {
  if(mode==2||mode==9)return scale*(mode==9?-1.0:1.0);
  if(mode==4)return 0.0;
  if(mode==7||mode==8)return r==c?(r==0?0.01:2.0):((r+c)%2==0?1.5:-1.0);
  return r==c?(r==0?scale:1.0):0.0;
 }
 void Fill(const std::array<double,N>& x,double* f,double (*j)[N]) const {
  for(size_t r=0;r<N;++r){
   for(size_t c=0;c<N;++c)j[r][c]=Entry(r,c);
   f[r]=0.0;for(size_t c=0;c<N;++c)f[r]+=j[r][c]*(x[c]-double(c+1));
   if(mode==1){f[r]=std::atan(x[r]);j[r][r]=1.0/(1.0+x[r]*x[r]);}
   if(mode==3){f[r]=1.0;j[r][r]=1e20;}
   if(mode==4)f[r]=1.0;
   if(mode==5)f[r]=std::numeric_limits<double>::quiet_NaN();
   if(mode==6){f[r]=j[r][r]*(x[r]*x[r]-double(r+1));j[r][r]=j[r][r]*2.0*x[r];}
  }
 }
 bool Evaluate(const std::array<double,N>& x,double* f,double (*j)[N]) const {if(!Record(1,x))return false;Fill(x,f,j);return true;}
 bool Value(double u,double v,double& a,double& b) const {std::array<double,N> x={u,v};if(!Record(0,x))return false;double f[N],j[N][N];Fill(x,f,j);a=f[0];b=f[1];return true;}
 bool ValueAndJacobian(double u,double v,double& a,double& b,double& p,double& q,double& r) const {std::array<double,N> x={u,v};if(!Record(1,x))return false;double f[N],j[N][N];Fill(x,f,j);a=f[0];b=f[1];p=j[0][0];q=j[0][1];r=j[1][1];return true;}
};
template<size_t N> void small_result(const MathSys::NewtonResultN<N>& r,const SmallFixture<N>& f){
 std::cout<<int(r.Status)<<' '<<r.NbIterations<<' '<<r.IsDone()<<' '<<bool(r)<<' '<<bits(r.ResidualNorm)<<' '<<bits(r.StepNorm)<<' ';
 for(double v:r.X)std::cout<<bits(v)<<' ';std::cout<<f.calls.size()<<' ';
 for(auto c:f.calls){std::cout<<c.first<<' ';for(double v:c.second)std::cout<<bits(v)<<' ';}
}
template<size_t N> void small_run(int mode,int fail,double scale,double x,double y,bool bounded,double lo,double hi,const MathSys::NewtonOptions& cfg){
 SmallFixture<N> f{mode,fail,scale,{}};std::array<double,N> start;for(size_t i=0;i<N;++i)start[i]=i%2==0?x:y;
 MathSys::NewtonBoundsN<N> b;b.HasBounds=bounded;b.Min.fill(lo);b.Max.fill(hi);
 if constexpr(N==2){auto fn=[&](double a,double b,double* out,double j[2][2]){return f.Evaluate({a,b},out,j);};small_result(MathSys::Solve2D(fn,start,b,cfg),f);f.calls.clear();small_result(MathSys::Solve2DSymmetric(f,start,b,cfg),f);}
 if constexpr(N==3){auto fn=[&](double a,double b,double c,double* out,double j[3][3]){return f.Evaluate({a,b,c},out,j);};small_result(MathSys::Solve3D(fn,start,b,cfg),f);}
 if constexpr(N==4){auto fn=[&](double a,double b,double c,double d,double* out,double j[4][4]){return f.Evaluate({a,b,c,d},out,j);};small_result(MathSys::Solve4D(fn,start,b,cfg),f);}
}
void modern_small_case(std::istream& in){
 int mode,fail,limit;bool search,bounded,soft;double scale,x,y,ft,xt,ratio,ext,lo,hi;
 in>>mode>>fail>>scale>>x>>y>>ft>>xt>>limit>>ratio>>search>>bounded>>soft>>ext>>lo>>hi;
 MathSys::NewtonOptions c;c.FTolerance=ft;c.XTolerance=xt;c.MaxIterations=limit;c.MaxStepRatio=ratio;c.EnableLineSearch=search;c.AllowSoftBounds=soft;c.SoftBoundsExtension=ext;
 small_run<2>(mode,fail,scale,x,y,bounded,lo,hi,c);small_run<3>(mode,fail,scale,x,y,bounded,lo,hi,c);small_run<4>(mode,fail,scale,x,y,bounded,lo,hi,c);std::cout<<std::endl;
}
