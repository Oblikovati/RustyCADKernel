// Exact pinned exception payloads; no invalid memory is dereferenced.
#include <NCollection_AliasedArray.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <string>
using Array = NCollection_AliasedArray<>;
template <class F> void observe(const char *name, F operation) {
  std::cout << name << "|";
  try {
    operation();
    std::cout << "OK";
  } catch (const Standard_Failure &failure) {
    std::cout << failure.ExceptionType() << "|" << failure.what() << "|"
              << !std::string(failure.GetStackString()).empty();
  }
  std::cout << "\n";
}
void constructor_cases() {
  observe("stride_zero", [] { Array a(0); });
  observe("size_zero", [] { Array a(4, size_t(0)); });
  observe("both_zero", [] { Array a(0, size_t(0)); });
  observe("signed_both_zero", [] { Array a(0, 0); });
  observe("signed_negative", [] { Array a(4, -1); });
  observe("borrowed_zero", [] {
    int value = 0;
    Array a(&value, size_t(0));
  });
}
void mutation_cases() {
  observe("resize_zero", [] {
    Array a(4, 2);
    a.Resize(size_t(0), true);
  });
  observe("resize_signed_zero", [] {
    Array a(4, 2);
    a.Resize(0, false);
  });
  observe("assign_stride", [] {
    Array a(4, 2), b(8, 2);
    a.Assign(b);
  });
  observe("assign_count", [] {
    Array a(4, 2), b(4, 3);
    a.Assign(b);
  });
}
void shape_cases() {
  constructor_cases();
  mutation_cases();
}
void pointer_cases(Array &a) {
  observe("data_type", [&] { a.Data<double>(); });
  observe("change_data_type", [&] { a.ChangeData<double>(); });
  observe("init_type", [&] { a.Init(double(1)); });
  observe("raw_signed", [&] { a.value(-1); });
  observe("raw_unsigned", [&] { a.value(size_t(2)); });
  observe("raw_mut_signed", [&] { a.changeValue(-1); });
  observe("raw_mut_unsigned", [&] { a.changeValue(size_t(2)); });
}
#define TYPED_CASES(Name)                                                      \
  observe(#Name "_negative_type", [&] { a.Name<double>(-1); });                \
  observe(#Name "_positive_type", [&] { a.Name<double>(2); });                 \
  observe(#Name "_negative", [&] { a.Name<int>(-1); });                        \
  observe(#Name "_positive", [&] { a.Name<int>(2); });                         \
  observe(#Name "_unsigned_type", [&] { a.Name<double>(size_t(2)); });         \
  observe(#Name "_unsigned", [&] { a.Name<int>(size_t(2)); });
void typed_cases(Array &a) {
  TYPED_CASES(Value)
  TYPED_CASES(Value2)
  TYPED_CASES(ChangeValue)
  TYPED_CASES(ChangeValue2)
}
void output_cases(Array &a) {
  double wide = 91;
  int narrow = 73;
  observe("output_negative_type", [&] { a.Value(-1, wide); });
  observe("output_positive_type", [&] { a.Value(2, wide); });
  observe("output_unsigned", [&] { a.Value(size_t(2), narrow); });
  observe("output2_negative_type", [&] { a.Value2(-1, wide); });
  observe("output2_positive_type", [&] { a.Value2(2, wide); });
  observe("output2_unsigned", [&] { a.Value2(size_t(2), narrow); });
  std::cout << "outputs_unchanged|" << wide << "|" << narrow << "\n";
}
void stack_cases() {
  Standard_Failure::SetDefaultStackTraceLength(3);
  Array a(4, 2);
  observe("type", [&] { a.Value<double>(size_t(0)); });
  observe("range", [] { Array a(0); });
  observe("dimension", [&] {
    Array b(8, 2);
    a.Assign(b);
  });
  observe("index", [&] { a.Value<int>(-1); });
}
int main(int argc, char **) {
  if (argc > 1) {
    stack_cases();
    return 0;
  }
  Standard_Failure::SetDefaultStackTraceLength(0);
  shape_cases();
  Array a(4, 2);
  pointer_cases(a);
  typed_cases(a);
  output_cases(a);
}
