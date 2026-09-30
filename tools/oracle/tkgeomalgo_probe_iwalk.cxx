// IntWalk_IWalking.gxx probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_walk_iw.rs. The template is instantiated here
// on a probe class, as the _0.cxx files do, with the types of IntPatch_TheIWalking:
// Adaptor3d_HSurfaceTool, IntPatch_TheSurfFunction and IntPatch_TheIWLineOfTheIWalking.
// Included last: the .gxx defines No_Standard_OutOfRange, undefined again below.
#include <Adaptor3d_HSurfaceTool.hxx>
#include <Bnd_Range.hxx>
#include <IntPatch_TheIWLineOfTheIWalking.hxx>
#include <IntPatch_TheSurfFunction.hxx>
#include <IntSurf_InteriorPoint.hxx>
#include <IntSurf_InteriorPointTool.hxx>
#include <IntSurf_PathPoint.hxx>
#include <IntSurf_PathPointTool.hxx>
#include <IntWalk_StatusDeflection.hxx>
#include <IntWalk_VectorOfInteger.hxx>
#include <IntWalk_VectorOfWalkingData.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Sequence.hxx>
#include <StdFail_NotDone.hxx>
#include <math_FunctionSetRoot.hxx>
#include <math_Vector.hxx>
// The declaration of IntPatch_TheIWalking.hxx under the probe name.
class GaProbeIWalking
{
public:
  GaProbeIWalking(const double Epsilon, const double Deflection, const double Step, const bool theToFillHoles = false);
  void SetTolerance(const double Epsilon, const double Deflection, const double Step);
  void Perform(const NCollection_Sequence<IntSurf_PathPoint>& Pnts1, const NCollection_Sequence<IntSurf_InteriorPoint>& Pnts2,
               IntPatch_TheSurfFunction& Func, const occ::handle<Adaptor3d_Surface>& S, const bool Reversed = false);
  void Perform(const NCollection_Sequence<IntSurf_PathPoint>& Pnts1, IntPatch_TheSurfFunction& Func,
               const occ::handle<Adaptor3d_Surface>& S, const bool Reversed = false);
  bool IsDone() const;
  int NbLines() const;
  const occ::handle<IntPatch_TheIWLineOfTheIWalking>& Value(const int Index) const;
  int NbSinglePnts() const;
  const IntSurf_PathPoint& SinglePnt(const int Index) const;
protected:
  bool Cadrage(math_Vector& BornInf, math_Vector& BornSup, math_Vector& UVap, double& Step, const int StepSign) const;
  bool TestArretPassage(const NCollection_Sequence<double>& Umult, const NCollection_Sequence<double>& Vmult,
                        IntPatch_TheSurfFunction& Section, math_Vector& UV, int& Irang);
  bool TestArretPassage(const NCollection_Sequence<double>& Umult, const NCollection_Sequence<double>& Vmult,
                        const math_Vector& UV, const int Index, int& Irang);
  bool TestArretAjout(IntPatch_TheSurfFunction& Section, math_Vector& UV, int& Irang, IntSurf_PntOn2S& PSol);
  void FillPntsInHoles(IntPatch_TheSurfFunction& Section, NCollection_Sequence<int>& CopySeqAlone,
                       NCollection_Sequence<IntSurf_InteriorPoint>& PntsInHoles);
  void TestArretCadre(const NCollection_Sequence<double>& Umult, const NCollection_Sequence<double>& Vmult,
                      const occ::handle<IntPatch_TheIWLineOfTheIWalking>& Line, IntPatch_TheSurfFunction& Section,
                      math_Vector& UV, int& Irang);
  IntWalk_StatusDeflection TestDeflection(IntPatch_TheSurfFunction& Section, const bool Finished, const math_Vector& UV,
                                          const IntWalk_StatusDeflection StatusPrecedent, int& NbDivision, double& Step,
                                          const int StepSign);
  void ComputeOpenLine(const NCollection_Sequence<double>& Umult, const NCollection_Sequence<double>& Vmult,
                       const NCollection_Sequence<IntSurf_PathPoint>& Pnts1, IntPatch_TheSurfFunction& Section, bool& Rajout);
  void OpenLine(const int N, const IntSurf_PntOn2S& Psol, const NCollection_Sequence<IntSurf_PathPoint>& Pnts1,
                IntPatch_TheSurfFunction& Section, const occ::handle<IntPatch_TheIWLineOfTheIWalking>& Line);
  bool IsValidEndPoint(const int IndOfPoint, const int IndOfLine);
  void RemoveTwoEndPoints(const int IndOfPoint);
  bool IsPointOnLine(const gp_Pnt2d& theP2d, const int Irang);
  void ComputeCloseLine(const NCollection_Sequence<double>& Umult, const NCollection_Sequence<double>& Vmult,
                        const NCollection_Sequence<IntSurf_PathPoint>& Pnts1, const NCollection_Sequence<IntSurf_InteriorPoint>& Pnts2,
                        IntPatch_TheSurfFunction& Section, bool& Rajout);
  void AddPointInCurrentLine(const int N, const IntSurf_PathPoint& PathPnt,
                             const occ::handle<IntPatch_TheIWLineOfTheIWalking>& CurrentLine) const;
  void MakeWalkingPoint(const int Case, const double U, const double V, IntPatch_TheSurfFunction& Section, IntSurf_PntOn2S& Psol);
  void Clear();
  bool IsPointOnLine(const IntSurf_PntOn2S& thePOn2S, const math_Vector& theInfBounds, const math_Vector& theSupBounds,
                     math_FunctionSetRoot& theSolver, IntPatch_TheSurfFunction& theFunc);
private:
  bool done;
  NCollection_Sequence<IntSurf_PathPoint> seqSingle;
  double fleche;
  double pas;
  math_Vector tolerance;
  double epsilon;
  bool reversed;
  IntWalk_VectorOfWalkingData wd1;
  IntWalk_VectorOfWalkingData wd2;
  IntWalk_VectorOfInteger nbMultiplicities;
  Bnd_Range mySRangeU;
  Bnd_Range mySRangeV;
  double Um;
  double UM;
  double Vm;
  double VM;
  IntSurf_PntOn2S previousPoint;
  gp_Vec previousd3d;
  gp_Dir2d previousd2d;
  NCollection_Sequence<int> seqAjout;
  NCollection_Sequence<int> seqAlone;
  NCollection_DataMap<int, NCollection_List<int>> PointLineLine;
  NCollection_Sequence<occ::handle<IntPatch_TheIWLineOfTheIWalking>> lines;
  bool ToFillHoles;
};
#define ThePointOfPath IntSurf_PathPoint
#define ThePointOfPathTool IntSurf_PathPointTool
#define ThePOPIterator NCollection_Sequence<IntSurf_PathPoint>
#define ThePointOfLoop IntSurf_InteriorPoint
#define ThePointOfLoopTool IntSurf_InteriorPointTool
#define ThePOLIterator NCollection_Sequence<IntSurf_InteriorPoint>
#define ThePSurface occ::handle<Adaptor3d_Surface>
#define ThePSurfaceTool Adaptor3d_HSurfaceTool
#define TheIWFunction IntPatch_TheSurfFunction
#define IntWalk_TheIWLine IntPatch_TheIWLineOfTheIWalking
#define IntWalk_TheIWLine_hxx <IntPatch_TheIWLineOfTheIWalking.hxx>
#define IntWalk_SequenceOfIWLine NCollection_Sequence<occ::handle<IntPatch_TheIWLineOfTheIWalking>>
#define Handle_IntWalk_TheIWLine occ::handle<IntPatch_TheIWLineOfTheIWalking>
#define IntWalk_IWalking GaProbeIWalking
#include <IntWalk_IWalking.lxx>
#include <IntWalk_IWalking.gxx>
#undef No_Standard_RangeError
#undef No_Standard_OutOfRange
#undef ThePointOfPath
#undef ThePointOfPathTool
#undef ThePOPIterator
#undef ThePointOfLoop
#undef ThePointOfLoopTool
#undef ThePOLIterator
#undef ThePSurface
#undef ThePSurfaceTool
#undef TheIWFunction
#undef IntWalk_TheIWLine
#undef IntWalk_TheIWLine_hxx
#undef IntWalk_SequenceOfIWLine
#undef Handle_IntWalk_TheIWLine
#undef IntWalk_IWalking
// A path point at (u, v) of the function: its point, its tangents unless `tangent`,
// the passing flag and `nmult` auxiliary parameters.
IntSurf_PathPoint ga_iw_path_point(GaReader& r,IntPatch_TheSurfFunction& f){
 double u=r.real(),v=r.real();bool pass=r.flag(),tangent=r.flag();int nmult=r.integer();
 math_Vector x(1,2),fv(1,1);math_Matrix d(1,1,1,2);x(1)=u;x(2)=v;f.Values(x,fv,d);
 IntSurf_PathPoint p(f.Point(),u,v);
 if(!tangent)p.SetDirections(f.Direction3d(),f.Direction2d());
 p.SetPassing(pass);
 for(int i=0;i<nmult;i++){double a=r.real();p.AddUV(a,r.real());}
 return p;
}
// An interior point at (u, v): the function point and tangents (null ones when tangent).
IntSurf_InteriorPoint ga_iw_interior_point(GaReader& r,IntPatch_TheSurfFunction& f){
 double u=r.real(),v=r.real();math_Vector x(1,2),fv(1,1);math_Matrix d(1,1,1,2);x(1)=u;x(2)=v;f.Values(x,fv,d);
 if(f.IsTangent())return IntSurf_InteriorPoint(f.Point(),u,v,gp_Vec(0,0,0),gp_Vec2d(1,0));
 gp_Dir2d d2=f.Direction2d();return IntSurf_InteriorPoint(f.Point(),u,v,f.Direction3d(),gp_Vec2d(d2.X(),d2.Y()));
}
void ga_iw_line_report(const occ::handle<IntPatch_TheIWLineOfTheIWalking>& l){
 int n=l->NbPoints();ga_int(n);for(int i=1;i<=n;i++)ga_iw_pnt2s(l->Value(i));
 ga_flag(l->IsClosed());ga_flag(l->HasFirstPoint());ga_flag(l->HasLastPoint());
 GA_TRY(ga_int(l->FirstPointIndex()));GA_TRY(ga_pnt(l->FirstPoint().Value()));
 GA_TRY(ga_int(l->LastPointIndex()));GA_TRY(ga_pnt(l->LastPoint().Value()));
 ga_int(l->NbPassingPoint());for(int i=1;i<=l->NbPassingPoint();i++){int a,b;l->PassingPoint(i,a,b);ga_int(a);ga_int(b);}
 int idx=0;const gp_Vec& v=l->TangentVector(idx);ga_xyz(v.XYZ());ga_int(idx);
 ga_flag(l->IsTangentAtBegining());ga_flag(l->IsTangentAtEnd());
}
void ga_iw_iw_report(const GaProbeIWalking& w){
 ga_flag(w.IsDone());
 GA_TRY(int n=w.NbLines();ga_int(n);for(int i=1;i<=n;i++)ga_iw_line_report(w.Value(i)));
 GA_TRY(int n=w.NbSinglePnts();ga_int(n);
  for(int i=1;i<=n;i++){const IntSurf_PathPoint& p=w.SinglePnt(i);ga_pnt(p.Value());double u,v;p.Value2d(u,v);ga_real(u);ga_real(v);});
 GA_TRY(w.Value(0));GA_TRY(w.Value(w.NbLines()+1));GA_TRY(w.SinglePnt(0));
}
// `surface quadric tol eps defl step fill reversed mode npath [u v pass tangent nmult
// (u v)*]* nloop [u v]*`: the walking of the quadric on the surface; mode 0 the full
// Perform, 1 the full Perform then the open-line Perform, 2 the consultation of a
// walking never performed.
void ga_iw_iw_walk_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_iw_surface(r);IntSurf_Quadric q=ga_intsurf_quadric(r);
 double tol=r.real(),eps=r.real(),defl=r.real(),step=r.real();bool fill=r.flag(),rev=r.flag();int mode=r.integer();
 GA_TRY(IntPatch_TheSurfFunction f(s,q);f.Set(tol);
  NCollection_Sequence<IntSurf_PathPoint> paths;NCollection_Sequence<IntSurf_InteriorPoint> loops;
  int np=r.integer();for(int i=0;i<np;i++)paths.Append(ga_iw_path_point(r,f));
  int nl=r.integer();for(int i=0;i<nl;i++)loops.Append(ga_iw_interior_point(r,f));
  GaProbeIWalking w(eps,defl,step,fill);
  if(mode!=2){GA_TRY(w.Perform(paths,loops,f,s,rev));}
  if(mode==1){GA_TRY(w.Perform(paths,f,s,rev));}
  ga_iw_iw_report(w));
 ga_end();
}
