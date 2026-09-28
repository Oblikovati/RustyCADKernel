// Oracle coverage for MathLin factor and least-squares headers at pinned 3d097a0.
#include <MathLin_Crout.hxx>
#include <MathLin_Householder.hxx>
#include <MathLin_SVD.hxx>
#include <MathLin_LeastSquares.hxx>
void factor_linear(MathUtils::LinearResult r){std::cout<<int(r.Status)<<' ';modern_lu_optional(r.Determinant);modern_eigen_vectors(r.Solution,{});}
void factor_inverse(MathUtils::InverseResult r){std::cout<<int(r.Status)<<' ';modern_lu_optional(r.Determinant);modern_eigen_vectors({},r.Inverse);}
void factor_least(MathLin::LeastSquaresResult r){std::cout<<int(r.Status)<<' '<<r.Rank<<' ';modern_lu_optional(r.ConditionNumber);modern_lu_optional(r.Residual);modern_lu_optional(r.ResidualSq);modern_eigen_vectors(r.Solution,{});}
void factor_crout(const math_Matrix& a,const math_Vector& b,double tol){
 auto r=MathLin::Crout(a,tol);std::cout<<int(r.Status)<<' '<<bits(r.MinAbsPivot)<<' '<<bits(r.MaxAbsPivot)<<' ';
 modern_lu_optional(r.Determinant);modern_lu_optional(r.ConditionEstimate);modern_eigen_vectors(r.D,r.L);modern_eigen_vectors({},r.Inverse);
 factor_linear(MathLin::SolveCrout(a,b,tol));factor_inverse(MathLin::InvertCrout(a,tol));
}
void factor_qr(const math_Matrix& a,const math_Vector& b,double tol){
 auto r=MathLin::QR(a,tol);std::cout<<int(r.Status)<<' '<<r.Rank<<' '<<bits(r.DiagonalThreshold)<<' ';modern_lu_optional(r.ConditionEstimate);modern_eigen_vectors({},r.Q);modern_eigen_vectors({},r.R);
 factor_linear(MathLin::SolveQR(a,b,tol));
 math_Matrix rhs(-2,int(a.RowSize())-3,3,4);for(size_t i=0;i<a.RowSize();++i){rhs.ChangeAt(i,0)=b.At(i);rhs.ChangeAt(i,1)=i;}
 auto multi=MathLin::SolveQRMultiple(a,rhs,tol);std::cout<<int(multi.Status)<<' ';modern_eigen_vectors({},multi.Solutions);
}
void factor_svd(const math_Matrix& a,const math_Vector& b,double tol){
 auto r=MathLin::SVD(a,tol);std::cout<<int(r.Status)<<' '<<r.Rank<<' ';modern_lu_optional(r.ConditionNumber);modern_eigen_vectors(r.SingularValues,r.U);modern_eigen_vectors({},r.V);
 factor_linear(MathLin::SolveSVD(a,b,tol));factor_inverse(MathLin::PseudoInverse(a,tol));modern_lu_optional(MathLin::ConditionNumber(a));std::cout<<MathLin::NumericalRank(a,tol)<<' ';
}
void modern_factors_case(std::istream& input){
 int m,n,mode;double tol,lambda;input>>m>>n>>mode>>tol>>lambda;
 math_Matrix a(-2,m-3,4,n+3);math_Vector b(5,m+4);
 for(int i=0;i<m;++i)for(int j=0;j<n;++j)input>>a.ChangeAt(i,j);
 for(int i=0;i<m;++i)input>>b.ChangeAt(i);
 if(mode==1)a.ChangeAt(0,0)=std::numeric_limits<double>::infinity();
 if(mode==2)b.ChangeAt(0)=std::numeric_limits<double>::quiet_NaN();
 if(mode==3&&n>1)a.ChangeAt(0,1)=std::numeric_limits<double>::quiet_NaN();
 factor_crout(a,b,tol);factor_qr(a,b,tol);factor_svd(a,b,tol);
 math_Vector w(size_t(m),1.0);w[0]=mode==4?0.0:2.0;
 for(auto method:{MathLin::LeastSquaresMethod::NormalEquations,MathLin::LeastSquaresMethod::QR,MathLin::LeastSquaresMethod::SVD,static_cast<MathLin::LeastSquaresMethod>(4)}){
 factor_least(MathLin::LeastSquares(a,b,method,tol));factor_least(MathLin::WeightedLeastSquares(a,b,w,method,tol));
 }
 factor_least(MathLin::RegularizedLeastSquares(a,b,lambda,tol));modern_lu_optional(MathLin::OptimalRegularization(a,b,1e-10,1e2,20));std::cout<<std::endl;
}
