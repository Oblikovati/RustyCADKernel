#include "UpstreamRetirement.h"
#include "clang/AST/ASTContext.h"
#include "clang/AST/Attr.h"
#include "clang/AST/DeclTemplate.h"
#include "clang/AST/RawCommentList.h"

namespace api_inventory {
namespace {
const clang::Decl &markedDeclaration(const clang::Decl &declaration) {
  if (const auto *wrapper = llvm::dyn_cast<clang::TemplateDecl>(&declaration)) {
    // Template-template parameters have no templated declaration to unwrap.
    if (const auto *wrapped = wrapper->getTemplatedDecl())
      return *wrapped;
  }
  return declaration;
}

llvm::json::Object markerObject(const clang::Decl &owner,
                                clang::SourceLocation location,
                                clang::ASTContext &context, bool inherited) {
  const auto *named = llvm::dyn_cast<clang::NamedDecl>(&owner);
  const auto &sources = context.getSourceManager();
  return llvm::json::Object{
      {"owner", named ? named->getQualifiedNameAsString() : ""},
      {"inherited_from_owner", inherited},
      {"location", locationObject(sources.getExpansionLoc(location), sources)}};
}

void appendDeprecatedAttribute(const clang::Decl &owner,
                               clang::ASTContext &context, bool inherited,
                               llvm::json::Array &markers) {
  const auto *attribute = owner.getAttr<clang::DeprecatedAttr>();
  if (!attribute)
    return;
  auto marker =
      markerObject(owner, attribute->getLocation(), context, inherited);
  marker["kind"] = "deprecated_attribute";
  marker["text"] = attribute->getMessage().str();
  markers.push_back(std::move(marker));
}

void appendRetirementDocumentation(const clang::Decl &owner,
                                   clang::ASTContext &context, bool inherited,
                                   llvm::json::Array &markers) {
  const auto *comment = context.getRawCommentForAnyRedecl(&owner);
  if (!comment)
    return;
  const auto text = comment->getRawText(context.getSourceManager());
  if (!text.contains_insensitive("deprecated") &&
      !text.contains_insensitive("obsolete"))
    return;
  auto marker = markerObject(owner, comment->getBeginLoc(), context, inherited);
  marker["kind"] = "documentation_candidate";
  marker["text"] = text.str();
  markers.push_back(std::move(marker));
}
} // namespace

llvm::json::Array upstreamRetirementMarkers(const clang::Decl &declaration,
                                            clang::ASTContext &context) {
  llvm::json::Array markers;
  const clang::Decl *owner = &declaration;
  while (owner && !llvm::isa<clang::TranslationUnitDecl>(owner)) {
    const auto &marked = markedDeclaration(*owner);
    appendDeprecatedAttribute(marked, context, owner != &declaration, markers);
    appendRetirementDocumentation(marked, context, owner != &declaration,
                                  markers);
    const auto *parent = owner->getDeclContext();
    owner = parent ? clang::Decl::castFromDeclContext(parent) : nullptr;
  }
  return markers;
}
} // namespace api_inventory
