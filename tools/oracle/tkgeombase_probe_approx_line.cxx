// The Approx multi-line drivers (tools/tkgeombase_corpus_approx.py): Approx_ComputeLine
// instantiated here over AppDef_MultiLine with a line tool whose WhatStatus,
// MakeMLBetween and MakeMLOneMorePoint follow scripted rules (AppDef_MyLineTool never
// adds points), AppDef_BSplineCompute, Approx_FitAndDivide(2d) and
// Approx_MCurvesToBSpCurve. The Rust twin is crates/tkgeombase/examples/gb_probe/approx.

#include <AppCont_Function.hxx>
#include <AppDef_BSplineCompute.hxx>
#include <AppDef_MultiLine.hxx>
#include <AppDef_MultiPointConstraint.hxx>
#include <AppDef_MyGradientOfCompute.hxx>
#include <AppDef_MyLineTool.hxx>
#include <AppDef_ParLeastSquareOfMyGradientOfCompute.hxx>
#include <Approx_FitAndDivide.hxx>
#include <Approx_FitAndDivide2d.hxx>
#include <Approx_MCurvesToBSpCurve.hxx>
#include <Approx_ParametrizationType.hxx>
#include <Approx_Status.hxx>
// The request's line rules: the status mode, the point cap of PointsAdded, and the
// parameter of every point of a line sampled on the source curves (kind 1).
struct GbAxRules{int mode=0;int cap=0;bool source=false;std::map<std::vector<double>,double> params;};
static GbAxRules gb_ax_rules;
// The source curves: 3D curve k is a nodal cubic lifted along z, 2D curve k a nodal
// cubic that never self-intersects at a representable parameter.
gp_Pnt gb_ax_p3(int k,double t){const double c=k;return gp_Pnt((t*t-1.0)*c,(t*t*t-1.1*t)*c,0.25*t+c);}
gp_Vec gb_ax_d3(int k,double t){const double c=k;return gp_Vec(2.0*t*c,(3.0*t*t-1.1)*c,0.25);}
gp_Pnt2d gb_ax_p2(int k,double t){const double c=k;return gp_Pnt2d(t*t-1.0+(c-1.0),(t*t*t-1.1*t)*c);}
gp_Vec2d gb_ax_d2(int k,double t){const double c=k;return gp_Vec2d(2.0*t,(3.0*t*t-1.1)*c);}
std::vector<double> gb_ax_key(const AppParCurves_MultiPoint& m){
 std::vector<double> key;
 for(int k=1;k<=m.NbPoints();++k){const gp_Pnt p=m.Point(k);key.push_back(p.X());key.push_back(p.Y());key.push_back(p.Z());}
 for(int k=m.NbPoints()+1;k<=m.NbPoints()+m.NbPoints2d();++k){const gp_Pnt2d p=m.Point2d(k);key.push_back(p.X());key.push_back(p.Y());}
 return key;
}
AppDef_MultiPointConstraint gb_ax_source_point(int nb3d,int nb2d,double t,bool tangent){
 NCollection_Array1<gp_Pnt> p3(1,nb3d);NCollection_Array1<gp_Pnt2d> p2(1,nb2d);
 for(int k=1;k<=nb3d;++k)p3(k)=gb_ax_p3(k,t);
 for(int k=1;k<=nb2d;++k)p2(k)=gb_ax_p2(k,t);
 AppDef_MultiPointConstraint m=nb2d==0?AppDef_MultiPointConstraint(p3):nb3d==0?AppDef_MultiPointConstraint(p2):AppDef_MultiPointConstraint(p3,p2);
 if(tangent){
  for(int k=1;k<=nb3d;++k)m.SetTang(k,gb_ax_d3(k,t));
  for(int k=1;k<=nb2d;++k)m.SetTang2d(nb3d+k,gb_ax_d2(k,t));
 }
 gb_ax_rules.params[gb_ax_key(m)]=t;
 return m;
}
double gb_ax_param(const AppDef_MultiLine& ml,int i){
 const auto found=gb_ax_rules.params.find(gb_ax_key(ml.Value(i)));
 if(found==gb_ax_rules.params.end())throw Standard_Failure("probe: a point without parameter");
 return found->second;
}
// AppDef_MyLineTool with the Approx members of the rules: mode 0 never adds points
// (AppDef), 1 adds them while the line is under the cap, 2 refuses to approximate,
// 3 cycles through the three on (I1 + I2 + NbPoints) % 3. A line without source
// comes back unchanged from MakeMLBetween and gets no extra point.
struct GbAxLineTool{
 static int FirstPoint(const AppDef_MultiLine& ml){return AppDef_MyLineTool::FirstPoint(ml);}
 static int LastPoint(const AppDef_MultiLine& ml){return AppDef_MyLineTool::LastPoint(ml);}
 static int NbP2d(const AppDef_MultiLine& ml){return AppDef_MyLineTool::NbP2d(ml);}
 static int NbP3d(const AppDef_MultiLine& ml){return AppDef_MyLineTool::NbP3d(ml);}
 static void Value(const AppDef_MultiLine& ml,const int i,NCollection_Array1<gp_Pnt>& p){AppDef_MyLineTool::Value(ml,i,p);}
 static void Value(const AppDef_MultiLine& ml,const int i,NCollection_Array1<gp_Pnt2d>& p){AppDef_MyLineTool::Value(ml,i,p);}
 static void Value(const AppDef_MultiLine& ml,const int i,NCollection_Array1<gp_Pnt>& p,NCollection_Array1<gp_Pnt2d>& q){AppDef_MyLineTool::Value(ml,i,p,q);}
 static bool Tangency(const AppDef_MultiLine& ml,const int i,NCollection_Array1<gp_Vec>& v){return AppDef_MyLineTool::Tangency(ml,i,v);}
 static bool Tangency(const AppDef_MultiLine& ml,const int i,NCollection_Array1<gp_Vec2d>& v){return AppDef_MyLineTool::Tangency(ml,i,v);}
 static bool Tangency(const AppDef_MultiLine& ml,const int i,NCollection_Array1<gp_Vec>& v,NCollection_Array1<gp_Vec2d>& w){return AppDef_MyLineTool::Tangency(ml,i,v,w);}
 static Approx_Status WhatStatus(const AppDef_MultiLine& ml,const int i1,const int i2){
  const int n=ml.NbMultiPoints();const Approx_Status added=n<gb_ax_rules.cap?Approx_PointsAdded:Approx_NoPointsAdded;
  if(gb_ax_rules.mode==1)return added;
  if(gb_ax_rules.mode==2)return Approx_NoApproximation;
  if(gb_ax_rules.mode==3){const int c=(i1+i2+n)%3;return c==0?added:c==1?Approx_NoPointsAdded:Approx_NoApproximation;}
  return Approx_NoPointsAdded;
 }
 static AppDef_MultiLine MakeMLBetween(const AppDef_MultiLine& ml,const int i1,const int i2,const int nbpmin){
  if(!gb_ax_rules.source)return ml;
  const int n=std::max(2*(i2-i1)+1,nbpmin),nb3d=NbP3d(ml),nb2d=NbP2d(ml);
  const double t1=gb_ax_param(ml,i1),t2=gb_ax_param(ml,i2);
  const bool tan1=ml.Value(i1).IsTangencyPoint(),tan2=ml.Value(i2).IsTangencyPoint();
  AppDef_MultiLine line(n);
  for(int k=0;k<n;++k){
   const double t=n==1?t1:t1+(t2-t1)*double(k)/double(n-1);
   line.SetValue(k+1,gb_ax_source_point(nb3d,nb2d,t,(k==0&&tan1)||(k==n-1&&tan2)));
  }
  return line;
 }
 static bool MakeMLOneMorePoint(const AppDef_MultiLine& ml,const int i1,const int i2,const int indbad,AppDef_MultiLine& other){
  if(!gb_ax_rules.source||ml.NbMultiPoints()>=gb_ax_rules.cap||indbad<=i1||indbad>i2)return false;
  AppDef_MultiLine line(i2-i1+2);int j=1;
  for(int i=i1;i<=i2;++i){
   if(i==indbad)line.SetValue(j++,gb_ax_source_point(NbP3d(ml),NbP2d(ml),0.5*(gb_ax_param(ml,i-1)+gb_ax_param(ml,i)),false));
   line.SetValue(j++,ml.Value(i));
  }
  other=line;return true;
 }
};
// Approx_ComputeLine over AppDef_MultiLine and GbAxLineTool, with the AppDef
// instantiations of the AppParCurves generics (they read the line through
// AppDef_MyLineTool, which GbAxLineTool forwards to).
class GbAxCompute{
public:
 GbAxCompute(const AppDef_MultiLine& Line,const int degreemin,const int degreemax,const double Tolerance3d,const double Tolerance2d,const int NbIterations,const bool cutting,const Approx_ParametrizationType parametrization,const bool Squares);
 GbAxCompute(const AppDef_MultiLine& Line,const math_Vector& Parameters,const int degreemin,const int degreemax,const double Tolerance3d,const double Tolerance2d,const int NbIterations,const bool cutting,const bool Squares);
 GbAxCompute(const math_Vector& Parameters,const int degreemin,const int degreemax,const double Tolerance3d,const double Tolerance2d,const int NbIterations,const bool cutting,const bool Squares);
 GbAxCompute(const int degreemin,const int degreemax,const double Tolerance3d,const double Tolerance2d,const int NbIterations,const bool cutting,const Approx_ParametrizationType parametrization,const bool Squares);
 void Init(const int degreemin,const int degreemax,const double Tolerance3d,const double Tolerance2d,const int NbIterations,const bool cutting,const Approx_ParametrizationType parametrization,const bool Squares);
 void Perform(const AppDef_MultiLine& Line);
 void SetDegrees(const int degreemin,const int degreemax);
 void SetTolerances(const double Tolerance3d,const double Tolerance2d);
 void SetConstraints(const AppParCurves_Constraint firstC,const AppParCurves_Constraint lastC);
 bool IsAllApproximated() const;
 bool IsToleranceReached() const;
 void Error(const int Index,double& tol3d,double& tol2d) const;
 int NbMultiCurves() const;
 const AppParCurves_MultiCurve& Value(const int Index=1) const;
 AppParCurves_MultiCurve& ChangeValue(const int Index=1);
 const AppParCurves_MultiBSpCurve& SplineValue();
 Approx_ParametrizationType Parametrization() const;
 const NCollection_Array1<double>& Parameters(const int Index=1) const;
private:
 bool Compute(const AppDef_MultiLine& Line,const int fpt,const int lpt,math_Vector& Para,double& TheTol3d,double& TheTol2d,int& indbad);
 bool ComputeCurve(const AppDef_MultiLine& Line,const int firspt,const int lastpt);
 void Parameters(const AppDef_MultiLine& Line,const int firstP,const int LastP,math_Vector& TheParameters) const;
 double SearchFirstLambda(const AppDef_MultiLine& Line,const math_Vector& Para,const math_Vector& V,const int index) const;
 double SearchLastLambda(const AppDef_MultiLine& Line,const math_Vector& Para,const math_Vector& V,const int index) const;
 void FirstTangencyVector(const AppDef_MultiLine& Line,const int index,math_Vector& V) const;
 void LastTangencyVector(const AppDef_MultiLine& Line,const int index,math_Vector& V) const;
 NCollection_Sequence<AppParCurves_MultiCurve> myMultiCurves;
 AppParCurves_MultiCurve TheMultiCurve;
 AppParCurves_MultiBSpCurve myspline;
 bool alldone;
 // The native constructors never set tolreached nor the current errors; the
 // instantiation starts them as the port does (false, 0).
 bool tolreached=false;
 Approx_ParametrizationType Par;
 occ::handle<NCollection_HArray1<double>> myParameters;
 occ::handle<NCollection_HArray1<double>> myfirstParam;
 NCollection_Sequence<occ::handle<NCollection_HArray1<double>>> myPar;
 NCollection_Sequence<double> Tolers3d;
 NCollection_Sequence<double> Tolers2d;
 occ::handle<NCollection_HArray1<AppParCurves_ConstraintCouple>> myConstraints;
 int mydegremin;
 int mydegremax;
 double mytol3d;
 double mytol2d;
 double currenttol3d=0.;
 double currenttol2d=0.;
 bool mycut;
 bool mysquares;
 int myitermax;
 AppParCurves_Constraint myfirstC;
 AppParCurves_Constraint mylastC;
 int myMultiLineNb;
 bool myIsClear;
};
#define MultiLine AppDef_MultiLine
#define LineTool GbAxLineTool
#define Approx_MyGradient AppDef_MyGradientOfCompute
#define Approx_ParLeastSquareOfMyGradient AppDef_ParLeastSquareOfMyGradientOfCompute
#define Approx_ComputeLine GbAxCompute
#include <Approx_ComputeLine.gxx>
#undef MultiLine
#undef LineTool
#undef Approx_MyGradient
#undef Approx_ParLeastSquareOfMyGradient
#undef Approx_ComputeLine
