// ShapeCustom_Curve2d and ShapeCustom_RestrictionParameters probe operations; the Rust
// twin is crates/tkshhealing/examples/sh_probe/shape_custom.rs.
//
// `sc_curve2d op ...`:
//  0 n (u v)... tol: IsLinear on the poles (lower bound 1): the flag and the deviation
//    (-1 when untouched);
//  1 (Geom2d curve) c1 c2 tol: ConvertToLine2d (the curve record of
//    tktopalgo_probe_breplib_common.cxx): the line type, cf, cl and the deviation (-1
//    when untouched), then the line digest;
//  2 (Geom2d B-spline record) tol: SimplifyBSpline2d, then the degree, the knots and
//    multiplicities, the poles and the curve digest.
// `sc_restriction nops (field value)...`: sets fields (0 GMaxDegree, 1 GMaxSeg, then the
// flags in header order: ConvertPlane, ConvertBezierSurf, ConvertRevolutionSurf,
// ConvertExtrusionSurf, ConvertOffsetSurf, ConvertCylindricalSurf, ConvertConicalSurf,
// ConvertToroidalSurf, ConvertSphericalSurf, SegmentSurfaceMode, ConvertCurve3d,
// ConvertOffsetCurv3d, ConvertCurve2d, ConvertOffsetCurv2d) through the references, then
// prints every field and the dynamic type.
#include <ShapeCustom_Curve2d.hxx>
#include <ShapeCustom_RestrictionParameters.hxx>
void sc_is_linear(GaReader& r){
 int n=r.integer();NCollection_Array1<gp_Pnt2d> poles(1,n);
 for(int i=1;i<=n;++i)poles(i)=r.pnt2d();double tol=r.real(),dev=-1.;
 GA_TRY(ga_flag(ShapeCustom_Curve2d::IsLinear(poles,tol,dev));ga_real(dev))}
void sc_convert_line(GaReader& r){
 occ::handle<Geom2d_Curve> c=tp_geom2d_curve(r);double c1=r.real(),c2=r.real(),tol=r.real();
 double cf=-1.,cl=-1.,dev=-1.;
 GA_TRY(occ::handle<Geom2d_Line> line=ShapeCustom_Curve2d::ConvertToLine2d(c,c1,c2,tol,cf,cl,dev);
  tp_type(line);ga_real(cf);ga_real(cl);ga_real(dev);if(!line.IsNull())shc_curve2d(line))}
void sc_simplify(GaReader& r){
 occ::handle<Geom2d_BSplineCurve> bs=occ::down_cast<Geom2d_BSplineCurve>(tp_geom2d_curve(r));double tol=r.real();
 GA_TRY(ga_flag(ShapeCustom_Curve2d::SimplifyBSpline2d(bs,tol)))
 ga_int(bs->Degree());ga_int(bs->NbKnots());
 for(int i=1;i<=bs->NbKnots();++i){ga_real(bs->Knot(i));ga_int(bs->Multiplicity(i));}
 ga_int(bs->NbPoles());for(int i=1;i<=bs->NbPoles();++i)ga_pnt2d(bs->Pole(i));
 shc_curve2d(bs);}
void sc_curve2d_case(std::istream& input){
 GaReader r{input};int op=r.integer();
 if(op==0)sc_is_linear(r);else if(op==1)sc_convert_line(r);else sc_simplify(r);
 ga_end();}
void sc_restriction_set(ShapeCustom_RestrictionParameters& p,int field,int value){
 if(field==0){p.GMaxDegree()=value;return;}
 if(field==1){p.GMaxSeg()=value;return;}
 bool* flags[]={&p.ConvertPlane(),&p.ConvertBezierSurf(),&p.ConvertRevolutionSurf(),&p.ConvertExtrusionSurf(),
  &p.ConvertOffsetSurf(),&p.ConvertCylindricalSurf(),&p.ConvertConicalSurf(),&p.ConvertToroidalSurf(),
  &p.ConvertSphericalSurf(),&p.SegmentSurfaceMode(),&p.ConvertCurve3d(),&p.ConvertOffsetCurv3d(),
  &p.ConvertCurve2d(),&p.ConvertOffsetCurv2d()};
 *flags[field-2]=value!=0;}
void sc_restriction_case(std::istream& input){
 GaReader r{input};occ::handle<ShapeCustom_RestrictionParameters> p=new ShapeCustom_RestrictionParameters();
 int nops=r.integer();for(int k=0;k<nops;++k){int field=r.integer();sc_restriction_set(*p,field,r.integer());}
 ga_int(p->GMaxDegree());ga_int(p->GMaxSeg());
 for(bool v:{p->ConvertPlane(),p->ConvertBezierSurf(),p->ConvertRevolutionSurf(),p->ConvertExtrusionSurf(),
  p->ConvertOffsetSurf(),p->ConvertCylindricalSurf(),p->ConvertConicalSurf(),p->ConvertToroidalSurf(),
  p->ConvertSphericalSurf(),p->SegmentSurfaceMode(),p->ConvertCurve3d(),p->ConvertOffsetCurv3d(),
  p->ConvertCurve2d(),p->ConvertOffsetCurv2d()})ga_flag(v);
 ga_text(p->DynamicType()->Name());ga_end();}
