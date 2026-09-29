// Copyright (c) 1996-1999 Matra Datavision
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

// AppDef_Variational requests (tools/tkgeombase_corpus_appdef.py): the smoothing of an
// AppDef line under constraints, its results and every setter. The Rust twin is
// crates/tkgeombase/examples/gb_probe/app_def/variational.rs.
#include <AppDef_Variational.hxx>
void gb_ad_harray(const occ::handle<NCollection_HArray1<double>>& a){gb_ax_array(a->Array1());}
occ::handle<NCollection_HArray1<double>> gb_ad_read_harray(GbReader& r){
 math_Vector v=gb_fem_read_vector(r);occ::handle<NCollection_HArray1<double>> a=new NCollection_HArray1<double>(v.Lower(),v.Upper());
 for(int i=v.Lower();i<=v.Upper();++i)a->SetValue(i,v(i));return a;
}
void gb_ad_var_flags(const AppDef_Variational& v){gb_flag(v.IsCreated());gb_flag(v.IsDone());gb_flag(v.IsOverConstrained());}
void gb_ad_var_settings(const AppDef_Variational& v){
 gb_int(v.MaxDegree());gb_int(v.MaxSegment());gb_int(v.Continuity());gb_flag(v.WithMinMax());gb_flag(v.WithCutting());
 gb_real(v.Tolerance());gb_int(v.NbIterations());double a,b,c;v.CriteriumWeight(a,b,c);gb_real(a);gb_real(b);gb_real(c);
}
// The results, each guarded (StdFail_NotDone before an approximation). MaxErrorIndex
// is uninitialized natively when every distance is 0, so it is printed only when the
// maximum error is not.
void gb_ad_var_results(const AppDef_Variational& v){
 GB_AD_TRY(gb_ax_bspline(v.Value()));
 GB_AD_TRY(const double e=v.MaxError();gb_real(e);if(e!=0.)gb_int(v.MaxErrorIndex()));
 GB_AD_TRY(gb_real(v.QuadraticError()));
 GB_AD_TRY(gb_real(v.AverageError()));
 GB_AD_TRY(gb_ad_harray(v.Parameters()));
 GB_AD_TRY(gb_ad_harray(v.Knots()));
 GB_AD_TRY(double a,b,c;v.Criterium(a,b,c);gb_real(a);gb_real(b);gb_real(c));
}
// Variational commands: 1 Approximate then the flags, 2 the settings, 3 the results,
// 4 Distance(rows from `lr`, columns from `lc`), 5 Dump, 6 SetConstraints, 7
// SetParameters, 8 SetKnots, 9 SetMaxDegree, 10 SetMaxSegment, 11 SetContinuity, 12
// SetWithMinMax, 13 SetWithCutting, 14 SetCriteriumWeight(3), 15
// SetCriteriumWeight(Order, Percent), 16 SetTolerance, 17 SetNbIterations.
void gb_ad_var_command(GbReader& r,AppDef_Variational& v,int npoints){
 const int code=r.integer();
 if(code==1){v.Approximate();gb_ad_var_flags(v);}
 else if(code==2)gb_ad_var_settings(v);
 else if(code==3)gb_ad_var_results(v);
 else if(code==4){const int lr=r.integer(),lc=r.integer(),rows=r.integer();math_Matrix m(lr,lr+rows-1,lc,lc+npoints-1,-7.);v.Distance(m);gb_fem_matrix(m);}
 else if(code==5){if(v.IsDone()&&v.MaxError()==0.)gb_int(-1);else gb_apc_dump(v);}
 else if(code==6)gb_flag(v.SetConstraints(gb_apc_read_constraints(r)));
 else if(code==7)v.SetParameters(gb_ad_read_harray(r));
 else if(code==8)gb_flag(v.SetKnots(gb_ad_read_harray(r)));
 else if(code==9)gb_flag(v.SetMaxDegree(r.integer()));
 else if(code==10)gb_flag(v.SetMaxSegment(r.integer()));
 else if(code==11)gb_flag(v.SetContinuity(gb_fem_shape(r.integer())));
 else if(code==12)v.SetWithMinMax(r.flag());
 else if(code==13)gb_flag(v.SetWithCutting(r.flag()));
 else if(code==14){const double a=r.real(),b=r.real();v.SetCriteriumWeight(a,b,r.real());}
 else if(code==15){const int order=r.integer();v.SetCriteriumWeight(order,r.real());}
 else if(code==16)v.SetTolerance(r.real());
 else v.SetNbIterations(r.integer());
}
// `ad_var line first last constraints maxdeg maxseg continuity minmax cutting tol
// iterations count commands...`.
void gb_ad_var_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);const int first=r.integer(),last=r.integer();
 GbApcConstraints cons=gb_apc_read_constraints(r);
 const int maxdeg=r.integer(),maxseg=r.integer(),cont=r.integer();const bool minmax=r.flag(),cutting=r.flag();
 const double tol=r.real();const int iterations=r.integer(),count=r.integer();
 try{
  AppDef_Variational v(line,first,last,cons,maxdeg,maxseg,gb_fem_shape(cont),minmax,cutting,tol,iterations);gb_ad_var_flags(v);
  for(int k=0;k<count;++k){GB_AD_TRY(gb_ad_var_command(r,v,last-first+1));}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
