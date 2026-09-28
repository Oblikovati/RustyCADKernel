// PLib, PLib_JacobiPolynomial and PLib_HermitJacobi protocols at pinned OCCT 3d097a0.
#include <PLib.hxx>
#include <PLib_HermitJacobi.hxx>
#include <PLib_JacobiPolynomial.hxx>
#include <math_Matrix.hxx>
#include <vector>
static std::vector<double> plib_read(std::istream& in,int n){std::vector<double> v(n>0?n:0);for(auto& x:v)in>>x;return v;}
static void plib_out(const double* v,int n){for(int i=0;i<n;++i)std::cout<<bits(v[i])<<' ';}
static void plib_array(const NCollection_Array1<double>& a){for(int i=a.Lower();i<=a.Upper();++i)std::cout<<bits(a(i))<<' ';}
static void plib_fail(const Standard_Failure& f){std::cout<<f.ExceptionType()<<' ';}
static void plib_poly_case(std::istream& in){
 double u;int dr,deg,dim;in>>u>>dr>>deg>>dim;auto c=plib_read(in,(deg+1)*dim);
 std::vector<double> r((dr+1)*dim+1,91.0);PLib::EvalPolynomial(u,dr,deg,dim,c[0],r[0]);plib_out(r.data(),(int)r.size());
 std::vector<double> v(dim+1,91.0);PLib::NoDerivativeEvalPolynomial(u,deg,dim,deg*dim,c[0],v[0]);plib_out(v.data(),dim+1);std::cout<<'\n';
}
static void plib_poly2_case(std::istream& in){
 double u,v;int udr,vdr,udeg,vdeg,dim;in>>u>>v>>udr>>vdr>>udeg>>vdeg>>dim;auto c=plib_read(in,(udeg+1)*(vdeg+1)*dim);
 std::vector<double> r(dim+1,91.0);PLib::EvalPoly2Var(u,v,udr,vdr,udeg,vdeg,dim,c[0],r[0]);plib_out(r.data(),dim+1);std::cout<<'\n';
}
static void plib_rational_case(std::istream& in){
 int deg,dr,dim,all;in>>deg>>dr>>dim>>all;int rows=std::max(std::min(dr,deg)+1,1);auto d=plib_read(in,rows*(dim+1));
 std::vector<double> r(all?(dr+1)*dim+1:dim+1,91.0);PLib::RationalDerivative(deg,dr,dim,d[0],r[0],all!=0);plib_out(r.data(),(int)r.size());
 std::vector<double> p((dr+1)*dim+1,0.0),w(dr+1,0.0);
 for(int i=0;i<=dr;++i){w[i]=i<rows?d[i*(dim+1)+dim]:0.0;for(int k=0;k<dim;++k)p[i*dim+k]=i<rows?d[i*(dim+1)+k]:0.0;}
 std::vector<double> q((dr+1)*dim+1,91.0);PLib::RationalDerivatives(dr,dim,p[0],w[0],q[0]);plib_out(q.data(),(int)q.size());std::cout<<'\n';
}
static void plib_lagrange_case(std::istream& in){
 double u;int dr,deg,dim;in>>u>>dr>>deg>>dim;auto v=plib_read(in,(deg+1)*dim);auto t=plib_read(in,deg+1);
 std::vector<double> r((std::max(std::min(dr,deg),0)+1)*dim+1,91.0);int code=PLib::EvalLagrange(u,dr,deg,dim,v[0],t[0],r[0]);
 std::cout<<code<<' ';plib_out(r.data(),(int)r.size());std::cout<<'\n';
}
static void plib_cubic_case(std::istream& in){
 double u;int dr,dim;in>>u>>dr>>dim;auto v=plib_read(in,2*dim);auto d=plib_read(in,2*dim);auto t=plib_read(in,2);
 std::vector<double> r((std::max(std::min(dr,3),0)+1)*dim+1,91.0);int code=PLib::EvalCubicHermite(u,dr,dim,v[0],d[0],t[0],r[0]);
 std::cout<<code<<' ';plib_out(r.data(),(int)r.size());std::cout<<'\n';
}
static void plib_hermite_coeffs_case(std::istream& in){
 double first,last;int fo,lo;in>>first>>last>>fo>>lo;int n=fo+lo+2;math_Matrix m(1,n,1,n,91.0);
 try{bool ok=PLib::HermiteCoefficients(first,last,fo,lo,m);std::cout<<ok<<' ';for(int i=1;i<=n;++i)for(int j=1;j<=n;++j)std::cout<<bits(m(i,j))<<' ';}catch(const Standard_Failure& f){plib_fail(f);}std::cout<<'\n';
}
static void plib_hermite_interp_case(std::istream& in){
 int dim,fo,lo;double first,last;in>>dim>>first>>last>>fo>>lo;
 NCollection_Array2<double> fc(1,dim,0,fo),lc(1,dim,0,lo);for(int i=1;i<=dim;++i)for(int j=0;j<=fo;++j)in>>fc(i,j);
 for(int i=1;i<=dim;++i)for(int j=0;j<=lo;++j)in>>lc(i,j);NCollection_Array1<double> c(0,(fo+lo+2)*dim-1);c.Init(91.0);
 try{bool ok=PLib::HermiteInterpolate(dim,first,last,fo,lo,fc,lc,c);std::cout<<ok<<' ';plib_array(c);}catch(const Standard_Failure& f){plib_fail(f);}std::cout<<'\n';
}
template<class Base> static void plib_point_overloads(int dim,int lower,const std::vector<double>& c,const std::vector<double>* w,bool trim,double u1,double u2){
 int n=(int)c.size()/dim;NCollection_Array1<Base> coefs(lower,lower+n-1),poles(lower,lower+n-1);
 for(int i=0;i<n;++i)for(int k=1;k<=dim;++k)coefs(lower+i).SetCoord(k,c[i*dim+k-1]);
 NCollection_Array1<double> wc(lower,lower+n-1),wp(lower,lower+n-1);wp.Init(91.0);if(w)for(int i=0;i<n;++i)wc(lower+i)=(*w)[i];
 try{if(trim){PLib::Trimming(u1,u2,coefs,w?&wc:PLib::NoWeights());poles=coefs;wp=wc;}else PLib::CoefficientsPoles(coefs,w?&wc:PLib::NoWeights(),poles,w?&wp:PLib::NoWeights());
  for(int i=poles.Lower();i<=poles.Upper();++i)for(int k=1;k<=dim;++k)std::cout<<bits(poles(i).Coord(k))<<' ';if(w)plib_array(wp);}catch(const Standard_Failure& f){plib_fail(f);}
}
static void plib_curve_case(std::istream& in,bool trim){
 int dim,rat,n,lower,wlen;double u1,u2;in>>dim>>rat>>n>>lower>>wlen>>u1>>u2;auto c=plib_read(in,n*dim);auto w=plib_read(in,wlen);
 NCollection_Array1<double> coefs(lower,lower+n*dim-1),poles(lower,lower+n*dim-1),wc(lower,lower+wlen-1),wp(lower,lower+wlen-1);poles.Init(91.0);wp.Init(91.0);
 for(int i=0;i<n*dim;++i)coefs(lower+i)=c[i];for(int i=0;i<wlen;++i)wc(lower+i)=w[i];
 try{if(trim){PLib::Trimming(u1,u2,dim,coefs,rat?&wc:PLib::NoWeights());plib_array(coefs);if(rat)plib_array(wc);}
  else{PLib::CoefficientsPoles(dim,coefs,rat?&wc:PLib::NoWeights(),poles,rat?&wp:PLib::NoWeights());plib_array(poles);if(rat)plib_array(wp);}}catch(const Standard_Failure& f){plib_fail(f);}
 std::cout<<"| ";if(wlen==n||!rat){const std::vector<double>* wv=rat?&w:nullptr;
  if(dim==3)plib_point_overloads<gp_Pnt>(3,lower,c,wv,trim,u1,u2);if(dim==2)plib_point_overloads<gp_Pnt2d>(2,lower,c,wv,trim,u1,u2);
  if(dim==1){NCollection_Array1<double> a(lower,lower+n-1),b(lower,lower+n-1),x(lower,lower+n-1),y(lower,lower+n-1);b.Init(91.0);y.Init(91.0);for(int i=0;i<n;++i){a(lower+i)=c[i];x(lower+i)=rat?w[i]:0.0;}
   try{if(trim){PLib::Trimming(u1,u2,a,rat?&x:PLib::NoWeights());plib_array(a);if(rat)plib_array(x);}else{PLib::CoefficientsPoles(a,rat?&x:PLib::NoWeights(),b,rat?&y:PLib::NoWeights());plib_array(b);if(rat)plib_array(y);}}catch(const Standard_Failure& f){plib_fail(f);}}}
 std::cout<<'\n';
}
static void plib_grid_out(const NCollection_Array2<gp_Pnt>& p,const NCollection_Array2<double>& w,bool rat){
 for(int i=p.LowerRow();i<=p.UpperRow();++i)for(int j=p.LowerCol();j<=p.UpperCol();++j)xyz(p(i,j).XYZ());if(rat)for(int i=w.LowerRow();i<=w.UpperRow();++i)for(int j=w.LowerCol();j<=w.UpperCol();++j)std::cout<<bits(w(i,j))<<' ';
}
static void plib_grid_case(std::istream& in){
 int rows,cols,rat,lr,lc;double u1,u2;in>>rows>>cols>>rat>>lr>>lc>>u1>>u2;
 NCollection_Array2<gp_Pnt> c(lr,lr+rows-1,lc,lc+cols-1),p(lr,lr+rows-1,lc,lc+cols-1);NCollection_Array2<double> wc(lr,lr+rows-1,lc,lc+cols-1),wp(lr,lr+rows-1,lc,lc+cols-1);wp.Init(91.0);
 for(int i=0;i<rows;++i)for(int j=0;j<cols;++j){double x,y,z;in>>x>>y>>z;c(lr+i,lc+j)=gp_Pnt(x,y,z);}
 for(int i=0;i<rows;++i)for(int j=0;j<cols;++j)in>>wc(lr+i,lc+j);
 try{PLib::CoefficientsPoles(c,rat?&wc:PLib::NoWeights2(),p,rat?&wp:PLib::NoWeights2());plib_grid_out(p,wp,rat);}catch(const Standard_Failure& f){plib_fail(f);}
 auto a=c;auto aw=wc;try{PLib::UTrimming(u1,u2,a,rat?&aw:PLib::NoWeights2());plib_grid_out(a,aw,rat);}catch(const Standard_Failure& f){plib_fail(f);}
 a=c;aw=wc;try{PLib::VTrimming(u1,u2,a,rat?&aw:PLib::NoWeights2());plib_grid_out(a,aw,rat);}catch(const Standard_Failure& f){plib_fail(f);}std::cout<<'\n';
}
static void plib_params_case(std::istream& in){
 int shape,maxdeg,code,niv,n,p;in>>shape>>maxdeg>>code>>niv>>n>>p;int points=-7,degree=-7;
 try{PLib::JacobiParameters(GeomAbs_Shape(shape),maxdeg,code,points,degree);}catch(const Standard_Failure& f){plib_fail(f);}std::cout<<points<<' '<<degree<<' ';
 try{std::cout<<PLib::NivConstr(GeomAbs_Shape(shape))<<' ';}catch(const Standard_Failure& f){plib_fail(f);}
 try{std::cout<<int(PLib::ConstraintOrder(niv))<<' ';}catch(const Standard_Failure& f){plib_fail(f);}
 try{std::cout<<bits(PLib::Bin(n,p))<<' ';}catch(const Standard_Failure& f){plib_fail(f);}std::cout<<'\n';
}
static void plib_length_case(std::istream& in){
 int deg,dim;double u1,u2,tol;in>>deg>>dim>>u1>>u2>>tol;auto c=plib_read(in,(deg+1)*dim);double length=91,error=91;
 PLib::EvalLength(deg,dim,c[0],u1,u2,length);std::cout<<bits(length)<<' ';PLib::EvalLength(deg,dim,c[0],u1,u2,tol,length,error);std::cout<<bits(length)<<' '<<bits(error)<<'\n';
}
template<class B> static void plib_basis(const B& b,double u,int upper){
 NCollection_Array1<double> v0(0,upper),v1(0,upper),v2(0,upper),v3(0,upper);
 b.D0(u,v0);plib_array(v0);b.D1(u,v0,v1);plib_array(v0);plib_array(v1);b.D2(u,v0,v1,v2);plib_array(v2);b.D3(u,v0,v1,v2,v3);plib_array(v0);plib_array(v1);plib_array(v2);plib_array(v3);
}
template<class B> static void plib_errors(const B& b,int dim,int newdeg,int maxdeg,double tol,std::vector<double> c){
 try{std::cout<<bits(b.MaxError(dim,c[0],newdeg))<<' ';}catch(const Standard_Failure& f){plib_fail(f);}
 int nd=-7;double me=91;try{b.ReduceDegree(dim,maxdeg,tol,c[0],nd,me);std::cout<<nd<<' '<<bits(me)<<' ';}catch(const Standard_Failure& f){plib_fail(f);}
 try{std::cout<<bits(b.AverageError(dim,c[0],newdeg))<<' ';}catch(const Standard_Failure& f){plib_fail(f);}
}
template<class B> static void plib_convert(const B& b,int dim,int todeg,const std::vector<double>& c){
 NCollection_Array1<double> j(0,(todeg+1)*dim-1),r(0,(todeg+1)*dim-1);r.Init(91.0);for(int i=0;i<(todeg+1)*dim;++i)j(i)=c[i];
 try{b.ToCoefficients(dim,todeg,j,r);plib_array(r);}catch(const Standard_Failure& f){plib_fail(f);}
}
static void plib_jacobi_case(std::istream& in,bool hermit){
 int work,shape,nb,dim,newdeg,maxdeg,todeg;double u,tol;in>>work>>shape>>nb>>u>>dim>>newdeg>>maxdeg>>tol>>todeg;auto c=plib_read(in,(std::max(work,0)+1)*dim);
 try{if(hermit){PLib_HermitJacobi b(work,GeomAbs_Shape(shape));std::cout<<b.WorkDegree()<<' '<<b.NivConstr()<<' ';plib_errors(b,dim,newdeg,maxdeg,tol,c);plib_convert(b,dim,todeg,c);plib_basis(b,u,work);}
  else{PLib_JacobiPolynomial b(work,GeomAbs_Shape(shape));int degree=work-2*(b.NivConstr()+1);std::cout<<b.WorkDegree()<<' '<<b.NivConstr()<<' ';
   NCollection_Array1<double> pts(0,std::max(nb/2,0));pts.Init(91.0);try{b.Points(nb,pts);plib_array(pts);}catch(const Standard_Failure& f){plib_fail(f);}
   NCollection_Array2<double> wts(0,std::max(nb/2,0),0,degree);b.Weights(nb,wts);for(int i=wts.LowerRow();i<=wts.UpperRow();++i)for(int k=wts.LowerCol();k<=wts.UpperCol();++k)std::cout<<bits(wts(i,k))<<' ';
   NCollection_Array1<double> mx(0,degree);b.MaxValue(mx);plib_array(mx);plib_errors(b,dim,newdeg,maxdeg,tol,c);plib_convert(b,dim,todeg,c);plib_basis(b,u,degree);}}
 catch(const Standard_Failure& f){plib_fail(f);}std::cout<<'\n';
}
