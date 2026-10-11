// OCCT 3d097a0 camera gesture and walking oracle; normal Rust tests replay committed assets.
#include <AIS_MouseGesture.hxx>
#include <AIS_NavigationMode.hxx>
#include <AIS_RotationMode.hxx>
#include <AIS_WalkDelta.hxx>
#include <iostream>
#include <iomanip>
#include <limits>
#include <cmath>
static void part(const AIS_WalkPart& p) {
  std::cout << ' ' << p.Value << ' ' << p.Pressure << ' ' << p.Duration << ' ' << p.IsEmpty();
}
static void delta(const std::string& name, const AIS_WalkDelta& d) {
  std::cout << name << ' ' << d.IsDefined() << ' ' << d.ToMove() << ' ' << d.ToRotate() << ' ' << d.IsEmpty()
    << ' ' << d.IsJumping() << ' ' << d.IsCrouching() << ' ' << d.IsRunning();
  for (int i=0;i<3;++i) part(d[static_cast<AIS_WalkTranslation>(i)]);
  for (int i=0;i<3;++i) part(d[static_cast<AIS_WalkRotation>(i)]);
  std::cout << '\n';
}
static void enums() {
  std::cout << "navigation " << AIS_NavigationMode_Orbit << ' ' << AIS_NavigationMode_FirstPersonFlight << ' ' << AIS_NavigationMode_FirstPersonWalk << ' ' << AIS_NavigationMode_LOWER << ' ' << AIS_NavigationMode_UPPER << '\n';
  std::cout << "rotation " << AIS_RotationMode_BndBoxActive << ' ' << AIS_RotationMode_PickLast << ' ' << AIS_RotationMode_PickCenter << ' ' << AIS_RotationMode_CameraAt << ' ' << AIS_RotationMode_BndBoxScene << ' ' << AIS_RotationMode_LOWER << ' ' << AIS_RotationMode_UPPER << '\n';
  std::cout << "mouse " << AIS_MouseGesture_NONE << ' ' << AIS_MouseGesture_SelectRectangle << ' ' << AIS_MouseGesture_SelectLasso << ' ' << AIS_MouseGesture_Zoom << ' ' << AIS_MouseGesture_ZoomVertical << ' ' << AIS_MouseGesture_ZoomWindow << ' ' << AIS_MouseGesture_Pan << ' ' << AIS_MouseGesture_RotateOrbit << ' ' << AIS_MouseGesture_RotateView << ' ' << AIS_MouseGesture_Drag << '\n';
  std::cout << "axes " << AIS_WalkTranslation_Forward << ' ' << AIS_WalkTranslation_Side << ' ' << AIS_WalkTranslation_Up << ' ' << AIS_WalkRotation_Yaw << ' ' << AIS_WalkRotation_Pitch << ' ' << AIS_WalkRotation_Roll << '\n';
}
static void part_cases() {
  AIS_WalkPart p; std::cout << "part-default"; part(p); std::cout << '\n';
  const double small=RealSmall(), below=std::nextafter(small,0.), above=std::nextafter(small,1.);
  const double values[]={0.,-0.,std::numeric_limits<double>::denorm_min(),-std::numeric_limits<double>::denorm_min(),below,-below,small,-small,above,-above,1e-300,-1e-300,INFINITY,-INFINITY,NAN};
  for (int i=0;i<15;++i) {
    p.Value=values[i]; p.Pressure=.25; p.Duration=7.; std::cout << "part-" << i; part(p); std::cout << '\n';
  }
}
static void flag_cases() {
  for (int mask=0;mask<16;++mask) {
    AIS_WalkDelta d; d.SetDefined(mask&1); d.SetJumping(mask&2); d.SetCrouching(mask&4); d.SetRunning(mask&8);
    delta("flags-"+std::to_string(mask),d);
  }
}
static void axis_cases() {
  for (int i=0;i<6;++i) {
    AIS_WalkDelta d;
    auto& p=i<3 ? d[static_cast<AIS_WalkTranslation>(i)] : d[static_cast<AIS_WalkRotation>(i-3)];
    p.Value=i<3 ? .125*(i+1) : -.125*(i+1); p.Pressure=.25+i; p.Duration=.5+i;
    delta("axis-"+std::to_string(i),d);
    auto copy=d; p.Value=0.; delta("axis-cleared-"+std::to_string(i),d); delta("axis-copy-"+std::to_string(i),copy);
  }
}
int main() {
  std::cout << std::setprecision(17);
  enums(); part_cases(); flag_cases(); axis_cases();
  for (int i=0;i<3;++i) {
    AIS_WalkDelta d; d[AIS_WalkTranslation_Forward].Value=i==0 ? NAN : i==1 ? INFINITY : -INFINITY;
    delta("nonfinite-"+std::to_string(i),d);
  }
}
