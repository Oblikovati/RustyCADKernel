#include <BOPAlgo_Builder.hxx>
#include <BOPAlgo_ToolsProvider.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <NCollection_List.hxx>
#include <TopoDS_Shape.hxx>
#include <iostream>
#include <iomanip>

static int list_size(const NCollection_List<TopoDS_Shape>& shapes) {
  return shapes.Extent();
}

static NCollection_List<TopoDS_Shape> singleton(const TopoDS_Shape& shape) {
  NCollection_List<TopoDS_Shape> shapes;
  shapes.Append(shape);
  return shapes;
}

static void report_builder_state(const TopoDS_Shape& shape) {
  BOPAlgo_Builder builder;
  builder.AddArgument(shape);
  builder.AddArgument(shape.Reversed());
  std::cout << "builder_add " << list_size(builder.Arguments()) << '\n';

  const NCollection_List<TopoDS_Shape> reset = singleton(shape);
  builder.SetArguments(reset);
  std::cout << "builder_set_after_add " << list_size(builder.Arguments()) << '\n';
  builder.Clear();
  builder.SetArguments(reset);
  std::cout << "builder_set_after_clear " << list_size(builder.Arguments()) << '\n';

  builder.SetNonDestructive(true);
  builder.SetGlue(BOPAlgo_GlueShift);
  builder.SetCheckInverted(false);
  builder.SetToFillHistory(false);
  builder.SetFuzzyValue(0.25);
  builder.Clear();
  std::cout << "builder_clear " << list_size(builder.Arguments()) << ' '
            << builder.NonDestructive() << ' ' << builder.Glue() << ' '
            << builder.CheckInverted() << ' ' << builder.HasHistory() << ' '
            << std::setprecision(17) << builder.FuzzyValue() << '\n';
}

static void report_tool_state(const TopoDS_Shape& shape) {
  BOPAlgo_ToolsProvider provider;
  provider.AddTool(shape);
  provider.AddTool(shape.Reversed());
  std::cout << "tools_add " << list_size(provider.Tools()) << '\n';

  const NCollection_List<TopoDS_Shape> reset = singleton(shape);
  provider.SetTools(reset);
  std::cout << "tools_set " << list_size(provider.Tools()) << '\n';
  provider.Clear();
  std::cout << "tools_clear " << list_size(provider.Tools()) << ' '
            << list_size(provider.Arguments()) << '\n';
  provider.SetTools(reset);
  std::cout << "tools_set_after_clear " << list_size(provider.Tools()) << '\n';
}

int main() {
  const TopoDS_Shape box = BRepPrimAPI_MakeBox(1.0, 1.0, 1.0).Shape();
  report_builder_state(box);
  report_tool_state(box);
}
