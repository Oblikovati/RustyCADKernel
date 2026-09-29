// Copyright (c) 1995-1999 Matra Datavision
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

// AppDef requests on the containers, the line tool, the Compute instantiation and
// AppDef_LinearCriteria (tools/tkgeombase_corpus_appdef.py); AppDef_Variational is
// in tkgeombase_probe_appdef_variational.cxx. The Rust twin is
// crates/tkgeombase/examples/gb_probe/app_def.
#include <AppDef_Compute.hxx>
#include <AppDef_LinearCriteria.hxx>
#include <AppDef_MultiLine.hxx>
#include <AppDef_MultiPointConstraint.hxx>
#include <AppDef_MyLineTool.hxx>
#include <FEmTool_Curve.hxx>
#include <PLib_HermitJacobi.hxx>
#define GB_AD_TRY(...) try{__VA_ARGS__;}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
template<class T> NCollection_Array1<T> gb_ad_array(int lower,const std::vector<T>& v){
 NCollection_Array1<T> a(lower,lower+int(v.size())-1);for(int i=0;i<int(v.size());++i)a(lower+i)=v[i];return a;
}
// `kind lower n3 n2 nv3 nv2 nc3 nc2` then the points, tangents and curvatures: kind
// 0 (), 1 (n3, n2), 2 tabP, 3 tabP2d, 4 (tabP, tabP2d), 5 the six arrays, 6 the four
// point and tangent arrays, 7 (tabP, tabVec, tabCur), 8 (tabP, tabVec), 9 (tabP2d,
// tabVec2d), 10 (tabP2d, tabVec2d, tabCur2d); the arrays start at `lower`.
AppDef_MultiPointConstraint gb_ad_read_mpc(GbReader& r){
 const int kind=r.integer(),lower=r.integer(),n3=r.integer(),n2=r.integer(),nv3=r.integer(),nv2=r.integer(),nc3=r.integer(),nc2=r.integer();
 std::vector<gp_Pnt> p3(n3);std::vector<gp_Pnt2d> p2(n2);std::vector<gp_Vec> v3(nv3),c3(nc3);std::vector<gp_Vec2d> v2(nv2),c2(nc2);
 for(gp_Pnt& p:p3)p=r.pnt();
 for(gp_Pnt2d& p:p2)p=r.pnt2d();
 for(gp_Vec& v:v3)v=r.vec();
 for(gp_Vec2d& v:v2)v=r.vec2d();
 for(gp_Vec& v:c3)v=r.vec();
 for(gp_Vec2d& v:c2)v=r.vec2d();
 const auto P3=gb_ad_array(lower,p3);const auto P2=gb_ad_array(lower,p2);const auto V3=gb_ad_array(lower,v3);
 const auto V2=gb_ad_array(lower,v2);const auto C3=gb_ad_array(lower,c3);const auto C2=gb_ad_array(lower,c2);
 switch(kind){
  case 0:return AppDef_MultiPointConstraint();
  case 1:return AppDef_MultiPointConstraint(n3,n2);
  case 2:return AppDef_MultiPointConstraint(P3);
  case 3:return AppDef_MultiPointConstraint(P2);
  case 4:return AppDef_MultiPointConstraint(P3,P2);
  case 5:return AppDef_MultiPointConstraint(P3,P2,V3,V2,C3,C2);
  case 6:return AppDef_MultiPointConstraint(P3,P2,V3,V2);
  case 7:return AppDef_MultiPointConstraint(P3,V3,C3);
  case 8:return AppDef_MultiPointConstraint(P3,V3);
  case 9:return AppDef_MultiPointConstraint(P2,V2);
  default:return AppDef_MultiPointConstraint(P2,V2,C2);
 }
}
void gb_ad_mpc(const AppDef_MultiPointConstraint& m){
 gb_apc_point(m);gb_flag(m.IsTangencyPoint());gb_flag(m.IsCurvaturePoint());
}
// MultiPointConstraint commands: 1 SetTang, 2 Tang, 3 SetTang2d, 4 Tang2d, 5 SetCurv,
// 6 Curv, 7 SetCurv2d, 8 Curv2d, 9 the flags, 10 Dump, 11 a copy's SetTang then the
// original, 12 the points.
void gb_ad_mpc_command(GbReader& r,AppDef_MultiPointConstraint& m){
 const int code=r.integer();
 if(code==1){const int i=r.integer();m.SetTang(i,r.vec());}
 else if(code==2)gb_vec(m.Tang(r.integer()));
 else if(code==3){const int i=r.integer();m.SetTang2d(i,r.vec2d());}
 else if(code==4)gb_vec2d(m.Tang2d(r.integer()));
 else if(code==5){const int i=r.integer();m.SetCurv(i,r.vec());}
 else if(code==6)gb_vec(m.Curv(r.integer()));
 else if(code==7){const int i=r.integer();m.SetCurv2d(i,r.vec2d());}
 else if(code==8)gb_vec2d(m.Curv2d(r.integer()));
 else if(code==9){gb_flag(m.IsTangencyPoint());gb_flag(m.IsCurvaturePoint());}
 else if(code==10)gb_apc_dump(m);
 else if(code==11){AppDef_MultiPointConstraint copy=m;const int i=r.integer();const gp_Vec v=r.vec();
  GB_AD_TRY(copy.SetTang(i,v));gb_flag(m.IsTangencyPoint());GB_AD_TRY(gb_vec(m.Tang(i)));}
 else gb_ad_mpc(m);
}
// `ad_mpc point count commands...`.
void gb_ad_mpc_case(std::istream& in){
 GbReader r{in};
 try{
  AppDef_MultiPointConstraint m=gb_ad_read_mpc(r);gb_ad_mpc(m);const int count=r.integer();
  for(int k=0;k<count;++k){GB_AD_TRY(gb_ad_mpc_command(r,m));}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// A line: kind 0 AppDef_MultiLine(), 1 (NbMult) and SetValue of an AppDef line's
// points, 2 (tabMultiP), 3 (tabP3d), 4 (tabP2d); `lower` for the arrays of 2 to 4.
AppDef_MultiLine gb_ad_read_line(GbReader& r){
 const int kind=r.integer();
 if(kind==0)return AppDef_MultiLine();
 if(kind==1){const int n=r.integer();AppDef_MultiLine source=gb_apc_read_line(r);AppDef_MultiLine line(n);
  for(int i=1;i<=std::min(n,source.NbMultiPoints());++i)line.SetValue(i,source.Value(i));return line;}
 const int lower=r.integer();
 if(kind==2){AppDef_MultiLine source=gb_apc_read_line(r);std::vector<AppDef_MultiPointConstraint> v;
  for(int i=1;i<=source.NbMultiPoints();++i)v.push_back(source.Value(i));return AppDef_MultiLine(gb_ad_array(lower,v));}
 const int n=r.integer();
 if(kind==3){std::vector<gp_Pnt> v(n);for(gp_Pnt& p:v)p=r.pnt();return AppDef_MultiLine(gb_ad_array(lower,v));}
 std::vector<gp_Pnt2d> v(n);for(gp_Pnt2d& p:v)p=r.pnt2d();return AppDef_MultiLine(gb_ad_array(lower,v));
}
template<class T> void gb_ad_print(const NCollection_Array1<T>& a,void (*print)(const T&)){
 gb_int(a.Lower());gb_int(a.Upper());for(const T& v:a)print(v);
}
void gb_ad_pnt(const gp_Pnt& p){gb_pnt(p);}
void gb_ad_pnt2d(const gp_Pnt2d& p){gb_pnt2d(p);}
void gb_ad_vec(const gp_Vec& v){gb_vec(v);}
void gb_ad_vec2d(const gp_Vec2d& v){gb_vec2d(v);}
// The line tool reads `i overload lower n3 n2` into arrays of sentinels; overload 0
// the 3D array, 1 the 2D one, 2 both; kind 0 Value, 1 Tangency, 2 Curvature.
void gb_ad_tool_read(GbReader& r,const AppDef_MultiLine& line,int kind){
 const int i=r.integer(),overload=r.integer(),lower=r.integer(),n3=r.integer(),n2=r.integer();
 if(kind==0){NCollection_Array1<gp_Pnt> a(lower,lower+n3-1);a.Init(gp_Pnt(-7.,-7.,-7.));NCollection_Array1<gp_Pnt2d> b(lower,lower+n2-1);b.Init(gp_Pnt2d(-7.,-7.));
  if(overload==0)AppDef_MyLineTool::Value(line,i,a);else if(overload==1)AppDef_MyLineTool::Value(line,i,b);else AppDef_MyLineTool::Value(line,i,a,b);
  gb_ad_print(a,gb_ad_pnt);gb_ad_print(b,gb_ad_pnt2d);return;}
 NCollection_Array1<gp_Vec> a(lower,lower+n3-1);a.Init(gp_Vec(-7.,-7.,-7.));NCollection_Array1<gp_Vec2d> b(lower,lower+n2-1);b.Init(gp_Vec2d(-7.,-7.));
 bool found;
 if(kind==1)found=overload==0?AppDef_MyLineTool::Tangency(line,i,a):overload==1?AppDef_MyLineTool::Tangency(line,i,b):AppDef_MyLineTool::Tangency(line,i,a,b);
 else found=overload==0?AppDef_MyLineTool::Curvature(line,i,a):overload==1?AppDef_MyLineTool::Curvature(line,i,b):AppDef_MyLineTool::Curvature(line,i,a,b);
 gb_flag(found);gb_ad_print(a,gb_ad_vec);gb_ad_print(b,gb_ad_vec2d);
}
// Line commands: 1 NbMultiPoints, 2 NbPoints, 3 SetValue(i, an AppDef line's first
// point), 4 Value, 5 Dump, 6 FirstPoint, 7 LastPoint NbP3d NbP2d, 8 Value, 9
// Tangency, 10 Curvature, 11 WhatStatus, 12 MakeMLBetween, 13 MakeMLOneMorePoint,
// 14 a copy's SetValue then the original.
void gb_ad_line_command(GbReader& r,AppDef_MultiLine& line){
 const int code=r.integer();
 if(code==1)gb_int(line.NbMultiPoints());
 else if(code==2)gb_int(line.NbPoints());
 else if(code==3){const int i=r.integer();AppDef_MultiLine source=gb_apc_read_line(r);line.SetValue(i,source.Value(1));}
 else if(code==4)gb_ad_mpc(line.Value(r.integer()));
 else if(code==5)gb_apc_dump(line);
 else if(code==6)gb_int(AppDef_MyLineTool::FirstPoint(line));
 else if(code==7){gb_int(AppDef_MyLineTool::LastPoint(line));gb_int(AppDef_MyLineTool::NbP3d(line));gb_int(AppDef_MyLineTool::NbP2d(line));}
 else if(code<=10)gb_ad_tool_read(r,line,code-8);
 else if(code==11){const int i1=r.integer();gb_int(AppDef_MyLineTool::WhatStatus(line,i1,r.integer()));}
 else if(code==12){const int i1=r.integer(),i2=r.integer();AppDef_MultiLine other=AppDef_MyLineTool::MakeMLBetween(line,i1,i2,r.integer());
  gb_int(other.NbMultiPoints());}
 else if(code==13){const int i1=r.integer(),i2=r.integer();AppDef_MultiLine other;gb_flag(AppDef_MyLineTool::MakeMLOneMorePoint(line,i1,i2,r.integer(),other));}
 else{AppDef_MultiLine copy=line;const int i=r.integer();AppDef_MultiLine source=gb_apc_read_line(r);
  GB_AD_TRY(copy.SetValue(i,source.Value(1)));GB_AD_TRY(gb_ad_mpc(line.Value(i)));}
}
// `ad_line line count commands...`.
void gb_ad_line_case(std::istream& in){
 GbReader r{in};
 try{
  AppDef_MultiLine line=gb_ad_read_line(r);const int count=r.integer();
  for(int k=0;k<count;++k){GB_AD_TRY(gb_ad_line_command(r,line));}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// `ad_compute line variant options firstC lastC params repeat`: AppDef_Compute itself,
// with the variants of ax_compute.
void gb_ad_compute_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);const int variant=r.integer();const GbAxOptions o=gb_ax_read_options(r);
 const AppParCurves_Constraint fc=AppParCurves_Constraint(r.integer()),lc=AppParCurves_Constraint(r.integer());
 math_Vector u=gb_fem_read_vector(r);const bool repeat=r.flag();
 try{
  AppDef_Compute c=variant==0?AppDef_Compute(line,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.par,o.squares)
   :variant==1?AppDef_Compute(line,u,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.squares)
   :variant==2?AppDef_Compute(o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.par,o.squares)
   :AppDef_Compute(u,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.squares);
  if(variant>=2){c.SetConstraints(fc,lc);c.Perform(line);}
  gb_ax_compute_results(c);
  if(repeat){c.Init(o.degmin,o.degmax+1,o.tol3d*10.,o.tol2d*10.,o.iterations,o.cutting,o.par,o.squares);c.Perform(line);gb_ax_compute_results(c);}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// The FEmTool curve of a criteria request: `dim nbelm workdeg shape knots... then per
// element the (0..workdeg, 1..dim) coefficients`.
occ::handle<FEmTool_Curve> gb_ad_read_curve(GbReader& r){
 const int dim=r.integer(),nb=r.integer(),wd=r.integer(),shape=r.integer();
 std::vector<double> knots(nb+1),coeffs((nb)*(wd+1)*dim);for(double& v:knots)v=r.real();for(double& v:coeffs)v=r.real();
 occ::handle<FEmTool_Curve> c=new FEmTool_Curve(dim,nb,PLib_HermitJacobi(wd,gb_fem_shape(shape)),1.e-7);
 for(int i=1;i<=nb+1;++i)c->Knots()(i)=knots[i-1];
 int k=0;
 for(int el=1;el<=nb;++el){NCollection_Array2<double> a(0,wd,1,dim);for(int i=0;i<=wd;++i)for(int d=1;d<=dim;++d)a(i,d)=coeffs[k++];c->SetElement(el,a);}
 return c;
}
void gb_ad_table(const occ::handle<NCollection_HArray2<occ::handle<NCollection_HArray1<int>>>>& t){
 gb_int(t->LowerRow());gb_int(t->UpperRow());gb_int(t->LowerCol());gb_int(t->UpperCol());
 for(int i=t->LowerRow();i<=t->UpperRow();++i)for(int j=t->LowerCol();j<=t->UpperCol();++j){
  const occ::handle<NCollection_HArray1<int>>& a=t->Value(i,j);gb_int(a->Lower());gb_int(a->Upper());for(int v:a->Array1())gb_int(v);}
}
// Criteria commands: 1 SetParameters, 2 SetCurve(new curve), 3 SetCurve(the same
// curve), 4 SetEstimation, 5 GetEstimation, 6 EstLength() = v, 7 EstLength, 8
// AssemblyTable, 9 DependenceTable, 10 QualityValues, 11 ErrorValues, 12 Hessian,
// 13 Gradient, 14 InputVector then the curve elements, 15 SetWeight(5), 16
// GetWeight, 17 SetWeight(Array1), 18 GetCurve null flag.
void gb_ad_criteria_command(GbReader& r,const occ::handle<AppDef_SmoothCriterion>& j){
 const int code=r.integer();
 if(code==1){math_Vector v=gb_fem_read_vector(r);occ::handle<NCollection_HArray1<double>> p=new NCollection_HArray1<double>(v.Lower(),v.Upper());
  for(int i=v.Lower();i<=v.Upper();++i)p->SetValue(i,v(i));j->SetParameters(p);}
 else if(code==2)j->SetCurve(gb_ad_read_curve(r));
 else if(code==3)j->SetCurve(j->Curve());
 else if(code==4){const double a=r.real(),b=r.real();j->SetEstimation(a,b,r.real());}
 else if(code==5){double a,b,c;j->GetEstimation(a,b,c);gb_real(a);gb_real(b);gb_real(c);}
 else if(code==6)j->EstLength()=r.real();
 else if(code==7)gb_real(j->EstLength());
 else if(code==8)gb_ad_table(j->AssemblyTable());
 else if(code==9){occ::handle<NCollection_HArray2<int>> t=j->DependenceTable();
  gb_int(t->LowerRow());gb_int(t->UpperRow());for(int i=t->LowerRow();i<=t->UpperRow();++i)for(int k=t->LowerCol();k<=t->UpperCol();++k)gb_int(t->Value(i,k));}
 else if(code==10){const double a=r.real(),b=r.real(),c=r.real();double j1=-7.,j2=-7.,j3=-7.;gb_int(j->QualityValues(a,b,c,j1,j2,j3));
  gb_real(j1);gb_real(j2);gb_real(j3);double e1,e2,e3;j->GetEstimation(e1,e2,e3);gb_real(e1);gb_real(e2);gb_real(e3);}
 else if(code==11){double a=-7.,b=-7.,c=-7.;j->ErrorValues(a,b,c);gb_real(a);gb_real(b);gb_real(c);}
 else if(code==12){const int el=r.integer(),d1=r.integer(),d2=r.integer(),rl=r.integer(),cl=r.integer(),n=r.integer();
  math_Matrix h(rl,rl+n-1,cl,cl+n-1,-7.);j->Hessian(el,d1,d2,h);gb_fem_matrix(h);}
 else if(code==13){const int el=r.integer(),d=r.integer(),lower=r.integer(),n=r.integer();math_Vector g(lower,lower+n-1,-7.);
  j->Gradient(el,d,g);gb_fem_vector(g);}
 else if(code==14){math_Vector x=gb_fem_read_vector(r);j->InputVector(x,j->AssemblyTable());const occ::handle<FEmTool_Curve> c=j->Curve();
  for(int el=1;el<=c->NbElements();++el){gb_int(c->Degree(el));NCollection_Array2<double> a(0,c->Base().WorkDegree(),1,c->Dimension());a.Init(-7.);
   c->GetElement(el,a);for(int i=a.LowerRow();i<=a.UpperRow();++i)for(int d=a.LowerCol();d<=a.UpperCol();++d)gb_real(a(i,d));}}
 else if(code==15){double w[5];for(double& v:w)v=r.real();j->SetWeight(w[0],w[1],w[2],w[3],w[4]);}
 else if(code==16){double a,b;j->GetWeight(a,b);gb_real(a);gb_real(b);}
 else if(code==17){math_Vector v=gb_fem_read_vector(r);NCollection_Array1<double> w(v.Lower(),v.Upper());
  for(int i=v.Lower();i<=v.Upper();++i)w(i)=v(i);j->SetWeight(w);}
 else gb_flag(j->Curve().IsNull());
}
// `ad_criteria line first last count commands...`.
void gb_ad_criteria_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);const int first=r.integer(),last=r.integer(),count=r.integer();
 try{
  occ::handle<AppDef_SmoothCriterion> j=new AppDef_LinearCriteria(line,first,last);
  for(int k=0;k<count;++k){GB_AD_TRY(gb_ad_criteria_command(r,j));}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
