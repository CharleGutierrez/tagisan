//! Gleam Abstract Syntax Tree (AST) definitions for Tagisan.
//!
//! Provides strongly-typed representations for Gleam modules, custom algebraic
//! data types (ADTs), expressions, pattern matching, and type annotations.

use std::fmt;
use serde::{Deserialize, Serialize};

/// Representation of a Gleam Type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GleamType {
    Int,
    Float,
    String,
    Bool,
    Nil,
    List(Box<GleamType>),
    Tuple(Vec<GleamType>),
    Custom {
        name: String,
        module: Option<String>,
        params: Vec<GleamType>,
    },
    Function {
        params: Vec<GleamType>,
        return_type: Box<GleamType>,
    },
    Var(usize),
    Generic(String),
}

impl GleamType {
    pub fn int() -> Self {
        GleamType::Int
    }

    pub fn float() -> Self {
        GleamType::Float
    }

    pub fn string() -> Self {
        GleamType::String
    }

    pub fn bool() -> Self {
        GleamType::Bool
    }

    pub fn nil() -> Self {
        GleamType::Nil
    }

    pub fn list(inner: GleamType) -> Self {
        GleamType::List(Box::new(inner))
    }

    pub fn tuple(elements: Vec<GleamType>) -> Self {
        GleamType::Tuple(elements)
    }

    pub fn custom(name: impl Into<String>, params: Vec<GleamType>) -> Self {
        GleamType::Custom {
            name: name.into(),
            module: None,
            params,
        }
    }

    pub fn custom_with_module(module: impl Into<String>, name: impl Into<String>, params: Vec<GleamType>) -> Self {
        GleamType::Custom {
            name: name.into(),
            module: Some(module.into()),
            params,
        }
    }

    pub fn function(params: Vec<GleamType>, return_type: GleamType) -> Self {
        GleamType::Function {
            params,
            return_type: Box::new(return_type),
        }
    }

    pub fn result(ok: GleamType, error: GleamType) -> Self {
        GleamType::Custom {
            name: "Result".to_string(),
            module: None,
            params: vec![ok, error],
        }
    }

    pub fn subject(msg_type: GleamType) -> Self {
        GleamType::Custom {
            name: "Subject".to_string(),
            module: Some("process".to_string()),
            params: vec![msg_type],
        }
    }

    pub fn is_generic(&self) -> bool {
        match self {
            GleamType::Generic(_) | GleamType::Var(_) => true,
            GleamType::List(inner) => inner.is_generic(),
            GleamType::Tuple(elems) => elems.iter().any(|t| t.is_generic()),
            GleamType::Custom { params, .. } => params.iter().any(|t| t.is_generic()),
            GleamType::Function { params, return_type } => {
                params.iter().any(|t| t.is_generic()) || return_type.is_generic()
            }
            _ => false,
        }
    }
}

impl fmt::Display for GleamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GleamType::Int => write!(f, "Int"),
            GleamType::Float => write!(f, "Float"),
            GleamType::String => write!(f, "String"),
            GleamType::Bool => write!(f, "Bool"),
            GleamType::Nil => write!(f, "Nil"),
            GleamType::List(inner) => write!(f, "List({})", inner),
            GleamType::Tuple(elems) => {
                write!(f, "#(")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            GleamType::Custom { name, module, params } => {
                if let Some(m) = module {
                    write!(f, "{}.", m)?;
                }
                write!(f, "{}", name)?;
                if !params.is_empty() {
                    write!(f, "(")?;
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", p)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            GleamType::Function { params, return_type } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", return_type)
            }
            GleamType::Var(id) => write!(f, "?{}", id),
            GleamType::Generic(name) => write!(f, "{}", name),
        }
    }
}

/// A field within an ADT constructor
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstructorField {
    pub label: Option<String>,
    pub type_: GleamType,
}

impl ConstructorField {
    pub fn new(label: Option<String>, type_: GleamType) -> Self {
        Self { label, type_ }
    }

    pub fn positional(type_: GleamType) -> Self {
        Self { label: None, type_ }
    }

    pub fn labeled(label: impl Into<String>, type_: GleamType) -> Self {
        Self {
            label: Some(label.into()),
            type_,
        }
    }
}

/// A variant constructor within a custom type definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Constructor {
    pub name: String,
    pub fields: Vec<ConstructorField>,
}

impl Constructor {
    pub fn new(name: impl Into<String>, fields: Vec<ConstructorField>) -> Self {
        Self {
            name: name.into(),
            fields,
        }
    }

    pub fn unit(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
    }

    pub fn arity(&self) -> usize {
        self.fields.len()
    }
}

/// Custom Type / ADT Definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeDefinition {
    pub is_pub: bool,
    pub is_opaque: bool,
    pub name: String,
    pub parameters: Vec<String>,
    pub constructors: Vec<Constructor>,
}

impl TypeDefinition {
    pub fn new(name: impl Into<String>, constructors: Vec<Constructor>) -> Self {
        Self {
            is_pub: true,
            is_opaque: false,
            name: name.into(),
            parameters: Vec::new(),
            constructors,
        }
    }

    pub fn with_params(
        name: impl Into<String>,
        parameters: Vec<String>,
        constructors: Vec<Constructor>,
    ) -> Self {
        Self {
            is_pub: true,
            is_opaque: false,
            name: name.into(),
            parameters,
            constructors,
        }
    }

    pub fn find_constructor(&self, name: &str) -> Option<&Constructor> {
        self.constructors.iter().find(|c| c.name == name)
    }
}

/// Literals in Gleam
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(i) => write!(f, "{}", i),
            Literal::Float(fl) => write!(f, "{:?}", fl),
            Literal::String(s) => write!(f, "{:?}", s),
            Literal::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            Literal::Nil => write!(f, "Nil"),
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinOp {
    // Integer arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Float arithmetic
    AddFloat,
    SubFloat,
    MulFloat,
    DivFloat,
    // Equality
    Eq,
    NotEq,
    // Relational
    Lt,
    LtEq,
    Gt,
    GtEq,
    // Boolean logic
    And,
    Or,
    // Gleam-specific
    Pipe,   // |>
    Concat, // <>
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::AddFloat => write!(f, "+."),
            BinOp::SubFloat => write!(f, "-."),
            BinOp::MulFloat => write!(f, "*."),
            BinOp::DivFloat => write!(f, "/."),
            BinOp::Eq => write!(f, "=="),
            BinOp::NotEq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::LtEq => write!(f, "<="),
            BinOp::Gt => write!(f, ">"),
            BinOp::GtEq => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Pipe => write!(f, "|>"),
            BinOp::Concat => write!(f, "<>"),
        }
    }
}

/// Pattern argument in Constructor pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternArg {
    pub label: Option<String>,
    pub pattern: Pattern,
}

impl PatternArg {
    pub fn new(label: Option<String>, pattern: Pattern) -> Self {
        Self { label, pattern }
    }

    pub fn positional(pattern: Pattern) -> Self {
        Self {
            label: None,
            pattern,
        }
    }
}

/// Pattern in pattern matching (`case` clauses, `let` bindings)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Constructor {
        name: String,
        module: Option<String>,
        args: Vec<PatternArg>,
    },
    Variable(String),
    Literal(Literal),
    Discard(Option<String>),
    Tuple(Vec<Pattern>),
    List {
        elements: Vec<Pattern>,
        tail: Option<Box<Pattern>>,
    },
    Or(Box<Pattern>, Box<Pattern>),
}

impl Pattern {
    pub fn is_wildcard(&self) -> bool {
        matches!(self, Pattern::Discard(_) | Pattern::Variable(_))
    }

    pub fn constructor_name(&self) -> Option<&str> {
        match self {
            Pattern::Constructor { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }
}

/// Case clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseClause {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
}

/// Function argument in a call
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallArgument {
    pub label: Option<String>,
    pub value: Expression,
}

impl CallArgument {
    pub fn positional(value: Expression) -> Self {
        Self { label: None, value }
    }

    pub fn labeled(label: impl Into<String>, value: Expression) -> Self {
        Self {
            label: Some(label.into()),
            value,
        }
    }
}

/// Expressions in Gleam
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Variable(String),
    Literal(Literal),
    BinOp {
        op: BinOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<CallArgument>,
    },
    RecordCreation {
        constructor: String,
        module: Option<String>,
        fields: Vec<(String, Expression)>,
    },
    FieldAccess {
        record: Box<Expression>,
        field: String,
    },
    TupleAccess {
        tuple: Box<Expression>,
        index: usize,
    },
    Case {
        subject: Box<Expression>,
        clauses: Vec<CaseClause>,
    },
    Block(Vec<Statement>),
    Tuple(Vec<Expression>),
    List(Vec<Expression>),
    AnonymousFunction {
        parameters: Vec<FunctionParam>,
        return_type: Option<GleamType>,
        body: Vec<Statement>,
    },
    Panic(Option<String>),
    Todo(Option<String>),
}

impl Expression {
    pub fn var(name: impl Into<String>) -> Self {
        Expression::Variable(name.into())
    }

    pub fn int(val: i64) -> Self {
        Expression::Literal(Literal::Int(val))
    }

    pub fn float(val: f64) -> Self {
        Expression::Literal(Literal::Float(val))
    }

    pub fn string(val: impl Into<String>) -> Self {
        Expression::Literal(Literal::String(val.into()))
    }

    pub fn bool(val: bool) -> Self {
        Expression::Literal(Literal::Bool(val))
    }

    pub fn nil() -> Self {
        Expression::Literal(Literal::Nil)
    }

    pub fn call(function: Expression, arguments: Vec<CallArgument>) -> Self {
        Expression::FunctionCall {
            function: Box::new(function),
            arguments,
        }
    }

    pub fn binop(op: BinOp, left: Expression, right: Expression) -> Self {
        Expression::BinOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn field(record: Expression, field: impl Into<String>) -> Self {
        Expression::FieldAccess {
            record: Box::new(record),
            field: field.into(),
        }
    }
}

/// Statements in a block or function body
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Let {
        pattern: Pattern,
        type_annotation: Option<GleamType>,
        value: Expression,
    },
    Expression(Expression),
    Use {
        patterns: Vec<Pattern>,
        call: Expression,
    },
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub label: Option<String>,
    pub type_annotation: Option<GleamType>,
}

impl FunctionParam {
    pub fn new(name: impl Into<String>, type_annotation: Option<GleamType>) -> Self {
        Self {
            name: name.into(),
            label: None,
            type_annotation,
        }
    }

    pub fn with_label(
        label: impl Into<String>,
        name: impl Into<String>,
        type_annotation: Option<GleamType>,
    ) -> Self {
        Self {
            name: name.into(),
            label: Some(label.into()),
            type_annotation,
        }
    }
}

/// Top-level function definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub is_pub: bool,
    pub name: String,
    pub parameters: Vec<FunctionParam>,
    pub return_type: Option<GleamType>,
    pub body: Vec<Statement>,
}

impl FunctionDefinition {
    pub fn new(
        name: impl Into<String>,
        parameters: Vec<FunctionParam>,
        return_type: Option<GleamType>,
        body: Vec<Statement>,
    ) -> Self {
        Self {
            is_pub: true,
            name: name.into(),
            parameters,
            return_type,
            body,
        }
    }
}

/// Import statement
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Import {
    pub module: String,
    pub alias: Option<String>,
    pub unqualified: Vec<String>,
}

impl Import {
    pub fn new(module: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            alias: None,
            unqualified: Vec::new(),
        }
    }

    pub fn with_alias(module: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            alias: Some(alias.into()),
            unqualified: Vec::new(),
        }
    }

    pub fn with_unqualified(module: impl Into<String>, unqualified: Vec<String>) -> Self {
        Self {
            module: module.into(),
            alias: None,
            unqualified,
        }
    }
}

/// Constant definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantDefinition {
    pub is_pub: bool,
    pub name: String,
    pub type_annotation: Option<GleamType>,
    pub value: Expression,
}

/// A complete Gleam Module
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GleamModule {
    pub name: String,
    pub imports: Vec<Import>,
    pub types: Vec<TypeDefinition>,
    pub constants: Vec<ConstantDefinition>,
    pub functions: Vec<FunctionDefinition>,
}

impl GleamModule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            imports: Vec::new(),
            types: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn add_type(&mut self, type_def: TypeDefinition) {
        self.types.push(type_def);
    }

    pub fn add_function(&mut self, func_def: FunctionDefinition) {
        self.functions.push(func_def);
    }

    pub fn find_type(&self, name: &str) -> Option<&TypeDefinition> {
        self.types.iter().find(|t| t.name == name)
    }

    pub fn find_function(&self, name: &str) -> Option<&FunctionDefinition> {
        self.functions.iter().find(|f| f.name == name)
    }
}
