//! Native Gleam Language Subsystem for Tagisan.
//!
//! Provides AST representations, fast lexer and recursive-descent parser, Hindley-Milner
//! type inference and unification, exhaustive pattern matching verification,
//! Erlang BEAM codegen, binary External Term Format (ETF) wire serialization,
//! and strongly-typed OTP Actor integration.

pub mod ast;
pub mod codegen;
pub mod parser;
pub mod runtime;
pub mod types;

pub use ast::{
    BinOp, CallArgument, CaseClause, ConstantDefinition, Constructor, ConstructorField,
    Expression, FunctionDefinition, FunctionParam, GleamModule, GleamType, Import,
    Literal, Pattern, PatternArg, Statement, TypeDefinition,
};
pub use codegen::{
    to_erlang_var, to_snake_case, CodegenError, ErlangCodeGen, EtfSynthesizer,
};
pub use parser::{parse_gleam_source, Lexer, ParseError, Parser, SpannedToken, Token};
pub use runtime::{GleamActor, GleamEvaluator, GleamValue, MessageValidator, RuntimeError};
pub use types::{TypeEnvironment, TypeError};
