// MathPoly::Utils oracle at pinned 3d097a0: every public helper of MathPoly_Utils.hxx and of the
// Utils namespace in MathPoly_Laguerre.hxx. long double values print as their 80-bit pattern
// (sign/exponent word, then significand, in hex) and NaNs as "LNaN"; doubles print as bits.
#include <MathPoly_Laguerre.hxx>
#include <MathPoly_Utils.hxx>
void mpu_ld(long double value){
 if(std::isnan(value)){std::cout<<"LNaN ";return;}
 unsigned char raw[16]={0};std::memcpy(raw,&value,10);uint64_t significand;uint16_t head;
 std::memcpy(&significand,raw,8);std::memcpy(&head,raw+8,2);std::cout<<'L'<<std::hex<<head<<'_'<<significand<<std::dec<<' ';
}
void mpu_z(const std::complex<double>& z){std::cout<<bits(z.real())<<' '<<bits(z.imag())<<' ';}
void mpu_result(const MathPoly::PolyResult& r){
 std::cout<<int(r.Status)<<' '<<r.NbRoots<<' '<<r.NbComplexRoots<<' ';
 for(size_t i=0;i<r.NbRoots;++i)std::cout<<bits(r.Roots[i])<<' '<<r.Multiplicities[i]<<' ';
 for(size_t i=0;i<r.NbComplexRoots;++i){mpu_z(r.ComplexRoots[i]);std::cout<<r.ComplexMultiplicities[i]<<' ';}
}
void mpu_iteration(const MathPoly::Utils::IterationResult& r){
 std::cout<<int(r.Status)<<' ';mpu_z(r.Root);std::cout<<r.RootError.has_value()<<' '<<bits(r.RootError.value_or(91.0))<<' '<<r.NbIterations<<' ';
}
struct MpuRequest{int degree,special,multiplicity,exponent,nbReal,nbComplex;unsigned iterations;double c[7],x,tolerance,real[6];
 std::complex<double> z,start,complexRoots[4];};
MpuRequest mpu_read(std::istream& in){
 MpuRequest q;double v;in>>v;q.degree=int(v);for(double& x:q.c)in>>x;double zr,zi,sr,si;in>>q.x>>q.tolerance>>zr>>zi>>v;q.multiplicity=int(v);
 in>>v;q.iterations=unsigned(v);in>>v;q.exponent=int(v);in>>sr>>si>>v;q.nbReal=int(v);for(double& x:q.real)in>>x;in>>v;q.nbComplex=int(v);
 for(auto& w:q.complexRoots){double a,b;in>>a>>b;w={a,b};}in>>v;q.special=int(v);q.z={zr,zi};q.start={sr,si};
 if(q.special==1)q.c[0]=std::numeric_limits<double>::quiet_NaN();
 if(q.special==2)q.c[q.degree]=std::numeric_limits<double>::infinity();
 if(q.special==3)q.tolerance=std::numeric_limits<double>::quiet_NaN();
 if(q.special==4)q.x=std::numeric_limits<double>::infinity();
 return q;
}
// Validate, ToDouble, ScalePolynomial, Evaluate, HasSmallResidual, RootBound, AppendUnique,
// SolveDistinctReal and SolveDirect on long-double coefficients.
void mathpoly_utils_long_case(std::istream& in){
 using namespace MathPoly::Utils;const MpuRequest q=mpu_read(in);const int d=q.degree;
 std::cout<<Validate(q.c,d,q.tolerance)<<' '<<Validate(q.c,d)<<' '<<Validate(nullptr,d)<<' ';
 const std::optional<double> narrowed=ToDouble(std::scalbn(static_cast<long double>(q.x),q.exponent));std::cout<<narrowed.has_value()<<' '<<bits(narrowed.value_or(91.0))<<' ';
 LongCoefficients scaled={};int power=77;const bool ok=d>=0&&d<=6&&ScalePolynomial(q.c,d,scaled,power);std::cout<<ok<<' '<<power<<' ';
 if(!ok)for(int i=0;i<=std::min(std::max(d,0),6);++i)scaled[size_t(i)]=q.c[i];
 for(int i=0;i<=6;++i)mpu_ld(scaled[size_t(i)]);
 if(d<0||d>6){std::cout<<'\n';return;}
 long double value=1,derivative=2,magnitude=3;Evaluate(scaled,d,q.x,value,derivative,magnitude);mpu_ld(value);mpu_ld(derivative);mpu_ld(magnitude);
 std::cout<<HasSmallResidual(scaled,d,q.x,THE_DISTINCT_ROOT_RESIDUAL_TOLERANCE)<<' '<<HasSmallResidual(scaled,d,q.x,static_cast<long double>(q.tolerance))<<' ';
 mpu_ld(RootBound(scaled,d));
 std::array<long double,4> roots={q.real[0],q.real[1],q.real[2],0.0L};size_t count=size_t(std::min(std::max(q.nbReal,0),3));
 AppendUnique(roots,count,q.x);std::cout<<count<<' ';for(size_t i=0;i<count;++i)mpu_ld(roots[i]);
 if(d>=1&&d<=4){std::array<long double,4> found={};size_t nb=0;std::cout<<SolveDistinctReal(scaled,d,found,nb)<<' '<<nb<<' ';for(size_t i=0;i<nb;++i)mpu_ld(found[i]);}
 if(d<=4)mpu_result(SolveDirect(q.c,d));
 std::cout<<'\n';
}
// EvaluateComplex, the complex HasSmallResidual, LaguerreIteration, FindRoot, DeflateReal,
// DeflateComplex, HasMultiplicity, HasComplexMultiplicity and both Consolidate helpers.
void mathpoly_utils_complex_case(std::istream& in){
 using namespace MathPoly::Utils;const MpuRequest q=mpu_read(in);const int d=q.degree;
 std::complex<double> value,derivative,second;double magnitude=91;EvaluateComplex(q.c,d,q.z,value,derivative,second,magnitude);
 mpu_z(value);mpu_z(derivative);mpu_z(second);std::cout<<bits(magnitude)<<' '<<HasSmallResidual(q.c,d,q.z,q.tolerance)<<' ';
 mpu_iteration(LaguerreIteration(q.c,d,q.start,q.tolerance,q.iterations));mpu_iteration(FindRoot(q.c,d,q.tolerance,q.iterations));
 double work[21]={};std::copy(q.c,q.c+7,work);int degree=d;std::cout<<DeflateReal(work,degree,q.x,q.tolerance)<<' '<<degree<<' ';for(int i=0;i<=degree;++i)std::cout<<bits(work[i])<<' ';
 if(d>=2){std::fill(work,work+21,0.0);std::copy(q.c,q.c+7,work);degree=d;std::cout<<DeflateComplex(work,degree,q.z,q.tolerance)<<' '<<degree<<' ';for(int i=0;i<=degree;++i)std::cout<<bits(work[i])<<' ';}
 double root=q.x;std::cout<<HasMultiplicity(q.c,d,root,size_t(q.multiplicity))<<' '<<bits(root)<<' ';
 std::complex<double> croot=q.z;std::cout<<HasComplexMultiplicity(q.c,d,croot,size_t(q.multiplicity))<<' ';mpu_z(croot);
 MathPoly::PolyResult reals;reals.NbRoots=size_t(q.nbReal);for(int i=0;i<q.nbReal;++i){reals.Roots[size_t(i)]=q.real[i];reals.Multiplicities[size_t(i)]=1;}
 ConsolidateRealRoots(reals,q.tolerance,q.c,d,q.exponent);mpu_result(reals);
 MathPoly::PolyResult complexes;complexes.NbComplexRoots=size_t(2*q.nbComplex);
 for(int i=0;i<q.nbComplex;++i){complexes.ComplexRoots[size_t(2*i)]=q.complexRoots[i];complexes.ComplexRoots[size_t(2*i+1)]=std::conj(q.complexRoots[i]);}
 ConsolidateComplexRoots(complexes,q.tolerance,q.c,d,q.exponent);mpu_result(complexes);
 std::cout<<IsFinite(q.z)<<' '<<THE_LAGUERRE_NB_STARTS<<' '<<THE_MULTIPLICITY_REFINEMENT_MAX_ITERATIONS<<' '<<THE_DISTINCT_ROOT_BISECTION_ITERATIONS<<' ';
 mpu_ld(THE_DISTINCT_ROOT_RESIDUAL_TOLERANCE);mpu_ld(THE_ROOT_MULTIPLICITY_TOLERANCE);std::cout<<'\n';
}
