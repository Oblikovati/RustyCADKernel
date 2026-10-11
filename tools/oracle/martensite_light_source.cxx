// Native state oracle for Martensite; OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <AIS_LightSource.hxx>
#include <Graphic3d_CLight.hxx>
#include <Graphic3d_TransformPers.hxx>
#include <gp_Ax1.hxx>
#include <iostream>
#include <iomanip>
#include <limits>
class LightProbe : public AIS_LightSource {
public:
  LightProbe(const occ::handle<Graphic3d_CLight>& light) : AIS_LightSource(light) {}
  void sync() { updateLightTransformPersistence(); updateLightLocalTransformation(); }
  void assign(const gp_Trsf& trsf) { setLocalTransformation(new TopLoc_Datum3D(trsf)); }
};
static void snapshot(const occ::handle<LightProbe>& object, int kind, int zoom, int head, int step) {
  const auto pers = object->TransformPersistence();
  std::cout << kind << ' ' << zoom << ' ' << head << ' ' << step << ' ';
  std::cout << object->Size() << ' ' << object->ArcSize() << ' ' << object->NbSplitsQuadric() << ' ';
  std::cout << object->NbSplitsArrow() << ' ' << object->NbArrows() << ' ';
  std::cout << (pers.IsNull() ? 0 : int(pers->Mode())) << ' ';
  gp_Pnt anchor;
  if (!pers.IsNull() && pers->Mode() == Graphic3d_TMF_ZoomPers) anchor = pers->AnchorPoint();
  std::cout << anchor.X() << ' ' << anchor.Y() << ' ' << anchor.Z() << ' ';
  const gp_Trsf trsf = object->LocalTransformation();
  for(int row=1; row<=3; ++row) for(int col=1; col<=4; ++col) std::cout << trsf.Value(row,col) << ' ';
  const auto light = object->Light();
  const gp_Pnt pos = light->Position();
  std::cout << pos.X() << ' ' << pos.Y() << ' ' << pos.Z() << ' ';
  const gp_Dir dir = kind == 0 || kind == 2 ? gp::DZ() : light->Direction();
  std::cout << dir.X() << ' ' << dir.Y() << ' ' << dir.Z() << '\n';
}
static void properties() {
  for(int kind=0;kind<4;++kind) {
    const occ::handle<Graphic3d_CLight> light = new Graphic3d_CLight(Graphic3d_TypeOfLightSource(kind));
    const occ::handle<LightProbe> object = new LightProbe(light);
    object->SetDisplayName(false); object->SetDisplayRange(false); object->SetSwitchOnClick(false);
    object->SetSize(std::numeric_limits<double>::quiet_NaN()); object->SetArcSize(-2);
    object->SetNbArrows(7); object->SetNbSplitsQuadric(-4); object->SetNbSplitsArrow(0);
    object->SetMarkerType(Aspect_TOM_STAR,true);
    object->SetMarkerImage(occ::handle<Graphic3d_MarkerImage>(),true);
    object->SetMarkerImage(occ::handle<Graphic3d_MarkerImage>(),false);
    std::cout << kind << ' ' << object->ToDisplayName() << ' ' << object->ToDisplayRange() << ' ' << object->ToSwitchOnClick() << ' ' << object->Size() << ' ' << object->ArcSize() << ' ' << object->NbArrows() << ' ' << object->NbSplitsQuadric() << ' ' << object->NbSplitsArrow() << ' ' << int(object->MarkerType(true)) << ' ' << int(object->MarkerType(false)) << '\n';
  }
}
int main(int argc, char**) {
  if(argc>1) { properties(); return 0; }
  std::cout << std::setprecision(17);
  for(int kind=0; kind<4; ++kind) for(int zoom=0; zoom<2; ++zoom) for(int head=0; head<2; ++head) {
    if (kind == 0 && head) continue;
    const occ::handle<Graphic3d_CLight> light = new Graphic3d_CLight(Graphic3d_TypeOfLightSource(kind));
    if (kind != 0) { light->SetDisplayPosition(gp_Pnt(2,3,4)); light->SetHeadlight(head != 0); }
    if (kind == 1 || kind == 3) light->SetDirection(gp_Dir(1,2,-3));
    const occ::handle<LightProbe> object = new LightProbe(light);
    object->SetZoomable(zoom != 0);
    object->sync(); snapshot(object,kind,zoom,head,0);
    gp_Trsf trsf; trsf.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(1,2,3)),0.7);
    trsf.SetTranslationPart(gp_Vec(5,6,7));
    object->assign(trsf); snapshot(object,kind,zoom,head,1);
    object->sync(); snapshot(object,kind,zoom,head,2);
  }
}
