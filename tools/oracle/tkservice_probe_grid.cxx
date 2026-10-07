// Aspect_RectangularGrid and Aspect_CircularGrid snapping, the Rust twin of
// crates/tkservice/examples/sv_probe/grid.rs. Both classes are abstract natively (the
// drawing members come from TKV3d), so the probe derives the same do-nothing stubs as the
// Aspect_Grid_Bounds GoogleTest.
#include <Aspect_CircularGrid.hxx>
#include <Aspect_RectangularGrid.hxx>
class SvRectGrid : public Aspect_RectangularGrid {
public:
 SvRectGrid(double xs,double ys,double xo,double yo,double a1,double a2,double ra)
  :Aspect_RectangularGrid(xs,ys,xo,yo,a1,a2,ra){}
 void Display() override {}
 void Erase() const override {}
 bool IsDisplayed() const override {return false;}
protected:
 void UpdateDisplay() override {}
};
class SvCircGrid : public Aspect_CircularGrid {
public:
 SvCircGrid(double rs,int nd,double xo,double yo,double ra):Aspect_CircularGrid(rs,nd,xo,yo,ra){}
 void Display() override {}
 void Erase() const override {}
 bool IsDisplayed() const override {return false;}
protected:
 void UpdateDisplay() override {}
};
// `XStep YStep XOrigin YOrigin FirstAngle SecondAngle Rotation X Y`: the constructor
// (or its exception), Init, then Compute(X, Y) and Hit(X, Y) after Activate.
void sv_rgrid_case(std::istream& input){
 GaReader r{input};double xs=r.real(),ys=r.real(),xo=r.real(),yo=r.real(),a1=r.real(),a2=r.real(),ra=r.real();
 double x=r.real(),y=r.real();
 GA_TRY(SvRectGrid grid(xs,ys,xo,yo,a1,a2,ra);grid.Init();double gx=0,gy=0;grid.Compute(x,y,gx,gy);
  ga_real(gx);ga_real(gy);grid.Activate();grid.Hit(x,y,gx,gy);ga_real(gx);ga_real(gy))
 ga_end();}
// `RadiusStep Divisions XOrigin YOrigin Rotation X Y`: SetGridValues (or its exception)
// then Compute(X, Y).
void sv_cgrid_case(std::istream& input){
 GaReader r{input};double rs=r.real();int nd=r.integer();double xo=r.real(),yo=r.real(),ra=r.real();
 double x=r.real(),y=r.real();
 GA_TRY(SvCircGrid grid(1.0,1,0.0,0.0,0.0);grid.SetGridValues(xo,yo,rs,nd,ra);double gx=0,gy=0;
  grid.Compute(x,y,gx,gy);ga_real(gx);ga_real(gy))
 ga_end();}
