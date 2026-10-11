// AIS_TextLabel oracle at OCCT 3d097a0; protected placement is exposed only for observation.
#include <AIS_TextLabel.hxx>
#include <AIS_InteractiveContext.hxx>
#include <OpenGl_GraphicDriver.hxx>
#include <Aspect_DisplayConnection.hxx>
#include <V3d_Viewer.hxx>
#include <Prs3d_TextAspect.hxx>
#include <Prs3d_Drawer.hxx>
#include <Graphic3d_AspectText3d.hxx>
#include <iostream>
#include <iomanip>
class LabelProbe : public AIS_TextLabel {
public:
  using AIS_TextLabel::calculateLabelParams;
  using AIS_TextLabel::calculateLabelTrsf;
};
static void emit_case(const occ::handle<LabelProbe>& label, int mode) {
  label->SetHJustification(Graphic3d_HorizontalTextAlignment(mode % 3));
  label->SetVJustification(Graphic3d_VerticalTextAlignment(mode / 3));
  gp_Pnt center; double width=0.0, height=0.0;
  const bool valid=label->calculateLabelParams(label->Position(),center,width,height);
  const gp_Trsf plane=label->calculateLabelTrsf(label->Position(),center);
  std::cout << mode << ' ' << valid << ' ' << width << ' ' << height;
  std::cout << ' ' << center.X() << ' ' << center.Y() << ' ' << center.Z();
  for(int row=1;row<=3;++row) for(int col=1;col<=4;++col) std::cout << ' ' << plane.Value(row,col);
  std::cout << '\n';
}
int main() {
  auto display=occ::handle<Aspect_DisplayConnection>(new Aspect_DisplayConnection());
  auto driver=occ::handle<OpenGl_GraphicDriver>(new OpenGl_GraphicDriver(display,false));
  auto viewer=occ::handle<V3d_Viewer>(new V3d_Viewer(driver));
  auto context=occ::handle<AIS_InteractiveContext>(new AIS_InteractiveContext(viewer));
  auto label=occ::handle<LabelProbe>(new LabelProbe());
  label->SetContext(context); label->SetText(TCollection_ExtendedString("AV\tlabel\nsecond"));
  label->SetFont("DejaVu Sans"); label->SetHeight(16.0); label->SetAngle(0.37);
  label->SetOrientation3D(gp_Ax2(gp_Pnt(2.0,3.0,4.0),gp_Dir(0.0,1.0,0.0),gp_Dir(1.0,0.0,0.0)));
  std::cout << std::setprecision(17);
  for(int mode=0;mode<12;++mode) emit_case(label,mode);
}
