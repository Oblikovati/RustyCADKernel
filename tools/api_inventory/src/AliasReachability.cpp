#include "AliasReachability.h"
#include "clang/AST/DeclTemplate.h"
#include "clang/AST/Type.h"
#include "clang/UnifiedSymbolResolution/USRGeneration.h"
#include "llvm/Support/raw_ostream.h"

namespace api_inventory {
namespace {
const clang::TemplateSpecializationType *
specializationOf(const clang::TypedefNameDecl &alias) {
  const clang::QualType type = alias.getUnderlyingType();
  return type->getAs<clang::TemplateSpecializationType>();
}

const clang::ClassTemplateDecl *
classTemplateOf(const clang::TemplateSpecializationType &specialization) {
  const clang::TemplateDecl *target =
      specialization.getTemplateName().getAsTemplateDecl();
  return llvm::dyn_cast_or_null<clang::ClassTemplateDecl>(target);
}

bool isExposableMember(const clang::NamedDecl &member) {
  if (member.isImplicit())
    return false;
  if (const auto *function = llvm::dyn_cast<clang::FunctionDecl>(&member))
    if (function->getDescribedFunctionTemplate())
      return false;
  const std::string access = declarationAccess(member);
  return access == "public" || access == "public_default" ||
         access == "protected";
}

const char *
templateArgumentScalarKind(const clang::TemplateArgument &argument) {
  switch (argument.getKind()) {
  case clang::TemplateArgument::Null:
    return "null";
  case clang::TemplateArgument::Type:
    return "type";
  case clang::TemplateArgument::Declaration:
    return "declaration";
  case clang::TemplateArgument::NullPtr:
    return "null_pointer";
  case clang::TemplateArgument::Integral:
    return "integral";
  case clang::TemplateArgument::StructuralValue:
    return "structural_value";
  default:
    return nullptr;
  }
}

const char *
templateArgumentCompositeKind(const clang::TemplateArgument &argument) {
  switch (argument.getKind()) {
  case clang::TemplateArgument::Template:
    return "template";
  case clang::TemplateArgument::TemplateExpansion:
    return "template_expansion";
  case clang::TemplateArgument::Expression:
    return "expression";
  case clang::TemplateArgument::Pack:
    return "pack";
  default:
    return "unknown";
  }
}

const char *templateArgumentKind(const clang::TemplateArgument &argument) {
  const char *scalar = templateArgumentScalarKind(argument);
  return scalar ? scalar : templateArgumentCompositeKind(argument);
}

std::string usrForAlias(const clang::TypedefNameDecl &alias) {
  llvm::SmallString<128> result;
  if (clang::index::generateUSRForDecl(&alias, result))
    return "";
  return std::string(result.str());
}

std::string usrForTarget(const clang::ClassTemplateDecl &target) {
  llvm::SmallString<128> result;
  if (clang::index::generateUSRForDecl(&target, result))
    return "";
  return std::string(result.str());
}

llvm::json::Object
specializationArgument(unsigned index, const clang::TemplateArgument &argument,
                       const clang::TemplateParameterList &parameters,
                       const clang::ASTContext &context);

llvm::json::Array
specializationArguments(const clang::ClassTemplateDecl &target,
                        const clang::TemplateSpecializationType &specialization,
                        const clang::ASTContext &context) {
  llvm::json::Array arguments;
  const auto *parameters = target.getTemplateParameters();
  for (unsigned index = 0; index < specialization.template_arguments().size();
       ++index)
    arguments.push_back(specializationArgument(
        index, specialization.template_arguments()[index], *parameters,
        context));
  return arguments;
}

llvm::json::Object
specializationArgument(unsigned index, const clang::TemplateArgument &argument,
                       const clang::TemplateParameterList &parameters,
                       const clang::ASTContext &context) {
  std::string rendered;
  llvm::raw_string_ostream output(rendered);
  clang::PrintingPolicy policy(context.getLangOpts());
  argument.print(policy, output, true);
  const auto parameterName =
      index < parameters.size()
          ? llvm::json::Value(parameters.getParam(index)->getNameAsString())
          : llvm::json::Value(nullptr);
  return llvm::json::Object{
      {"index", static_cast<int64_t>(index)},
      {"parameter", parameterName},
      {"parameter_mapping", index < parameters.size() ? "direct" : "unmapped"},
      {"kind", templateArgumentKind(argument)},
      {"text", output.str()},
      {"dependent", argument.isDependent()}};
}

bool targetIsHidden(const clang::ClassTemplateDecl &target,
                    clang::ASTContext &context, const Config &config) {
  const auto targetEvent = declarationEvent(target, context, config);
  if (targetEvent.getBoolean("api_visible").value_or(false))
    return false;
  const auto reason = targetEvent.getString("exclusion_reason");
  return reason && (reason->contains("private_access") ||
                    reason->contains("private_ancestor"));
}

llvm::json::Object exposedMember(const clang::NamedDecl &member,
                                 clang::ASTContext &context,
                                 const Config &config) {
  return llvm::json::Object{
      {"visibility_basis", "public_or_protected_member_of_alias_target"},
      {"api_visible_through_alias", true},
      {"declaration", declarationEvent(member, context, config)}};
}

llvm::json::Array exposedMembers(const clang::CXXRecordDecl &record,
                                 clang::ASTContext &context,
                                 const Config &config) {
  llvm::json::Array members;
  for (const clang::Decl *member : record.decls()) {
    const auto *named = llvm::dyn_cast<clang::NamedDecl>(member);
    if (!named || !isExposableMember(*named))
      continue;
    members.push_back(exposedMember(*named, context, config));
  }
  return members;
}

std::string aliasSource(const clang::TypedefNameDecl &alias,
                        const clang::ASTContext &context) {
  const auto &sources = context.getSourceManager();
  const auto location = sources.getExpansionLoc(alias.getLocation());
  return canonicalPath(sources.getFilename(location).str()).string();
}

void appendAliasIdentity(llvm::json::Object &event,
                         const clang::TypedefNameDecl &alias,
                         const clang::ClassTemplateDecl &target) {
  event["alias_usr"] = usrForAlias(alias);
  event["alias_qualified_name"] = alias.getQualifiedNameAsString();
  event["target_template_usr"] = usrForTarget(target);
  event["target_template_qualified_name"] = target.getQualifiedNameAsString();
}

void appendProofLimits(llvm::json::Object &event) {
  event["member_surface_basis"] = "primary_template_pattern";
  event["exposure_scope"] = "direct_public_and_protected_members";
  event["template_constraints_evaluated"] = false;
  event["member_constraints_evaluated"] = false;
  event["proof_status"] = "supplemental_unresolved";
  event["coverage_credit"] = false;
  event["status"] = "private_template_alias_members_recorded";
}

llvm::json::Object
reachabilityEvent(const clang::TypedefNameDecl &alias,
                  const clang::TemplateSpecializationType &specialization,
                  const clang::ClassTemplateDecl &target,
                  clang::ASTContext &context, const Config &config) {
  const auto *record = target.getTemplatedDecl()->getDefinition();
  llvm::json::Object event{{"event", "alias_reachability"},
                           {"source", aliasSource(alias, context)},
                           {"main_header", config.mainHeader.string()}};
  appendAliasIdentity(event, alias, target);
  event["target_definition_available"] = record != nullptr;
  event["specialization_arguments"] =
      specializationArguments(target, specialization, context);
  event["exposed_members"] =
      record ? exposedMembers(*record, context, config) : llvm::json::Array{};
  appendProofLimits(event);
  return event;
}

} // namespace

std::vector<llvm::json::Object>
aliasReachabilityEvents(const clang::TypedefNameDecl &alias,
                        clang::ASTContext &context, const Config &config) {
  const auto specialization = specializationOf(alias);
  if (!specialization)
    return {};
  const auto target = classTemplateOf(*specialization);
  if (!target || !targetIsHidden(*target, context, config))
    return {};
  return {reachabilityEvent(alias, *specialization, *target, context, config)};
}
} // namespace api_inventory
