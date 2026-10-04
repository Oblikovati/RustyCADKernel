#include "Schema.h"
#include "clang/AST/ASTContext.h"
#include "clang/AST/DeclCXX.h"
#include "clang/AST/DeclFriend.h"
#include "clang/AST/DeclOpenMP.h"
#include "clang/AST/DeclTemplate.h"
#include "clang/UnifiedSymbolResolution/USRGeneration.h"
#include "clang/Lex/Lexer.h"
#include "llvm/Support/FormatVariadic.h"
#include "llvm/Support/raw_ostream.h"
#include <cctype>
#include <system_error>

namespace api_inventory {
namespace {
std::string accessName(clang::AccessSpecifier access) {
  switch (access) {
  case clang::AS_public: return "public";
  case clang::AS_protected: return "protected";
  case clang::AS_private: return "private";
  case clang::AS_none: return "none";
  }
  return "unhandled";
}

llvm::json::Array ancestorAccess(const clang::Decl &declaration);

std::string usrFor(const clang::Decl *declaration) {
  llvm::SmallString<128> buffer;
  if (clang::index::generateUSRForDecl(declaration, buffer)) return "";
  return std::string(buffer.str());
}

std::string typeText(clang::QualType type, const clang::ASTContext &context,
                     bool canonical) {
  if (type.isNull()) return "";
  clang::PrintingPolicy policy(context.getLangOpts());
  policy.SuppressTagKeyword = false;
  policy.FullyQualifiedName = true;
  clang::QualType printed = canonical ? type.getCanonicalType() : type;
  return printed.getAsString(policy);
}

std::string declarationType(const clang::Decl &declaration,
                            const clang::ASTContext &context) {
  if (const auto *value = llvm::dyn_cast<clang::ValueDecl>(&declaration))
    return typeText(value->getType(), context, true);
  if (const auto *alias = llvm::dyn_cast<clang::TypedefNameDecl>(&declaration))
    return typeText(alias->getUnderlyingType(), context, true);
  if (const auto *templ = llvm::dyn_cast<clang::TemplateDecl>(&declaration))
    return templ->getQualifiedNameAsString();
  return "";
}

std::string directAccess(const clang::Decl &declaration) {
  const auto *named = llvm::dyn_cast<clang::NamedDecl>(&declaration);
  if (!named) return "none";
  if (named->getAccess() != clang::AS_none) return accessName(named->getAccess());
  if (const auto *record = llvm::dyn_cast<clang::CXXRecordDecl>(named)) {
    const auto *classTemplate = record->getDescribedClassTemplate();
    // Clang's pattern record may omit access that the template wrapper retains.
    if (classTemplate && classTemplate->getAccess() != clang::AS_none)
      return accessName(classTemplate->getAccess());
  }
  const auto *record = llvm::dyn_cast<clang::CXXRecordDecl>(named->getDeclContext());
  if (!record) return "public";
  return record->isClass() ? "private_default" : "public_default";
}

bool isCandidateKind(const clang::Decl &declaration) {
  return llvm::isa<clang::NamespaceDecl, clang::NamespaceAliasDecl,
                   clang::RecordDecl, clang::EnumDecl, clang::EnumConstantDecl,
                   clang::FunctionDecl, clang::FieldDecl, clang::IndirectFieldDecl,
                   clang::UnresolvedUsingValueDecl,
                   clang::TypedefNameDecl, clang::UsingDecl,
                   clang::UsingDirectiveDecl, clang::UsingEnumDecl,
                   clang::ClassTemplateDecl, clang::FunctionTemplateDecl,
                   clang::TypeAliasTemplateDecl, clang::VarTemplateDecl,
                   clang::ClassTemplateSpecializationDecl,
                   clang::ClassTemplatePartialSpecializationDecl,
                   clang::VarTemplateSpecializationDecl,
                   clang::VarTemplatePartialSpecializationDecl,
                   clang::VarDecl,
                   clang::TemplateTypeParmDecl, clang::NonTypeTemplateParmDecl,
                   clang::TemplateTemplateParmDecl, clang::ConceptDecl>(&declaration);
}

bool hasLocalAncestor(const clang::Decl &declaration) {
  if (llvm::isa<clang::TemplateTypeParmDecl,
                clang::NonTypeTemplateParmDecl,
                clang::TemplateTemplateParmDecl>(&declaration))
    return false;
  const clang::DeclContext *context = declaration.getDeclContext();
  while (context && !context->isTranslationUnit()) {
    const auto *owner = clang::Decl::castFromDeclContext(context);
    if (llvm::isa<clang::FunctionDecl, clang::BlockDecl,
                  clang::CapturedDecl>(owner))
      return true;
    context = context->getParent();
  }
  return false;
}

std::string exclusionFor(const clang::Decl &declaration) {
  if (declaration.isImplicit()) return "implicit_declaration";
  if (llvm::isa<clang::ParmVarDecl>(&declaration)) return "function_parameter";
  if (llvm::isa<clang::TemplateTypeParmDecl,
                clang::NonTypeTemplateParmDecl,
                clang::TemplateTemplateParmDecl>(&declaration))
    return "template_parameter_metadata";
  if (const auto *variable = llvm::dyn_cast<clang::VarDecl>(&declaration))
    if (!variable->isFileVarDecl() && !variable->isStaticDataMember())
      return "local_variable";
  if (hasLocalAncestor(declaration)) return "local_scope";
  if (llvm::isa<clang::AccessSpecDecl>(&declaration)) return "access_marker";
  if (llvm::isa<clang::UsingShadowDecl>(&declaration)) return "using_shadow";
  if (llvm::isa<clang::FriendDecl>(&declaration)) return "friend_wrapper";
  if (llvm::isa<clang::StaticAssertDecl>(&declaration)) return "static_assert";
  if (llvm::isa<clang::LinkageSpecDecl, clang::ExternCContextDecl>(&declaration))
    return "declaration_context";
  if (llvm::isa<clang::TranslationUnitDecl>(&declaration)) return "translation_unit";
  if (llvm::isa<clang::ImportDecl>(&declaration)) return "module_import";
  if (llvm::isa<clang::LabelDecl, clang::EmptyDecl, clang::BlockDecl,
                clang::CapturedDecl, clang::OMPThreadPrivateDecl>(&declaration))
    return "non_api_declaration";
  if (!isCandidateKind(declaration)) return "unhandled_decl_kind";
  const std::string access = directAccess(declaration);
  if (access == "private" || access == "private_default") return "private_access";
  for (const llvm::json::Value &ancestor : ancestorAccess(declaration)) {
    const auto *owner = ancestor.getAsObject();
    if (!owner) continue;
    const auto accessValue = owner->getString("access");
    if (accessValue && (accessValue->contains("private"))) return "private_ancestor";
  }
  return "";
}

bool isTemplatePattern(const clang::Decl &declaration) {
  if (const auto *function = llvm::dyn_cast<clang::FunctionDecl>(&declaration))
    return function->getDescribedFunctionTemplate() != nullptr;
  if (const auto *record = llvm::dyn_cast<clang::CXXRecordDecl>(&declaration))
    return record->getDescribedClassTemplate() != nullptr;
  if (const auto *variable = llvm::dyn_cast<clang::VarDecl>(&declaration))
    return variable->getDescribedVarTemplate() != nullptr;
  return false;
}

llvm::json::Array parameterTypes(const clang::FunctionDecl &function,
                                 const clang::ASTContext &context,
                                 bool canonical) {
  llvm::json::Array types;
  for (const clang::ParmVarDecl *parameter : function.parameters())
    types.push_back(typeText(parameter->getType(), context, canonical));
  return types;
}

std::string defaultArgumentState(const clang::ParmVarDecl &parameter) {
  if (parameter.hasUnparsedDefaultArg()) return "unparsed";
  if (parameter.hasUninstantiatedDefaultArg()) return "uninstantiated";
  if (!parameter.hasDefaultArg()) return "absent";
  return parameter.hasInheritedDefaultArg() ? "inherited" : "explicit";
}

std::string defaultArgumentExpression(const clang::ParmVarDecl &parameter,
                                      const clang::ASTContext &context) {
  if (!parameter.hasDefaultArg()) return "";
  const clang::SourceRange range = parameter.getDefaultArgRange();
  if (range.isInvalid()) return "";
  bool invalid = false;
  const auto text = clang::Lexer::getSourceText(
      clang::CharSourceRange::getTokenRange(range), context.getSourceManager(),
      context.getLangOpts(), &invalid);
  return invalid ? "" : text.str();
}

llvm::json::Object defaultArgumentEvent(const clang::ParmVarDecl &parameter,
                                        unsigned index,
                                        const clang::ASTContext &context) {
  const std::string expression = defaultArgumentExpression(parameter, context);
  const std::string state = defaultArgumentState(parameter);
  return llvm::json::Object{
      {"parameter_index", static_cast<int64_t>(index)},
      {"state", state},
      {"expression", expression},
      {"expression_status",
       state == "absent" ? "not_applicable"
                         : expression.empty() ? "unavailable" : "captured"}};
}

llvm::json::Array defaultArguments(const clang::FunctionDecl &function,
                                   const clang::ASTContext &context) {
  llvm::json::Array arguments;
  for (unsigned index = 0; index < function.getNumParams(); ++index)
    arguments.push_back(defaultArgumentEvent(*function.getParamDecl(index), index,
                                             context));
  return arguments;
}

const clang::FunctionDecl *functionForApi(const clang::Decl &declaration) {
  if (const auto *function = llvm::dyn_cast<clang::FunctionDecl>(&declaration))
    return function;
  if (const auto *templ = llvm::dyn_cast<clang::FunctionTemplateDecl>(&declaration))
    return templ->getTemplatedDecl();
  return nullptr;
}

const clang::TemplateParameterList *templateParametersForApi(
    const clang::Decl &declaration) {
  if (const auto *templ = llvm::dyn_cast<clang::TemplateDecl>(&declaration))
    return templ->getTemplateParameters();
  if (const auto *function = llvm::dyn_cast<clang::FunctionDecl>(&declaration)) {
    const auto *templ = function->getDescribedFunctionTemplate();
    return templ ? templ->getTemplateParameters() : nullptr;
  }
  if (const auto *partial =
          llvm::dyn_cast<clang::ClassTemplatePartialSpecializationDecl>(
              &declaration))
    return partial->getTemplateParameters();
  if (const auto *partial =
          llvm::dyn_cast<clang::VarTemplatePartialSpecializationDecl>(
              &declaration))
    return partial->getTemplateParameters();
  return nullptr;
}

bool isIdentifierCharacter(char character) {
  return std::isalnum(static_cast<unsigned char>(character)) || character == '_';
}

std::string replaceIdentifier(const std::string &text, const std::string &name,
                              const std::string &replacement) {
  if (name.empty()) return text;
  std::string result;
  for (std::size_t index = 0; index < text.size();) {
    const bool matches = text.compare(index, name.size(), name) == 0;
    const bool leftBoundary = index == 0 || !isIdentifierCharacter(text[index - 1]);
    const std::size_t end = index + name.size();
    const bool rightBoundary = end == text.size() || !isIdentifierCharacter(text[end]);
    if (matches && leftBoundary && rightBoundary) {
      result += replacement;
      index = end;
    } else {
      result += text[index++];
    }
  }
  return result;
}

std::string canonicalSignature(const clang::Decl &declaration,
                               const clang::ASTContext &context);

std::string identitySignature(const clang::Decl &declaration,
                              const clang::ASTContext &context) {
  std::string signature = canonicalSignature(declaration, context);
  const auto *function = functionForApi(declaration);
  if (!function) return signature;
  for (const clang::FunctionDecl *redeclaration : function->redecls()) {
    for (unsigned index = 0; index < redeclaration->getNumParams(); ++index) {
      const clang::ParmVarDecl *parameter = redeclaration->getParamDecl(index);
      if (parameter->getIdentifier())
        signature = replaceIdentifier(signature, parameter->getNameAsString(),
                                       "__argument_" + std::to_string(index));
    }
  }
  return signature;
}

std::string templateParameterSignature(const clang::Decl &declaration,
                                       const clang::ASTContext &context) {
  const auto *parameters = templateParametersForApi(declaration);
  if (!parameters) return "";
  std::string signature;
  llvm::raw_string_ostream output(signature);
  clang::PrintingPolicy policy(context.getLangOpts());
  policy.FullyQualifiedName = true;
  parameters->print(output, context, policy);
  output.flush();
  std::string normalized = signature;
  for (unsigned index = 0; index < parameters->size(); ++index) {
    const auto *parameter = llvm::dyn_cast<clang::NamedDecl>(
        parameters->getParam(index));
    if (parameter && !parameter->getName().empty())
      normalized = replaceIdentifier(normalized, parameter->getNameAsString(),
                                     "__template_param_" + std::to_string(index));
  }
  return replaceIdentifier(normalized, "class", "typename");
}

llvm::json::Array ancestorAccess(const clang::Decl &declaration) {
  llvm::json::Array ancestors;
  const clang::DeclContext *context = declaration.getDeclContext();
  while (context && !context->isTranslationUnit()) {
    const auto *owner = clang::Decl::castFromDeclContext(context);
    if (const auto *named = llvm::dyn_cast<clang::NamedDecl>(owner)) {
      if (llvm::isa<clang::CXXRecordDecl, clang::EnumDecl>(owner)) {
        ancestors.push_back(llvm::json::Object{
            {"qualified_name", named->getQualifiedNameAsString()},
            {"usr", usrFor(named)},
            {"access", directAccess(*owner)}});
      }
    }
    context = context->getParent();
  }
  return ancestors;
}

std::string canonicalSignature(const clang::Decl &declaration,
                              const clang::ASTContext &context) {
  if (const auto *function = functionForApi(declaration))
    return typeText(function->getType(), context, true);
  return declarationType(declaration, context);
}

} // namespace

std::filesystem::path canonicalPath(const std::filesystem::path &path) {
  std::error_code error;
  auto canonical = std::filesystem::canonical(path, error);
  if (!error) return canonical;
  auto absolute = std::filesystem::absolute(path, error);
  return (error ? path : absolute).lexically_normal();
}

bool isUnder(const std::filesystem::path &path, const std::filesystem::path &root) {
  auto relative = path.lexically_relative(root);
  return !relative.empty() && *relative.begin() != ".." && !relative.is_absolute();
}

bool shouldEmit(const std::filesystem::path &path, const Config &config) {
  if (path == config.mainHeader || config.extraEmitHeaders.find(path) !=
                                        config.extraEmitHeaders.end())
    return true;
  const auto extension = path.extension().string();
  const bool fragment = extension == ".lxx" || extension == ".pxx" || extension == ".gxx";
  return fragment && isUnder(path, config.sourceRoot);
}

std::string declarationAccess(const clang::Decl &declaration) {
  return directAccess(declaration);
}

llvm::json::Object locationObject(clang::SourceLocation location,
                                  const clang::SourceManager &sources) {
  llvm::json::Object result;
  if (location.isInvalid()) return result;
  const clang::SourceLocation fileLocation = sources.getFileLoc(location);
  const clang::PresumedLoc presumed = sources.getPresumedLoc(fileLocation);
  if (presumed.isInvalid()) return result;
  result["file"] = canonicalPath(presumed.getFilename()).string();
  result["line"] = static_cast<int64_t>(presumed.getLine());
  result["column"] = static_cast<int64_t>(presumed.getColumn());
  return result;
}

llvm::json::Object declarationEvent(const clang::Decl &declaration,
                                    clang::ASTContext &context,
                                    const Config &config) {
  const clang::SourceManager &sources = context.getSourceManager();
  auto spelling = locationObject(sources.getSpellingLoc(declaration.getLocation()), sources);
  auto expansion = locationObject(sources.getExpansionLoc(declaration.getLocation()), sources);
  const auto expansionFile = expansion.getString("file");
  const std::string kind = declaration.getDeclKindName();
  std::string reason = exclusionFor(declaration);
  if (isTemplatePattern(declaration)) reason = "template_owned_declaration";
  const auto *named = llvm::dyn_cast<clang::NamedDecl>(&declaration);
  const auto *function = functionForApi(declaration);
  const std::string source = expansionFile ? expansionFile->str() : "";
  llvm::json::Object event{
      {"event", "declaration"},
      {"source", source},
      {"main_header", config.mainHeader.string()},
      {"usr", usrFor(&declaration)},
      {"qualified_name", named ? named->getQualifiedNameAsString() : ""},
      {"kind", kind},
      {"signature", canonicalSignature(declaration, context)},
      {"identity_signature", identitySignature(declaration, context)},
      {"template_signature", templateParameterSignature(declaration, context)},
      {"access", directAccess(declaration)},
      {"ancestor_access", ancestorAccess(declaration)},
      {"parent_usr", declaration.getDeclContext()->isTranslationUnit()
                          ? ""
                          : usrFor(clang::Decl::castFromDeclContext(
                                declaration.getDeclContext()))},
      {"implicit", declaration.isImplicit()},
      {"macro_generated", declaration.getLocation().isMacroID()},
      {"spelling_location", std::move(spelling)},
      {"expansion_location", std::move(expansion)},
      {"api_visible", reason.empty()},
      {"exclusion_reason", reason},
      {"canonical_type", declarationType(declaration, context)}};
  if (llvm::isa<clang::UnresolvedUsingValueDecl>(&declaration)) {
    event["api_resolution"] = "unresolved_using_value_target";
    event["callable_status"] = "unknown";
  }
  if (function) {
    event["parameter_types"] = parameterTypes(*function, context, true);
    event["written_parameter_types"] = parameterTypes(*function, context, false);
    event["default_arguments"] = defaultArguments(*function, context);
    event["variadic"] = function->isVariadic();
  } else {
    event["type"] = declarationType(declaration, context);
  }
  if (const auto *tag = llvm::dyn_cast<clang::TagDecl>(&declaration))
    event["tag_kind"] = tag->getKindName();
  return event;
}

llvm::json::Object headerSeenEvent(const std::filesystem::path &path,
                                   const Config &config) {
  return llvm::json::Object{{"event", "header_seen"},
                            {"source", path.string()},
                            {"main_header", config.mainHeader.string()},
                            {"emit_declarations", shouldEmit(path, config)}};
}

void writeEvent(llvm::json::Object event) {
  llvm::outs() << llvm::formatv("{0}\n", llvm::json::Value(std::move(event)));
}
} // namespace api_inventory
