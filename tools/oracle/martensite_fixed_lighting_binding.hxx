#pragma once
// Adjacent group model updates must not rebind lights as if they were WorldView updates.
static void fixed_adjacent_groups(const NativeFixedLightScene& fixture,const std::string& input) {
  if(input.rfind("lit-adjacent-",0)!=0)return;
  const std::string mode="lit-"+input.substr(13);
  const auto group=fixture.structure->NewGroup();
  group->SetGroupPrimitivesAspect(fixture.aspect);
  group->AddPrimitiveArray(lighting_vertices(mode,false));
  gp_Trsf rotation;rotation.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(0,1,0)),.7);
  group->SetTransformation(rotation);
}
