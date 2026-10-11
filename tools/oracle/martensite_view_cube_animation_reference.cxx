// Martensite oracle: retained references are actual AIS_ViewCube::ViewAnimation slots.
#include <AIS_ViewCube.hxx>
#include <AIS_AnimationCamera.hxx>
#include <iostream>
class AnimationReferenceCube:public AIS_ViewCube {
public:
  void replaceProtected(const occ::handle<AIS_AnimationCamera>& animation) {myViewAnimation=animation;}
};
static int identity(const occ::handle<AIS_AnimationCamera>& value,
                    const occ::handle<AIS_AnimationCamera>& original,
                    const occ::handle<AIS_AnimationCamera>& first,
                    const occ::handle<AIS_AnimationCamera>& second) {
  return value.IsNull()?0:value==original?1:value==first?2:value==second?3:-1;
}
static void capture(int step,const AnimationReferenceCube& cube,
                    const occ::handle<AIS_AnimationCamera>& left,
                    const occ::handle<AIS_AnimationCamera>& right,
                    const occ::handle<AIS_AnimationCamera>& original,
                    const occ::handle<AIS_AnimationCamera>& first,
                    const occ::handle<AIS_AnimationCamera>& second,
                    const occ::handle<AIS_AnimationCamera>& copied) {
  std::cout<<step<<' '<<(&left==&right)<<' '<<(&left==&cube.ViewAnimation())<<' '
    <<identity(left,original,first,second)<<' '<<identity(right,original,first,second)<<' '
    <<left.IsNull()<<' '<<right.IsNull()<<' '<<identity(copied,original,first,second)<<' '
    <<original->GetRefCount()<<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<'\n';
}
int main() {
  AnimationReferenceCube cube;auto original=cube.ViewAnimation();auto copied=original;
  auto first=occ::handle<AIS_AnimationCamera>(new AIS_AnimationCamera("first",{}));
  auto second=occ::handle<AIS_AnimationCamera>(new AIS_AnimationCamera("second",{}));
  const auto& left=cube.ViewAnimation();const auto& right=cube.ViewAnimation();
  capture(0,cube,left,right,original,first,second,copied);
  cube.SetViewAnimation(first);capture(1,cube,left,right,original,first,second,copied);
  cube.SetViewAnimation({});capture(2,cube,left,right,original,first,second,copied);
  cube.SetViewAnimation(original);capture(3,cube,left,right,original,first,second,copied);
  cube.replaceProtected(second);capture(4,cube,left,right,original,first,second,copied);
  cube.replaceProtected({});capture(5,cube,left,right,original,first,second,copied);
  cube.replaceProtected(first);capture(6,cube,left,right,original,first,second,copied);
}
