// Modern eigenanalysis oracle at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathLin_EigenSearch.hxx>
#include <MathLin_Jacobi.hxx>
void modern_eigen_vectors(const std::optional<math_Vector>& v,const std::optional<math_Matrix>& m) {
 std::cout<<v.has_value()<<' ';if(v)for(size_t i=0;i<v->Size();++i)std::cout<<bits(v->At(i))<<' ';
 std::cout<<m.has_value()<<' ';if(m)for(size_t i=0;i<m->RowSize();++i)for(size_t j=0;j<m->ColSize();++j)std::cout<<bits(m->At(i,j))<<' ';
}
void modern_eigen_case(std::istream& input) {
 int n,lower,limit,mode;input>>n>>lower>>limit>>mode;
 math_Vector d(lower,lower+n-1),e(lower,lower+n-1);for(int i=0;i<n;++i)input>>d(lower+i);for(int i=0;i<n;++i)input>>e(lower+i);
 if(mode==1)d.ChangeAt(0)=std::numeric_limits<double>::infinity();
 math_Matrix a(lower,lower+n-1,lower+2,lower+n+1,0.0);
 for(int i=0;i<n;++i){a.ChangeAt(i,i)=d.At(i);if(i+1<n){a.ChangeAt(i,i+1)=e.At(i+1);a.ChangeAt(i+1,i)=e.At(i+1);}}
 if(mode==2&&n>1)a.ChangeAt(1,0)+=1;
 auto r=MathLin::EigenTridiagonal(d,e,limit);std::cout<<int(r.Status)<<' '<<r.Dimension<<' ';modern_eigen_vectors(r.EigenValues,r.EigenVectors);
 for(bool sort:{false,true}){auto j=MathLin::Jacobi(a,sort);std::cout<<int(j.Status)<<' '<<j.NbIterations<<' ';modern_eigen_vectors(j.EigenValues,j.EigenVectors);}
 for(double power:{0.5,-0.5,2.0,-3.0})modern_eigen_vectors({},MathLin::MatrixPower(a,power));
 std::cout<<'\n';
}

#include <MathLin_Gauss.hxx>
void modern_lu_optional(std::optional<double> value) {std::cout<<value.has_value()<<' '<<bits(value.value_or(91))<<' ';}
void modern_lu_case(std::istream& input) {
 int n,lower,mode;double tol;input>>n>>lower>>mode>>tol;
 math_Matrix a(lower,lower+n-1,lower+2,lower+n+1);math_Vector b(lower,lower+n-1);
 for(int i=0;i<n;++i)for(int j=0;j<n;++j)input>>a.ChangeAt(i,j);
 for(int i=0;i<n;++i)input>>b.ChangeAt(i);
 if(mode==1)a.ChangeAt(0,0)=std::numeric_limits<double>::infinity();
 if(mode==2)b.ChangeAt(0)=std::numeric_limits<double>::quiet_NaN();
 auto r=MathLin::LU(a,tol);std::cout<<int(r.Status)<<' '<<r.Sign<<' '<<r.RowLower<<' '<<r.ColLower<<' '<<r.Dimension<<' '<<bits(r.MinAbsPivot)<<' '<<bits(r.MaxAbsPivot)<<' ';
 modern_lu_optional(r.Determinant);modern_lu_optional(r.ConditionEstimate);modern_eigen_vectors({},r.LU);
 std::cout<<r.Pivot.has_value()<<' ';if(r.Pivot)for(size_t i=0;i<r.Pivot->Size();++i)std::cout<<r.Pivot->At(i)<<' ';
 auto s=MathLin::Solve(a,b,tol);std::cout<<int(s.Status)<<' ';modern_lu_optional(s.Determinant);modern_eigen_vectors(s.Solution,{});
 auto inv=MathLin::Invert(a,tol);std::cout<<int(inv.Status)<<' ';modern_lu_optional(inv.Determinant);modern_eigen_vectors({},inv.Inverse);
 auto det=MathLin::Determinant(a,tol);std::cout<<int(det.Status)<<' ';modern_lu_optional(det.Determinant);
 math_Matrix rhs(-3,n-4,2,3);for(int i=0;i<n;++i){rhs.ChangeAt(i,0)=b.At(i);rhs.ChangeAt(i,1)=i;}
 auto multiple=MathLin::SolveMultiple(a,rhs,tol);std::cout<<int(multiple.Status)<<' ';modern_lu_optional(multiple.Determinant);modern_eigen_vectors({},multiple.Solutions);std::cout<<std::endl;
}
