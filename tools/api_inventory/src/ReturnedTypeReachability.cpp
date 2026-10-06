#include "ReturnedTypeReachability.h"
#include "clang/AST/Attr.h"
#include "clang/AST/DeclTemplate.h"
#include "clang/AST/Type.h"

namespace api_inventory {
namespace {
const clang::FunctionDecl *returnedFunction(const clang::Decl &declaration) {
  if (const auto *pattern =
          llvm::dyn_cast<clang::FunctionTemplateDecl>(&declaration))
    return pattern->getTemplatedDecl();
  return llvm::dyn_cast<clang::FunctionDecl>(&declaration);
}

clang::QualType exposedType(const clang::Decl &declaration) {
  if (const auto *function = returnedFunction(declaration))
    return function->getReturnType();
  if (const auto *alias = llvm::dyn_cast<clang::TypedefNameDecl>(&declaration))
    return alias->getUnderlyingType();
  return {};
}

clang::QualType unwrapType(clang::QualType type, llvm::json::Array &wrappers) {
  while (type->isPointerType() || type->isReferenceType()) {
    wrappers.push_back(type->isPointerType()           ? "pointer"
                       : type->isLValueReferenceType() ? "lvalue_reference"
                                                       : "rvalue_reference");
    type = type->getPointeeType();
  }
  return type;
}

bool privatelyNamed(const llvm::json::Object &event) {
  const auto reason = event.getString("exclusion_reason").value_or("");
  return reason == "private_access" || reason == "private_ancestor";
}

bool memberHasOwnIdentity(const clang::Decl &member) {
  if (member.isImplicit() || !llvm::isa<clang::NamedDecl>(member))
    return false;
  if (const auto *function = llvm::dyn_cast<clang::FunctionDecl>(&member))
    return !function->getDescribedFunctionTemplate();
  if (const auto *record = llvm::dyn_cast<clang::CXXRecordDecl>(&member))
    return !record->getDescribedClassTemplate();
  return !llvm::isa<clang::UsingShadowDecl>(member);
}

std::string memberExposure(const clang::Decl &member,
                           const clang::CXXRecordDecl &record) {
  const auto access = declarationAccess(member);
  if (access == "private" || access == "private_default")
    return "private_member_inaccessible";
  if (access == "protected")
    return record.hasAttr<clang::FinalAttr>()
               ? "protected_member_of_final_type"
               : "requires_derived_inferred_type";
  return "inferred_type_public_member";
}

void withholdReachabilityCredit(llvm::json::Object &event) {
  event["coverage_credit"] = false;
  event["proof_status"] = "supplemental_unresolved";
  event["private_type_name_accessible"] = false;
  event["call_constraints_evaluated"] = false;
  event["reachability_complete"] = false;
  event["limitations"] = llvm::json::Array{
      "base_class_members_not_traversed",
      "construction_cv_and_call_constraints_not_evaluated",
      "field_types_and_nested_type_closure_not_traversed",
      "function_and_member_pointer_type_closure_not_traversed"};
}

llvm::json::Object exposureEvent(const clang::Decl &seed,
                                 const clang::Decl &producer,
                                 clang::QualType type,
                                 clang::ASTContext &context,
                                 const Config &config) {
  auto declaration = declarationEvent(producer, context, config);
  llvm::json::Object event{
      {"event", "returned_type_reachability"},
      {"source", declaration.getString("source").value_or("").str()},
      {"main_header", config.mainHeader.string()},
      {"seed_declaration", declarationEvent(seed, context, config)},
      {"producer_declaration", std::move(declaration)},
      {"exposure_kind", llvm::isa<clang::TypedefNameDecl>(producer)
                            ? "type_alias"
                            : "return_type"},
      {"written_exposed_type", type.getAsString()}};
  withholdReachabilityCredit(event);
  return event;
}

class ReturnedExposureWalker {
public:
  ReturnedExposureWalker(const clang::Decl &seed, clang::ASTContext &context,
                         const Config &config)
      : seed_(seed), context_(context), config_(config) {}
  void append(const clang::Decl &producer);
  std::vector<llvm::json::Object> events;

private:
  void appendTarget(llvm::json::Object event, clang::QualType type,
                    const clang::CXXRecordDecl *record);
  void appendRecord(llvm::json::Object event,
                    const clang::CXXRecordDecl &record);
  llvm::json::Array membersOf(const clang::CXXRecordDecl &record);
  const clang::Decl &seed_;
  clang::ASTContext &context_;
  const Config &config_;
  std::set<const clang::CXXRecordDecl *> visited_;
};

void ReturnedExposureWalker::append(const clang::Decl &producer) {
  const auto original = exposedType(producer);
  if (original.isNull())
    return;
  llvm::json::Array wrappers;
  const auto type = unwrapType(original, wrappers);
  const auto *record = type->getAsCXXRecordDecl();
  if (!record && !type->isDependentType())
    return;
  if (record && !privatelyNamed(declarationEvent(*record, context_, config_)))
    return;
  auto event = exposureEvent(seed_, producer, original, context_, config_);
  event["type_wrappers"] = std::move(wrappers);
  appendTarget(std::move(event), type, record);
}

void ReturnedExposureWalker::appendTarget(llvm::json::Object event,
                                          clang::QualType type,
                                          const clang::CXXRecordDecl *record) {
  if (type->isDependentType() ||
      (record && llvm::isa<clang::ClassTemplateSpecializationDecl>(record))) {
    event["status"] = "unsupported_template_or_dependent_type";
    if (record)
      event["target_declaration"] =
          declarationEvent(*record, context_, config_);
    else
      event["private_type_name_accessible"] = nullptr;
    events.push_back(std::move(event));
    return;
  }
  appendRecord(std::move(event), *record);
}

llvm::json::Array
ReturnedExposureWalker::membersOf(const clang::CXXRecordDecl &record) {
  llvm::json::Array members;
  for (const auto *member : record.decls()) {
    if (!memberHasOwnIdentity(*member))
      continue;
    const auto access = memberExposure(*member, record);
    members.push_back(llvm::json::Object{
        {"access_basis", access},
        {"requires_callability_review", true},
        {"declaration", declarationEvent(*member, context_, config_)}});
    if (access == "inferred_type_public_member" ||
        access == "requires_derived_inferred_type")
      append(*member);
  }
  return members;
}

void ReturnedExposureWalker::appendRecord(llvm::json::Object event,
                                          const clang::CXXRecordDecl &record) {
  event["target_declaration"] = declarationEvent(record, context_, config_);
  const auto *definition = record.getDefinition();
  const bool fresh = visited_.insert(record.getCanonicalDecl()).second;
  event["target_definition_available"] = definition != nullptr;
  event["status"] = !definition ? "target_definition_unavailable"
                    : !fresh    ? "cycle_or_repeated_target"
                                : "private_record_members_recorded";
  if (definition && fresh)
    event["members"] = membersOf(*definition);
  events.push_back(std::move(event));
}
} // namespace

std::vector<llvm::json::Object>
returnedTypeReachabilityEvents(const clang::Decl &declaration,
                               clang::ASTContext &context,
                               const Config &config) {
  ReturnedExposureWalker walker(declaration, context, config);
  walker.append(declaration);
  return std::move(walker.events);
}
} // namespace api_inventory
