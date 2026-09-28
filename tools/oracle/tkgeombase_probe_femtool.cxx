// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

// FEmTool requests (tools/tkgeombase_corpus_femtool.py). Each request builds one
// object, then runs a list of commands `code args...` on it; every command prints
// its results (or the exception type), so call orders and cached state compare too.
#include <FEmTool_Assembly.hxx>
#include <FEmTool_Curve.hxx>
#include <FEmTool_ElementsOfRefMatrix.hxx>
#include <FEmTool_LinearFlexion.hxx>
#include <FEmTool_LinearJerk.hxx>
#include <FEmTool_LinearTension.hxx>
#include <FEmTool_ProfileMatrix.hxx>
#include <GeomAbs_Shape.hxx>
#include <math_Matrix.hxx>
#include <math_Vector.hxx>
#include <PLib_HermitJacobi.hxx>
GeomAbs_Shape gb_fem_shape(int code){
 static const GeomAbs_Shape shapes[]={GeomAbs_C0,GeomAbs_C1,GeomAbs_C2,GeomAbs_G1,GeomAbs_C3};
 return shapes[code];
}
void gb_fem_vector(const math_Vector& v){gb_int(v.Lower());gb_int(v.Upper());for(int i=v.Lower();i<=v.Upper();++i)gb_real(v(i));}
void gb_fem_matrix(const math_Matrix& m){
 gb_int(m.LowerRow());gb_int(m.UpperRow());gb_int(m.LowerCol());gb_int(m.UpperCol());
 for(int i=m.LowerRow();i<=m.UpperRow();++i)for(int j=m.LowerCol();j<=m.UpperCol();++j)gb_real(m(i,j));
}
// `lower length values...` as a math_Vector.
math_Vector gb_fem_read_vector(GbReader& r){
 const int lower=r.integer(),length=r.integer();math_Vector v(lower,lower+length-1);
 for(int i=0;i<length;++i)v(lower+i)=r.real();return v;
}
// `rowLower colLower rows cols values...` (row by row) as a 2D array.
NCollection_Array2<double> gb_fem_read_array2(GbReader& r){
 const int rl=r.integer(),cl=r.integer(),rows=r.integer(),cols=r.integer();
 NCollection_Array2<double> a(rl,rl+rows-1,cl,cl+cols-1);
 for(int i=0;i<rows;++i)for(int j=0;j<cols;++j)a(rl+i,cl+j)=r.real();return a;
}
// The text a member prints on std::cout, one token stream with '|' for newlines.
template<class F> void gb_fem_captured(F print){
 std::ostringstream text;std::streambuf* saved=std::cout.rdbuf(text.rdbuf());
 try{print();}catch(...){std::cout.rdbuf(saved);throw;}
 std::cout.rdbuf(saved);
 for(char c:text.str())std::cout<<(c=='\n'?'|':c);std::cout<<' ';
}
void gb_fem_elements_case(std::istream& in){
 GbReader r{in};const int wd=r.integer(),shape=r.integer(),der=r.integer();const double u=r.real();const int flen=r.integer();
 try{
  PLib_HermitJacobi base(wd,gb_fem_shape(shape));
  FEmTool_ElementsOfRefMatrix e(base,der);gb_int(e.NbVariables());gb_int(e.NbEquations());
  math_Vector x(1,1,u),f(1,flen,0.);gb_flag(e.Value(x,f));gb_fem_vector(f);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// Criterion commands: 1 DependenceTable, 2 Value, 3 Hessian, 4 Gradient, 5 Set(first, last).
void gb_fem_criterion_command(GbReader& r,const occ::handle<FEmTool_ElementaryCriterion>& c){
 const int code=r.integer();
 if(code==1){occ::handle<NCollection_HArray2<int>> t=c->DependenceTable();
  gb_int(t->LowerRow());gb_int(t->UpperRow());gb_int(t->LowerCol());gb_int(t->UpperCol());
  for(int i=t->LowerRow();i<=t->UpperRow();++i)for(int j=t->LowerCol();j<=t->UpperCol();++j)gb_int(t->Value(i,j));}
 else if(code==2)gb_real(c->Value());
 else if(code==3){const int d1=r.integer(),d2=r.integer(),rl=r.integer(),ru=r.integer(),cl=r.integer(),cu=r.integer();
  math_Matrix h(rl,ru,cl,cu,-7.);c->Hessian(d1,d2,h);gb_fem_matrix(h);}
 else if(code==4){const int d=r.integer(),gl=r.integer(),gu=r.integer();math_Vector g(gl,gu,-7.);c->Gradient(d,g);gb_fem_vector(g);}
 else{const double a=r.real(),b=r.real();c->Set(a,b);}
}
void gb_fem_criterion_case(std::istream& in){
 GbReader r{in};const int kind=r.integer(),wd=r.integer(),shape=r.integer(),with_coeff=r.integer();
 occ::handle<NCollection_HArray2<double>> coeff;
 if(with_coeff){coeff=new NCollection_HArray2<double>(gb_fem_read_array2(r));}
 const double first=r.real(),last=r.real();const int count=r.integer();
 occ::handle<FEmTool_ElementaryCriterion> c;
 try{
  if(kind==0)c=new FEmTool_LinearFlexion(wd,gb_fem_shape(shape));
  else if(kind==1)c=new FEmTool_LinearJerk(wd,gb_fem_shape(shape));
  else c=new FEmTool_LinearTension(wd,gb_fem_shape(shape));
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';gb_end();return;}
 if(with_coeff)c->Set(coeff);
 c->Set(first,last);
 for(int k=0;k<count;++k){try{gb_fem_criterion_command(r,c);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
 gb_end();
}
// Profile commands: 1 Init, 2 ChangeValue +=, 3 Decompose, 4 Solve, 5 Multiplied,
// 6 IsInProfile, 7 OutM, 8 OutS, 9 Prepare, 10 iterative Solve, 11 Row/ColNumber.
void gb_fem_profile_command(GbReader& r,const occ::handle<FEmTool_ProfileMatrix>& m){
 const int code=r.integer();
 if(code==1)m->Init(r.real());
 else if(code==2){const int i=r.integer(),j=r.integer();m->ChangeValue(i,j)+=r.real();gb_real(m->ChangeValue(i,j));}
 else if(code==3)gb_flag(m->Decompose());
 else if(code==4||code==5){math_Vector b=gb_fem_read_vector(r);math_Vector x(1,m->RowNumber(),0.);
  if(code==4)m->Solve(b,x);else m->Multiplied(b,x);gb_fem_vector(x);}
 else if(code==6){const int i=r.integer();gb_flag(m->IsInProfile(i,r.integer()));}
 else if(code==7)gb_fem_captured([&]{m->OutM();});
 else if(code==8)gb_fem_captured([&]{m->OutS();});
 else if(code==9)gb_flag(m->Prepare());
 else if(code==10){math_Vector b(1,m->RowNumber(),1.),x(1,m->RowNumber(),0.),res(1,m->RowNumber(),0.);m->Solve(b,b,x,res);}
 else{gb_int(m->RowNumber());gb_int(m->ColNumber());}
}
void gb_fem_profile_case(std::istream& in){
 GbReader r{in};const int n=r.integer();NCollection_Array1<int> first(1,n);
 for(int i=1;i<=n;++i)first(i)=r.integer();const int count=r.integer();
 occ::handle<FEmTool_ProfileMatrix> m=new FEmTool_ProfileMatrix(first);
 m->Init(0.);
 for(int k=0;k<count;++k){try{gb_fem_profile_command(r,m);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
 gb_end();
}
