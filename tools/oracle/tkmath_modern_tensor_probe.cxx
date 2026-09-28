// Tensor and vector quadrature at pinned OCCT 3d097a0.
#include <MathInteg_Multiple.hxx>
#include <MathInteg_Set.hxx>
struct ModernTensorObjective{
 int mode,fail,equations;std::vector<std::vector<double>> calls;
 bool Record(const math_Vector& x){std::vector<double> p;for(size_t i=0;i<x.Size();++i)p.push_back(x.At(i));calls.push_back(p);return int(calls.size())!=fail;}
 int NbEquations() const{return equations;}
 bool Value(const math_Vector& x,double& out){if(!Record(x))return false;if(mode==2&&calls.size()%3==0)return true;double value=1.0;for(size_t i=0;i<x.Size();++i)value*=x.At(i)+1.0;out=mode==1?std::numeric_limits<double>::quiet_NaN():(mode==3?std::numeric_limits<double>::max():value);return true;}
 bool Value(const math_Vector& x,math_Vector& out){if(!Record(x))return false;if(mode==2&&calls.size()%3==0)return true;for(size_t j=0;j<out.Size();++j)out.ChangeAt(j)=mode==1?std::numeric_limits<double>::quiet_NaN():(mode==3?std::numeric_limits<double>::max():x.At(0)*x.At(0)+double(j));return true;}
 void Trace(){std::cout<<calls.size()<<' ';for(const auto& p:calls){std::cout<<p.size()<<' ';for(double x:p)std::cout<<bits(x)<<' ';}calls.clear();}
};
void tensor_result(const MathUtils::IntegResult& r,ModernTensorObjective& f){std::cout<<int(r.Status)<<' '<<r.NbIterations<<' '<<r.NbPoints<<' '<<r.Value.has_value()<<' ';if(r.Value)std::cout<<bits(*r.Value)<<' ';f.Trace();}
void tensor_set_result(const MathInteg::SetResult& r,ModernTensorObjective& f){std::cout<<int(r.Status)<<' '<<r.NbEquations<<' '<<r.NbPoints<<' '<<r.Values.has_value()<<' ';if(r.Values){std::cout<<r.Values->Size()<<' ';for(size_t i=0;i<r.Values->Size();++i)std::cout<<bits(r.Values->At(i))<<' ';}f.Trace();}
void modern_tensor_case(std::istream& input){
 int n,mode,fail,equations,order,maxorder;double a,b;input>>n>>mode>>fail>>equations>>a>>b>>order>>maxorder;
 ModernTensorObjective f{mode,fail,equations,{}};math_Vector lower(-2,n-3),upper(5,n+4);math_IntegerVector orders(9,n+8);
 for(int i=0;i<n;++i){lower.ChangeAt(i)=a+double(i)*0.1;upper.ChangeAt(i)=b+double(i)*0.1;orders.ChangeAt(i)=order+i%2;}
 MathInteg::MultipleConfig cfg;cfg.MaxOrder=maxorder;
 tensor_result(MathInteg::GaussMultiple(f,n,lower,upper,orders,cfg),f);
 tensor_result(MathInteg::GaussMultipleUniform(f,n,lower,upper,order),f);
 tensor_set_result(MathInteg::GaussSet(f,a,b,order),f);
 tensor_set_result(MathInteg::GaussSet(f,lower,upper,order),f);std::cout<<std::endl;
}
