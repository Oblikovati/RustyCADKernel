// V3d view camera operations on the null driver, the Rust twin of
// crates/tkv3d/examples/v3_probe/v3d.rs.
#include <Aspect_NeutralWindow.hxx>
#include <Bnd_Box.hxx>
#include <V3d.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include "tkservice_probe_null_driver.cxx"
// The axis of an integer code (0 X, 1 Y, otherwise Z).
V3d_TypeOfAxe v3_axe(double code){int c=int(code);return c==0?V3d_X:(c==1?V3d_Y:V3d_Z);}
// The orientation of an integer code, clamped to the 26 orientations.
V3d_TypeOfOrientation v3_orientation(double code){int c=int(code);return V3d_TypeOfOrientation(c<0?0:(c>25?25:c));}
// The camera queries of the view after an operation.
void v3_view_state(const occ::handle<V3d_View>& v,int w,int h){
 double x,y,z;v->Eye(x,y,z);ga_real(x);ga_real(y);ga_real(z);v->At(x,y,z);ga_real(x);ga_real(y);ga_real(z);
 v->Up(x,y,z);ga_real(x);ga_real(y);ga_real(z);v->Proj(x,y,z);ga_real(x);ga_real(y);ga_real(z);
 ga_real(v->Twist());ga_real(v->Scale());double sw,sh;v->Size(sw,sh);ga_real(sw);ga_real(sh);ga_real(v->ZSize());ga_real(v->Depth());
 v->AxialScale(x,y,z);ga_real(x);ga_real(y);ga_real(z);ga_real(v->Focale());
 double xv,yv;v->Convert(w/3,h/4,xv,yv);ga_real(xv);ga_real(yv);
 int xp,yp;v->Convert(0.3*sw,-0.2*sh,xp,yp);ga_int(xp);ga_int(yp);
 v->Convert(w/5,h/3,x,y,z);ga_real(x);ga_real(y);ga_real(z);
 double p[6];v->ConvertWithProj(w/2,h/5,p[0],p[1],p[2],p[3],p[4],p[5]);for(double value:p)ga_real(value);
 v->Project(1.0,2.0,3.0,x,y);ga_real(x);ga_real(y);v->Project(1.0,2.0,3.0,x,y,z);ga_real(x);ga_real(y);ga_real(z);
 v->Convert(1.0,2.0,3.0,xp,yp);ga_int(xp);ga_int(yp);ga_real(v->Convert(17));ga_int(v->Convert(1.5));}
// One operation `Code A B C D` of tools/tkv3d_corpus_v3d.py.
void v3_view_op(const occ::handle<V3d_View>& v,int code,double a,double b,double c,double d){
 int ia=int(a),ib=int(b),ic=int(c),id=int(d);
 switch(code){
  case 0:v->SetProj(v3_orientation(a),b!=0);break;case 1:v->SetSize(a);break;case 2:v->SetZoom(a,b!=0);break;
  case 3:v->Rotate(a,b,c,d!=0);break;case 4:v->Turn(a,b,c,d!=0);break;case 5:v->SetTwist(a);break;
  case 6:v->SetEye(a,b,c);break;case 7:v->SetAt(a,b,c);break;case 8:v->SetDepth(a);break;case 9:v->SetProj(a,b,c);break;
  case 10:v->Panning(a,b,c,d!=0);break;case 11:v->Pan(ia,ib,c,d!=0);break;
  case 12:v->StartZoomAtPoint(ia,ib);v->ZoomAtPoint(ia,ib,ic,id);break;
  case 13:v->StartRotation(ia,ib,0.0);v->Rotation(ic,id);break;
  case 14:v->Move(a,b,c,d!=0);break;case 15:v->Translate(a,b,c,d!=0);break;case 16:v->Place(ia,ib,c);break;
  case 17:v->WindowFit(ia,ib,ic,id);break;
  case 18:{Bnd_Box box;box.Update(d-a,d-b,d-c,d+a,d+b,d+c);v->FitAll(box,0.01,false);break;}
  case 19:v->SetScale(a);break;case 20:v->SetAxialScale(a,b,c);break;
  case 21:v->Rotate(v3_axe(b),a,c,d,0.0,true);break;case 22:v->SetZSize(a);break;case 23:v->DepthFitAll(a,b);break;
  case 24:v->FitAll(a,b,c,d);break;case 25:v->AxialScale(ia,ib,v3_axe(c));break;case 26:v->SetCenter(ia,ib);break;
  case 27:v->SetUp(a,b,c);break;case 28:v->ZFitAll(a);break;case 29:v->Translate(v3_axe(b),a,c!=0);break;
  case 30:v->Move(v3_axe(b),a,c!=0);break;case 31:v->Zoom(ia,ib,ic,id);break;
  case 32:v->Camera()->SetProjectionType(Graphic3d_Camera::Projection_Perspective);break;case 33:v->SetFocale(a);break;
  case 34:v->Reset(false);break;case 35:v->SetFront();break;case 36:v->Turn(v3_axe(b),a,c!=0);break;
  case 37:v->Rotate(v3_axe(b),a,c!=0);break;case 38:v->Rotate(a,b,c,d,0.0,0.0,true);break;
  case 39:v->WindowFitAll(ia,ib,ic,id);break;default:v->SetUp(v3_orientation(a));break;}}
// `W H N (Code A B C D)*N`: a view of a WxH window on the null driver; after each operation,
// its exception type (if any) and the camera queries.
void v3_view_case(std::istream& input){
 GaReader r{input};int w=r.integer(),h=r.integer(),n=r.integer();
 occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());viewer->SetDefaultLights();viewer->SetLightOn();
 occ::handle<V3d_View> view=viewer->CreateView();occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow();
 window->SetSize(w,h);view->SetWindow(window);
 for(int i=0;i<n;++i){int code=r.integer();double a=r.real(),b=r.real(),c=r.real(),d=r.real();
  GA_TRY(v3_view_op(view,code,a,b,c,d))GA_TRY(v3_view_state(view,w,h))}
 ga_end();}
