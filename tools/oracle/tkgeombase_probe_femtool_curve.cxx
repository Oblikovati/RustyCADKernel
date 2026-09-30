// FEmTool_Curve and FEmTool_Assembly requests (tools/tkgeombase_corpus_femtool.py);
// the helpers are in tkgeombase_probe_femtool.cxx.

void gb_fem_array1(const NCollection_Array1<double>& a){gb_int(a.Lower());gb_int(a.Upper());for(int i=a.Lower();i<=a.Upper();++i)gb_real(a(i));}
// Curve commands: 1 SetElement, 2 GetElement, 3 GetPolynom, 4 D0/D1/D2, 5 Length,
// 6 SetDegree, 7 ReduceDegree, 8 Knots()(i) = v, 9 Degree.
void gb_fem_curve_command(GbReader& r,const occ::handle<FEmTool_Curve>& c){
 const int code=r.integer();
 if(code==1){const int el=r.integer();NCollection_Array2<double> a=gb_fem_read_array2(r);c->SetElement(el,a);}
 else if(code==2){const int el=r.integer(),rl=r.integer(),cl=r.integer(),rows=r.integer(),cols=r.integer();
  NCollection_Array2<double> a(rl,rl+rows-1,cl,cl+cols-1);a.Init(-7.);c->GetElement(el,a);
  for(int i=a.LowerRow();i<=a.UpperRow();++i)for(int j=a.LowerCol();j<=a.UpperCol();++j)gb_real(a(i,j));}
 else if(code==3){const int lower=r.integer(),length=r.integer();NCollection_Array1<double> a(lower,lower+length-1);
  a.Init(-7.);c->GetPolynom(a);gb_fem_array1(a);}
 else if(code==4){const int order=r.integer();const double u=r.real();const int lower=r.integer(),length=r.integer();
  NCollection_Array1<double> a(lower,lower+length-1);a.Init(-7.);
  if(order==0)c->D0(u,a);else if(order==1)c->D1(u,a);else c->D2(u,a);gb_fem_array1(a);}
 else if(code==5){const double a=r.real(),b=r.real();double l=-7.;c->Length(a,b,l);gb_real(l);}
 else if(code==6){const int el=r.integer();c->SetDegree(el,r.integer());}
 else if(code==7){const int el=r.integer();const double tol=r.real();int d=-7;double e=-7.;c->ReduceDegree(el,tol,d,e);gb_int(d);gb_real(e);}
 else if(code==8){const int i=r.integer();c->Knots()(i)=r.real();}
 else gb_int(c->Degree(r.integer()));
}
void gb_fem_curve_case(std::istream& in){
 GbReader r{in};const int dim=r.integer(),nb=r.integer(),wd=r.integer(),shape=r.integer();
 occ::handle<FEmTool_Curve> c;
 std::vector<double> knots(nb+1);for(double& k:knots)k=r.real();
 const int count=r.integer();
 try{c=new FEmTool_Curve(dim,nb,PLib_HermitJacobi(wd,gb_fem_shape(shape)),1.e-7);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';gb_end();return;}
 // The native knots and coefficients start uninitialized; the port zeroes them.
 NCollection_Array1<double>& k=c->Knots();for(int i=0;i<=nb;++i)k(i+1)=knots[i];
 for(int i=1;i<=nb;++i){NCollection_Array2<double> zero(0,wd,1,dim);zero.Init(0.);c->SetElement(i,zero);}
 for(int n=0;n<count;++n){try{gb_fem_curve_command(r,c);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
 gb_end();
}
// Assembly commands: 1 AddMatrix, 2 AddVector, 3 AddConstraint, 4 Solve, 5 Solution,
// 6 NullifyMatrix, 7 NullifyVector, 8 ResetConstraint, 9 NullifyConstraint, 10 NbGlobVar.
void gb_fem_assembly_command(GbReader& r,FEmTool_Assembly& a){
 const int code=r.integer();
 if(code==1){const int el=r.integer(),d1=r.integer(),d2=r.integer();NCollection_Array2<double> v=gb_fem_read_array2(r);
  math_Matrix m(v.LowerRow(),v.UpperRow(),v.LowerCol(),v.UpperCol());
  for(int i=v.LowerRow();i<=v.UpperRow();++i)for(int j=v.LowerCol();j<=v.UpperCol();++j)m(i,j)=v(i,j);
  a.AddMatrix(el,d1,d2,m);}
 else if(code==2){const int el=r.integer(),d=r.integer();math_Vector v=gb_fem_read_vector(r);a.AddVector(el,d,v);}
 else if(code==3){const int idx=r.integer(),el=r.integer(),d=r.integer();math_Vector v=gb_fem_read_vector(r);a.AddConstraint(idx,el,d,v,r.real());}
 else if(code==4)gb_flag(a.Solve());
 else if(code==5){const int lower=r.integer();math_Vector x(lower,lower+a.NbGlobVar()-1,-7.);a.Solution(x);gb_fem_vector(x);}
 else if(code==6)a.NullifyMatrix();
 else if(code==7)a.NullifyVector();
 else if(code==8)a.ResetConstraint();
 else if(code==9)a.NullifyConstraint();
 else gb_int(a.NbGlobVar());
}
// `rows cols dependence... ; trows tcols (length indexes...) per cell ; commands`.
void gb_fem_assembly_case(std::istream& in){
 GbReader r{in};const int dr=r.integer(),dc=r.integer();NCollection_Array2<int> dep(1,dr,1,dc);
 for(int i=1;i<=dr;++i)for(int j=1;j<=dc;++j)dep(i,j)=r.integer();
 const int tr=r.integer(),tc=r.integer();
 occ::handle<NCollection_HArray2<occ::handle<NCollection_HArray1<int>>>> table=
  new NCollection_HArray2<occ::handle<NCollection_HArray1<int>>>(1,tr,1,tc);
 for(int i=1;i<=tr;++i)for(int j=1;j<=tc;++j){const int lower=r.integer(),length=r.integer();
  occ::handle<NCollection_HArray1<int>> cell=new NCollection_HArray1<int>(lower,lower+length-1);
  for(int k=0;k<length;++k)cell->SetValue(lower+k,r.integer());table->SetValue(i,j,cell);}
 const int count=r.integer();
 try{
  FEmTool_Assembly a(dep,table);
  for(int n=0;n<count;++n){try{gb_fem_assembly_command(r,a);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
