// Direct and Laguerre polynomial oracle at pinned 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathPoly_Laguerre.hxx>
void modern_poly_result(const MathPoly::PolyResult& r){
 std::cout<<int(r.Status)<<' '<<r.NbRoots<<' '<<r.NbComplexRoots<<' '<<r.NbRealRootsWithMultiplicity()<<' ';
 for(int i=0;i<MathPoly::THE_MAX_POLY_DEGREE;++i)std::cout<<bits(r.Roots[i])<<' '<<r.Multiplicities[i]<<' '<<bits(r.ComplexRoots[i].real())<<' '<<bits(r.ComplexRoots[i].imag())<<' '<<r.ComplexMultiplicities[i]<<' ';
}
void modern_poly_case(std::istream& input){
 double c[5];int mode;for(double& x:c)input>>x;input>>mode;
 if(mode==1)c[0]=std::numeric_limits<double>::quiet_NaN();else if(mode==2)c[4]=std::numeric_limits<double>::infinity();
 modern_poly_result(MathPoly::Linear(c[0],c[1]));modern_poly_result(MathPoly::Quadratic(c[0],c[1],c[2]));modern_poly_result(MathPoly::Cubic(c[0],c[1],c[2],c[3]));modern_poly_result(MathPoly::Quartic(c[0],c[1],c[2],c[3],c[4]));std::cout<<std::endl;
}
// Laguerre entry points: mode 0 Laguerre, 1 NaN coefficient, 2 infinite coefficient, 3 NaN tolerance,
// 4 LaguerreN(size=degree+1), 5 Quintic, 6 Sextic, 7 Octic. Coefficients are ascending, 21 slots.
void modern_laguerre_case(std::istream& input){
 int degree,mode;double tolerance;unsigned iterations;double c[21];input>>degree>>tolerance>>iterations>>mode;for(double& x:c)input>>x;
 if(mode==1)c[1]=std::numeric_limits<double>::quiet_NaN();else if(mode==2)c[0]=-std::numeric_limits<double>::infinity();else if(mode==3)tolerance=std::numeric_limits<double>::quiet_NaN();
 if(mode==4)modern_poly_result(MathPoly::LaguerreN(c,size_t(degree+1),tolerance,iterations));
 else if(mode==5)modern_poly_result(MathPoly::Quintic(c[5],c[4],c[3],c[2],c[1],c[0]));
 else if(mode==6)modern_poly_result(MathPoly::Sextic(c[6],c[5],c[4],c[3],c[2],c[1],c[0]));
 else if(mode==7)modern_poly_result(MathPoly::Octic(c));
 else modern_poly_result(MathPoly::Laguerre(c,degree,tolerance,iterations));
 std::cout<<std::endl;
}
// std::complex<double> operations with libgcc/glibc semantics; mode 1..4 inject non-finite parts.
void modern_complex_case(std::istream& input){
 double v[4];int mode;for(double& x:v)input>>x;input>>mode;
 if(mode==1)v[0]=std::numeric_limits<double>::infinity();else if(mode==2)v[1]=std::numeric_limits<double>::quiet_NaN();else if(mode==3)v[3]=-std::numeric_limits<double>::infinity();else if(mode==4){v[2]=0.0;v[3]=0.0;}
 volatile double a=v[0],b=v[1],cc=v[2],d=v[3];const std::complex<double> z(a,b),w(cc,d);
 const std::complex<double> values[]={z*w,z/w,std::sqrt(z),v[2]/w,z+v[3],z*v[3],std::conj(z),std::polar(std::abs(v[2]),v[3])};
 for(const std::complex<double>& x:values)std::cout<<bits(x.real())<<' '<<bits(x.imag())<<' ';
 std::cout<<bits(std::abs(z))<<' '<<bits(std::norm(z))<<std::endl;
}
