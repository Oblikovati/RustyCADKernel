// TKGeomBase oracle: the GC package (tkgeombase_probe.cxx). Each request names one class;
// its first number selects the constructor overload (in header order) and the arguments
// follow. A GC_Root algorithm prints its status, IsDone, IsError, Value() and the
// conversion operator; a transformation maker prints Value() and the conversion. A Geom or
// Geom2d result prints its class name and defining data: the gp element of a conic, line,
// plane, cone or cylinder, the bounds and basis of a trimmed curve or surface, the gp_Trsf
// of a transformation. The makers are heap-allocated (zeroed), as in the gce probe.
#include <GC_MakeArcOfCircle.hxx>
#include <GC_MakeArcOfCircle2d.hxx>
#include <GC_MakeArcOfEllipse.hxx>
#include <GC_MakeArcOfEllipse2d.hxx>
#include <GC_MakeArcOfHyperbola.hxx>
#include <GC_MakeArcOfHyperbola2d.hxx>
#include <GC_MakeArcOfParabola.hxx>
#include <GC_MakeArcOfParabola2d.hxx>
#include <GC_MakeCircle.hxx>
#include <GC_MakeCircle2d.hxx>
#include <GC_MakeConicalSurface.hxx>
#include <GC_MakeCylindricalSurface.hxx>
#include <GC_MakeEllipse.hxx>
#include <GC_MakeEllipse2d.hxx>
#include <GC_MakeHyperbola.hxx>
#include <GC_MakeHyperbola2d.hxx>
#include <GC_MakeLine.hxx>
#include <GC_MakeLine2d.hxx>
#include <GC_MakeMirror.hxx>
#include <GC_MakeMirror2d.hxx>
#include <GC_MakeParabola2d.hxx>
#include <GC_MakePlane.hxx>
#include <GC_MakeRotation.hxx>
#include <GC_MakeRotation2d.hxx>
#include <GC_MakeScale.hxx>
#include <GC_MakeScale2d.hxx>
#include <GC_MakeSegment.hxx>
#include <GC_MakeSegment2d.hxx>
#include <GC_MakeTranslation.hxx>
#include <GC_MakeTranslation2d.hxx>
#include <GC_MakeTrimmedCone.hxx>
#include <GC_MakeTrimmedCylinder.hxx>
#include <Geom_Circle.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Hyperbola.hxx>
#include <Geom_Line.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_Transformation.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_Transformation.hxx>
#include <Geom2d_TrimmedCurve.hxx>
void gb_name(const occ::handle<Standard_Transient>& t){std::cout<<(t.IsNull()?"null":t->DynamicType()->Name())<<' ';}
void gb_geom_curve(const occ::handle<Geom_Curve>& c){
 gb_name(c);if(c.IsNull())return;
 if(auto t=occ::down_cast<Geom_TrimmedCurve>(c)){gb_real(t->FirstParameter());gb_real(t->LastParameter());gb_geom_curve(t->BasisCurve());return;}
 if(auto l=occ::down_cast<Geom_Line>(c)){gb_lin(l->Lin());return;}
 if(auto x=occ::down_cast<Geom_Circle>(c)){gb_circ(x->Circ());return;}
 if(auto x=occ::down_cast<Geom_Ellipse>(c)){gb_elips(x->Elips());return;}
 if(auto x=occ::down_cast<Geom_Hyperbola>(c)){gb_hypr(x->Hypr());return;}
 if(auto x=occ::down_cast<Geom_Parabola>(c)){gb_parab(x->Parab());return;}
}
void gb_geom2d_curve(const occ::handle<Geom2d_Curve>& c){
 gb_name(c);if(c.IsNull())return;
 if(auto t=occ::down_cast<Geom2d_TrimmedCurve>(c)){gb_real(t->FirstParameter());gb_real(t->LastParameter());gb_geom2d_curve(t->BasisCurve());return;}
 if(auto l=occ::down_cast<Geom2d_Line>(c)){gb_lin2d(l->Lin2d());return;}
 if(auto x=occ::down_cast<Geom2d_Circle>(c)){gb_circ2d(x->Circ2d());return;}
 if(auto x=occ::down_cast<Geom2d_Ellipse>(c)){gb_elips2d(x->Elips2d());return;}
 if(auto x=occ::down_cast<Geom2d_Hyperbola>(c)){gb_hypr2d(x->Hypr2d());return;}
 if(auto x=occ::down_cast<Geom2d_Parabola>(c)){gb_parab2d(x->Parab2d());return;}
}
void gb_geom_surface(const occ::handle<Geom_Surface>& s){
 gb_name(s);if(s.IsNull())return;
 if(auto t=occ::down_cast<Geom_RectangularTrimmedSurface>(s)){
  double u1,u2,v1,v2;t->Bounds(u1,u2,v1,v2);gb_real(u1);gb_real(u2);gb_real(v1);gb_real(v2);gb_geom_surface(t->BasisSurface());return;}
 if(auto x=occ::down_cast<Geom_Plane>(s)){gb_pln(x->Pln());return;}
 if(auto x=occ::down_cast<Geom_ConicalSurface>(s)){gb_cone(x->Cone());return;}
 if(auto x=occ::down_cast<Geom_CylindricalSurface>(s)){gb_cylinder(x->Cylinder());return;}
}
void gb_geom_trsf(const occ::handle<Geom_Transformation>& t){gb_name(t);if(!t.IsNull())gb_trsf(t->Trsf());}
void gb_geom2d_trsf(const occ::handle<Geom2d_Transformation>& t){gb_name(t);if(!t.IsNull())gb_trsf2d(t->Trsf2d());}
template<class M,class P> void gb_gc_root(M* maker,P print){
 std::unique_ptr<M> owned(maker);using Result=decltype(maker->Value());
 gb_int(maker->Status());gb_flag(maker->IsDone());gb_flag(maker->IsError());
 GB_TRY(print(maker->Value()));GB_TRY(print(static_cast<Result>(*maker)));
}
template<class M,class P> void gb_gc_trsf(M* maker,P print){
 std::unique_ptr<M> owned(maker);using Result=decltype(maker->Value());
 print(maker->Value());print(static_cast<Result>(*maker));
}
#define GB_GC(PRINT,...) GB_TRY(gb_gc_root(new __VA_ARGS__,PRINT))
#define GB_GC_TRSF(PRINT,...) GB_TRY(gb_gc_trsf(new __VA_ARGS__,PRINT))
#include "tkgeombase_probe_gc_curves.cxx"
#include "tkgeombase_probe_gc_surfaces.cxx"
