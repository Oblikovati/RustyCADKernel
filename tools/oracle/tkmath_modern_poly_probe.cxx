// Direct polynomial oracle at pinned 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathPoly_Quartic.hxx>
void modern_poly_result(const MathPoly::PolyResult& r){
 std::cout<<int(r.Status)<<' '<<r.NbRoots<<' '<<r.NbComplexRoots<<' '<<r.NbRealRootsWithMultiplicity()<<' ';
 for(int i=0;i<MathPoly::THE_MAX_POLY_DEGREE;++i)std::cout<<bits(r.Roots[i])<<' '<<r.Multiplicities[i]<<' '<<bits(r.ComplexRoots[i].real())<<' '<<bits(r.ComplexRoots[i].imag())<<' '<<r.ComplexMultiplicities[i]<<' ';
}
void modern_poly_case(std::istream& input){
 double c[5];int mode;for(double& x:c)input>>x;input>>mode;
 if(mode==1)c[0]=std::numeric_limits<double>::quiet_NaN();else if(mode==2)c[4]=std::numeric_limits<double>::infinity();
 modern_poly_result(MathPoly::Linear(c[0],c[1]));modern_poly_result(MathPoly::Quadratic(c[0],c[1],c[2]));modern_poly_result(MathPoly::Cubic(c[0],c[1],c[2],c[3]));modern_poly_result(MathPoly::Quartic(c[0],c[1],c[2],c[3],c[4]));std::cout<<std::endl;
}
