// Cross-schema current-data behavior at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <FSD_BinaryFile.hxx>
#include <Storage_Schema.hxx>
#include <Storage_Data.hxx>
#include <Storage_TypeData.hxx>
#include <Storage_DefaultCallBack.hxx>
#include <Storage_StreamTypeMismatchError.hxx>
#include <iostream>

struct SchemaBindingProbe : Storage_Schema {
  using Storage_Schema::HasTypeBinding;
  using Storage_Schema::BindType;
  using Storage_Schema::TypeBinding;
};

struct FailingInfoDriver : FSD_BinaryFile {
  Storage_Error BeginWriteInfoSection() override {
    throw Storage_StreamTypeMismatchError("keep current data for binding probe");
  }
};

int main(int argc, char** argv)
{
  if (argc != 2) return 2;
  SchemaBindingProbe first, second;
  occ::handle<Storage_Data> current = new Storage_Data;
  occ::handle<FailingInfoDriver> driver = new FailingInfoDriver;
  driver->Open(argv[1], Storage_VSWrite);
  try { first.Write(driver, current); }
  catch (const Storage_StreamTypeMismatchError&) { std::cout << "write raised\n"; }
  occ::handle<Storage_CallBack> original = new Storage_DefaultCallBack;
  occ::handle<Storage_CallBack> replacement = new Storage_DefaultCallBack;
  std::cout << first.HasTypeBinding("A") << ' ' << second.HasTypeBinding("A") << '\n';
  first.BindType("A", original);
  std::cout << second.HasTypeBinding("A") << ' ' << (second.TypeBinding("A") == original) << '\n';
  second.BindType("A", replacement);
  std::cout << (first.TypeBinding("A") == original) << ' ' << current->TypeData()->NumberOfTypes() << '\n';
  second.BindType("B", occ::handle<Storage_CallBack>());
  std::cout << first.HasTypeBinding("B") << ' ' << first.TypeBinding("B").IsNull()
            << ' ' << first.TypeBinding("missing").IsNull() << ' '
            << current->TypeData()->NumberOfTypes() << '\n';
  driver->Close();
}
