// Application-defined virtual surface forwards geometry but mutates valid live caller references once.
#pragma once
class IsolineAliasSurface:public Geom_Surface {
  occ::handle<Geom_Plane> plane=new Geom_Plane(gp_Ax3());
public:
  occ::handle<Geom_Surface>* source=nullptr;
  TopLoc_Location* location=nullptr;
  NCollection_Sequence<double>* u=nullptr;
  NCollection_Sequence<double>* v=nullptr;
  mutable int calls=0;
  bool failClosed=false;
  void UReverse()override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    plane->UReverse();
  }
  double UReversedParameter(double x)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return plane->UReversedParameter(x);
  }
  void VReverse()override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    plane->VReverse();
  }
  double VReversedParameter(double x)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return plane->VReversedParameter(x);
  }
  void Bounds(double& a,double& b,double& c,double& d)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    plane->Bounds(a,b,c,d);
  }
  bool IsUClosed()const override {
    if(failClosed)throw Standard_Failure("isoline closed callback=true; expected successful closure query");
    return false;
  }
  bool IsVClosed()const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return false;
  }
  bool IsUPeriodic()const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return false;
  }
  bool IsVPeriodic()const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return false;
  }
  occ::handle<Geom_Curve> UIso(double x)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return changed_curve(x,true);
  }
  occ::handle<Geom_Curve> VIso(double x)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return changed_curve(x,false);
  }
  GeomAbs_Shape Continuity()const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return GeomAbs_CN;
  }
  bool IsCNu(int)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return true;
  }
  bool IsCNv(int)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return true;
  }
  gp_Pnt EvalD0(double a,double b)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return plane->EvalD0(a,b);
  }
  ResD1 EvalD1(double a,double b)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return plane->EvalD1(a,b);
  }
  ResD2 EvalD2(double a,double b)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return plane->EvalD2(a,b);
  }
  ResD3 EvalD3(double a,double b)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return plane->EvalD3(a,b);
  }
  gp_Vec EvalDN(double a,double b,int c,int d)const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return plane->EvalDN(a,b,c,d);
  }
  void Transform(const gp_Trsf& transform)override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    plane->Transform(transform);
  }
  occ::handle<Geom_Geometry> Copy()const override {
    // Delegation retains real geometry; alias mutation belongs only to the iso hooks.
    return new IsolineAliasSurface;
  }
private:
  occ::handle<Geom_Curve> changed_curve(double parameter,bool isU)const {
    ++calls;
    *source=new Geom_Plane(gp_Ax3());
    gp_Trsf trsf;trsf.SetTranslation(gp_Vec(7.,-3.,2.));*location=TopLoc_Location(trsf);
    u->Append(.75);v->Append(.8);
    return isU?plane->UIso(parameter):plane->VIso(parameter);
  }
};
