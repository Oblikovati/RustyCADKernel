// Rust translation of ModelingAlgorithms/TKExpress/Expr at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Expr` package: symbolic expressions, functions and relations.
//!
//! Each class is one module named after it in snake case (`Expr_ArcSine` is
//! `arc_sine::ExprArcSine`); the abstract classes are traits (`ExprGeneralExpression`,
//! `ExprUnaryExpression`, `ExprBinaryExpression`, `ExprPolyExpression`,
//! `ExprNamedExpression`, `ExprGeneralFunction`, `ExprGeneralRelation`,
//! `ExprSingleRelation`). The package functions of `Expr` are in [`package`] and the
//! operators of `Expr_Operators.hxx` in [`operators`].
pub mod absolute;
pub mod arc_cosine;
pub mod arc_sine;
pub mod arc_tangent;
pub mod arg_cosh;
pub mod arg_sinh;
pub mod arg_tanh;
pub mod binary_expression;
pub mod binary_function;
pub mod cosh;
pub mod cosine;
pub mod difference;
pub mod division;
pub mod equal;
pub mod errors;
pub mod exponential;
pub mod exponentiate;
pub mod function_derivative;
pub mod general_expression;
pub mod general_function;
pub mod general_relation;
pub mod handles;
pub mod log_of10;
pub mod log_of_e;
pub mod named_constant;
pub mod named_expression;
pub mod named_function;
pub mod named_unknown;
pub mod numeric_value;
pub mod operators;
pub mod package;
pub mod poly_expression;
pub mod poly_function;
pub mod product;
pub mod relation_iterator;
pub mod ru_iterator;
pub mod sign;
pub mod sine;
pub mod single_relation;
pub mod sinh;
pub mod square;
pub mod square_root;
pub mod sum;
pub mod system_relation;
pub mod tangent;
pub mod tanh;
pub mod text;
pub mod unary_expression;
pub mod unary_function;
pub mod unary_kit;
pub mod unary_minus;
pub mod unknown_iterator;

pub use absolute::ExprAbsolute;
pub use arc_cosine::ExprArcCosine;
pub use arc_sine::ExprArcSine;
pub use arc_tangent::ExprArcTangent;
pub use arg_cosh::ExprArgCosh;
pub use arg_sinh::ExprArgSinh;
pub use arg_tanh::ExprArgTanh;
pub use binary_expression::ExprBinaryExpression;
pub use binary_function::ExprBinaryFunction;
pub use cosh::ExprCosh;
pub use cosine::ExprCosine;
pub use difference::ExprDifference;
pub use division::ExprDivision;
pub use equal::ExprEqual;
pub use exponential::ExprExponential;
pub use exponentiate::ExprExponentiate;
pub use function_derivative::ExprFunctionDerivative;
pub use general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
pub use general_function::{ExprFunctionHandle, ExprGeneralFunction};
pub use general_relation::{ExprGeneralRelation, ExprRelationHandle};
pub use handles::ExprHandle;
pub use log_of_e::ExprLogOfe;
pub use log_of10::ExprLogOf10;
pub use named_constant::ExprNamedConstant;
pub use named_expression::ExprNamedExpression;
pub use named_function::ExprNamedFunction;
pub use named_unknown::ExprNamedUnknown;
pub use numeric_value::ExprNumericValue;
pub use poly_expression::ExprPolyExpression;
pub use poly_function::ExprPolyFunction;
pub use product::ExprProduct;
pub use relation_iterator::ExprRelationIterator;
pub use ru_iterator::ExprRUIterator;
pub use sign::ExprSign;
pub use sine::ExprSine;
pub use single_relation::ExprSingleRelation;
pub use sinh::ExprSinh;
pub use square::ExprSquare;
pub use square_root::ExprSquareRoot;
pub use sum::ExprSum;
pub use system_relation::ExprSystemRelation;
pub use tangent::ExprTangent;
pub use tanh::ExprTanh;
pub use unary_expression::ExprUnaryExpression;
pub use unary_function::ExprUnaryFunction;
pub use unary_minus::ExprUnaryMinus;
pub use unknown_iterator::ExprUnknownIterator;
