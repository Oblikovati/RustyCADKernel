// Callback subclass RTTI and virtual dispatch at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <FSD_BinaryFile.hxx>
#include <Standard_Persistent.hxx>
#include <Storage_BaseDriver.hxx>
#include <Storage_CallBack.hxx>
#include <Storage_Data.hxx>
#include <Storage_Schema.hxx>
#include <Storage_StreamTypeMismatchError.hxx>
#include <Standard_Transient.hxx>
#include <iostream>

struct SchemaBindingProbe : Storage_Schema {
  using Storage_Schema::BindType;
  using Storage_Schema::TypeBinding;
};

struct FailingInfoDriver : FSD_BinaryFile {
  Storage_Error BeginWriteInfoSection() override {
    throw Storage_StreamTypeMismatchError("retain callback probe data");
  }
};

class TestDispatchCallBack : public Storage_CallBack {
public:
  occ::handle<Standard_Persistent> New() const override {
    ++myCalls;
    return new Standard_Persistent;
  }
  void Add(const occ::handle<Standard_Persistent>&,
           const occ::handle<Storage_Schema>&) const override { ++myCalls; }
  void Write(const occ::handle<Standard_Persistent>&,
             const occ::handle<Storage_BaseDriver>&,
             const occ::handle<Storage_Schema>&) const override { ++myCalls; }
  void Read(const occ::handle<Standard_Persistent>&,
            const occ::handle<Storage_BaseDriver>&,
            const occ::handle<Storage_Schema>&) const override { ++myCalls; }
  int Calls() const { return myCalls; }
  DEFINE_STANDARD_RTTIEXT(TestDispatchCallBack, Storage_CallBack)

private:
  mutable int myCalls = 0;
};

IMPLEMENT_STANDARD_RTTIEXT(TestDispatchCallBack, Storage_CallBack)

int main(int argc, char** argv) {
  if (argc != 2) return 2;
  SchemaBindingProbe schema;
  occ::handle<Storage_Data> current = new Storage_Data;
  occ::handle<FailingInfoDriver> driver = new FailingInfoDriver;
  driver->Open(argv[1], Storage_VSWrite);
  try { schema.Write(driver, current); }
  catch (const Storage_StreamTypeMismatchError&) {}

  occ::handle<TestDispatchCallBack> concrete = new TestDispatchCallBack;
  occ::handle<Storage_CallBack> original = concrete.operator->();
  schema.BindType("dispatch", original);
  std::cout << original->GetRefCount() << '\n';
  const auto bound = schema.TypeBinding("dispatch");
  std::cout << bound->GetRefCount() << ' '
            << (bound->This() == bound.operator->()) << '\n';
  const auto derived = occ::down_cast<TestDispatchCallBack>(bound);
  const occ::handle<Standard_Transient> base = bound.operator->();
  std::cout << bound->GetRefCount() << ' '
            << (derived == concrete) << ' ' << (base == bound) << '\n';
  const auto persistent = bound->New();
  bound->Add(persistent, {});
  bound->Write(persistent, {}, {});
  bound->Read(persistent, {}, {});
  std::cout << (bound == original) << ' ' << derived->Calls() << '\n';
  std::cout << bound->DynamicType()->Name() << '\n';
  std::cout << bound->IsKind(STANDARD_TYPE(Storage_CallBack)) << '\n';
  driver->Close();
}
