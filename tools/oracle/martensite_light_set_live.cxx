// Local CPU oracle for Martensite: OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Graphic3d_LightSet.hxx>
#include <Quantity_Color.hxx>
#include <Standard_NoSuchObject.hxx>
#include <Standard_OutOfRange.hxx>
#include <Standard_ProgramError.hxx>
#include <Standard_NotImplemented.hxx>
#include <array>
#include <cstring>
#include <iostream>
#include <sstream>
#include <vector>
using Light = occ::handle<Graphic3d_CLight>;
class Cursor : public Graphic3d_LightSet::Iterator {
public:
  using Graphic3d_LightSet::Iterator::Iterator;
  using Graphic3d_LightSet::Iterator::operator=;
  void skip()
  {
    // The oracle exposes the native protected recheck without changing iteration.
    skipFiltered();
  }
};
struct LiveProbe {
  occ::handle<Graphic3d_LightSet> set = new Graphic3d_LightSet;
  std::array<Light, 6> lights;
  Cursor cursor, other;
  Graphic3d_LightSet::IterationFilter filter;
};
static LiveProbe create(int filter) {
  LiveProbe probe;
  probe.filter = Graphic3d_LightSet::IterationFilter(filter);
  for (int id = 0; id < 6; ++id) {
    probe.lights[id] = new Graphic3d_CLight(Graphic3d_TypeOfLightSource(id % 4));
    probe.lights[id]->SetIntensity(float(id + 1));
  }
  probe.lights[0]->SetColor(Quantity_Color(0.25, 0.5, 0.75, Quantity_TOC_RGB));
  probe.lights[4]->SetColor(Quantity_Color(0.75, 0.25, 0.5, Quantity_TOC_RGB));
  probe.lights[1]->SetCastShadows(true);
  probe.lights[3]->SetCastShadows(true);
  probe.lights[2]->SetEnabled(false);
  return probe;
}
static int ordinal(const LiveProbe& probe, const Light& light) {
  for (int id = 0; id < 6; ++id) {
    if (probe.lights[id] == light) return id;
  }
  return -1;
}
static int cursor_value(const LiveProbe& probe, const Cursor& cursor) {
  try { return ordinal(probe, cursor.Value()); }
  catch (const Standard_NoSuchObject&) { return -11; }
}
static int mutate_source(LiveProbe& probe, const std::string& action, int id, int value) {
  const Light light = id < 0 ? Light() : probe.lights.at(id);
  if (action == "add") return probe.set->Add(light);
  if (action == "remove") return probe.set->Remove(light);
  if (action == "contains") return probe.set->Contains(light);
  if (action == "enable") light->SetEnabled(value != 0);
  if (action == "shadow") light->SetCastShadows(value != 0);
  if (action == "intensity") light->SetIntensity(float(value));
  if (action == "color") light->SetColor(probe.lights.at(value)->Color());
  return 0;
}
static int apply(LiveProbe& probe, const std::string& action, int id, int value) {
  if (action == "start") probe.cursor = Cursor(*probe.set, probe.filter);
  else if (action == "handle") probe.cursor = Cursor(probe.set, probe.filter);
  else if (action == "null") probe.cursor = Cursor(occ::handle<Graphic3d_LightSet>(), probe.filter);
  else if (action == "default") probe.cursor = Cursor();
  else if (action == "next") probe.cursor.Next();
  else if (action == "skip") probe.cursor.skip();
  else if (action == "copy") probe.other = probe.cursor;
  else if (action == "othernext") probe.other.Next();
  else if (action == "update") return int(probe.set->UpdateRevision());
  else if (action == "index") return ordinal(probe, probe.set->Value(id));
  else return mutate_source(probe, action, id, value);
  return 0;
}
static int apply_checked(LiveProbe& probe, const std::string& action, int id, int value) {
  try { return apply(probe, action, id, value); }
  catch (const Standard_NoSuchObject&) { return -11; }
  catch (const Standard_OutOfRange&) { return -12; }
  catch (const Standard_NotImplemented&) { return -14; }
  catch (const Standard_ProgramError&) { return -13; }
}
static void emit_counts(const LiveProbe& probe) {
  const auto set = probe.set;
  std::cout << set->Revision() << ' ' << set->Extent() << ' ';
  for (int kind = 0; kind < 4; ++kind) {
    std::cout << set->NbLightsOfType(Graphic3d_TypeOfLightSource(kind)) << ' ';
    std::cout << set->NbEnabledLightsOfType(Graphic3d_TypeOfLightSource(kind)) << ' ';
  }
  std::cout << set->NbEnabled() << ' ' << set->NbCastShadows() << ' ';
  for (int axis = 0; axis < 4; ++axis) {
    const float value = set->AmbientColor()[axis];
    uint32_t bits; std::memcpy(&bits, &value, sizeof(bits));
    std::cout << bits << ' ';
  }
}
static void emit_keys(const LiveProbe& probe) {
  const auto set = probe.set;
  std::cout << (set->KeyEnabledLong().IsEmpty() ? "-" : set->KeyEnabledLong().ToCString()) << ' ';
  std::cout << (set->KeyEnabledShort().IsEmpty() ? "-" : set->KeyEnabledShort().ToCString()) << ' ';
  if (set->IsEmpty()) std::cout << '-';
  for (int index = set->Lower(); index <= set->Upper(); ++index) {
    if (index > 1) std::cout << ',';
    std::cout << ordinal(probe, set->Value(index));
  }
  std::cout << '\n';
}
static void replay(int filter, const std::vector<std::string>& events) {
  LiveProbe probe = create(filter);
  for (const auto& event : events) {
    std::istringstream stream(event);
    std::string action; int id, value; stream >> action >> id >> value;
    const int result = apply_checked(probe, action, id, value);
    std::cout << filter << ' ' << event << ' ' << result << ' ';
    std::cout << probe.cursor.More() << ' ' << cursor_value(probe, probe.cursor) << ' ';
    std::cout << probe.other.More() << ' ' << cursor_value(probe, probe.other) << ' ';
    emit_counts(probe); emit_keys(probe);
  }
}
int main() {
  std::vector<std::string> events;
  for (std::string event; std::getline(std::cin, event);) events.push_back(event);
  for (const int filter : {0, 2, 4, 8, 6, 14}) replay(filter, events);
}
