//! Gleam Actor Model Runtime and OTP Bridge for Tagisan.
//!
//! Bridges Gleam actor definitions (`actor.Spec`, message handlers) to Tagisan's native
//! BEAM/OTP runtime (`src/otp/actor.rs` and `src/otp/supervisor.rs`). Enforces strict
//! type-checked actor messaging: incoming messages MUST conform to the declared `pub type Msg`
//! schema before reaching the actor mailbox.

use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::gleam::ast::*;
use crate::gleam::codegen::to_snake_case;
use crate::gleam::parser::parse_gleam_source;
use crate::gleam::types::{TypeEnvironment, TypeError};
use crate::otp::actor::{ActorError, GenServer};
use crate::otp::etf::Term;
use crate::otp::supervisor::ChildSpec;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuntimeError {
    #[error("Gleam parse error: {0}")]
    ParseError(String),

    #[error("Gleam type error: {0}")]
    TypeError(#[from] TypeError),

    #[error("Actor message protocol violation: expected variant of '{expected_type}', got term '{term}': {reason}")]
    ProtocolViolation {
        expected_type: String,
        term: String,
        reason: String,
    },

    #[error("Runtime evaluation error: {0}")]
    EvaluationError(String),

    #[error("Actor crashed intentionally: {0}")]
    ActorCrashed(String),

    #[error("Unknown function or constructor: {0}")]
    UnknownSymbol(String),
}

// -----------------------------------------------------------------------------
// Gleam Value (Runtime Value Representation)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GleamValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
    Tuple(Vec<GleamValue>),
    List(Vec<GleamValue>),
    Constructor {
        name: String,
        fields: Vec<GleamValue>,
    },
    Map(HashMap<String, GleamValue>),
}

impl fmt::Display for GleamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GleamValue::Int(i) => write!(f, "{}", i),
            GleamValue::Float(fl) => write!(f, "{:?}", fl),
            GleamValue::String(s) => write!(f, "{:?}", s),
            GleamValue::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            GleamValue::Nil => write!(f, "Nil"),
            GleamValue::Tuple(elems) => {
                write!(f, "#(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            GleamValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            GleamValue::Constructor { name, fields } => {
                write!(f, "{}", name)?;
                if !fields.is_empty() {
                    write!(f, "(")?;
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", field)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            GleamValue::Map(m) => write!(f, "{:?}", m),
        }
    }
}

impl GleamValue {
    pub fn to_term(&self) -> Term {
        match self {
            GleamValue::Int(i) => Term::int(*i),
            GleamValue::Float(f) => Term::float(*f),
            GleamValue::String(s) => Term::string(s.clone()),
            GleamValue::Bool(b) => Term::boolean(*b),
            GleamValue::Nil => Term::nil(),
            GleamValue::Tuple(elems) => {
                let terms = elems.iter().map(|e| e.to_term()).collect();
                Term::tuple(terms)
            }
            GleamValue::List(items) => {
                let terms = items.iter().map(|e| e.to_term()).collect();
                Term::list(terms)
            }
            GleamValue::Constructor { name, fields } => {
                let atom_tag = Term::atom(to_snake_case(name));
                if fields.is_empty() {
                    atom_tag
                } else {
                    let mut elements = vec![atom_tag];
                    for f in fields {
                        elements.push(f.to_term());
                    }
                    Term::tuple(elements)
                }
            }
            GleamValue::Map(m) => {
                let pairs = m
                    .iter()
                    .map(|(k, v)| (Term::atom(k.clone()), v.to_term()))
                    .collect();
                Term::map(pairs)
            }
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            GleamValue::Int(i) => serde_json::json!(i),
            GleamValue::Float(f) => serde_json::json!(f),
            GleamValue::String(s) => serde_json::json!(s),
            GleamValue::Bool(b) => serde_json::json!(b),
            GleamValue::Nil => serde_json::Value::Null,
            GleamValue::Tuple(elems) => {
                serde_json::Value::Array(elems.iter().map(|e| e.to_json()).collect())
            }
            GleamValue::List(items) => {
                serde_json::Value::Array(items.iter().map(|e| e.to_json()).collect())
            }
            GleamValue::Constructor { name, fields } => {
                let mut map = serde_json::Map::new();
                map.insert("constructor".to_string(), serde_json::json!(name));
                let fields_json: Vec<serde_json::Value> = fields.iter().map(|f| f.to_json()).collect();
                map.insert("fields".to_string(), serde_json::Value::Array(fields_json));
                serde_json::Value::Object(map)
            }
            GleamValue::Map(m) => {
                let mut map = serde_json::Map::new();
                for (k, v) in m {
                    map.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(map)
            }
        }
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => GleamValue::Nil,
            serde_json::Value::Bool(b) => GleamValue::Bool(*b),
            serde_json::Value::Number(num) => {
                if let Some(i) = num.as_i64() {
                    GleamValue::Int(i)
                } else if let Some(f) = num.as_f64() {
                    GleamValue::Float(f)
                } else {
                    GleamValue::Int(0)
                }
            }
            serde_json::Value::String(s) => GleamValue::String(s.clone()),
            serde_json::Value::Array(arr) => {
                GleamValue::List(arr.iter().map(Self::from_json).collect())
            }
            serde_json::Value::Object(obj) => {
                if let Some(c_name) = obj.get("constructor").and_then(|v| v.as_str()) {
                    let fields = if let Some(fields_arr) = obj.get("fields").and_then(|v| v.as_array()) {
                        fields_arr.iter().map(Self::from_json).collect()
                    } else {
                        Vec::new()
                    };
                    GleamValue::Constructor {
                        name: c_name.to_string(),
                        fields,
                    }
                } else {
                    let mut map = HashMap::new();
                    for (k, v) in obj {
                        map.insert(k.clone(), Self::from_json(v));
                    }
                    GleamValue::Map(map)
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Message Protocol Schema Validator
// -----------------------------------------------------------------------------

pub struct MessageValidator;

impl MessageValidator {
    /// Strict type validation of an incoming ETF `Term` against a declared `GleamType`
    pub fn validate(
        term: &Term,
        expected_type: &GleamType,
        env: &TypeEnvironment,
    ) -> Result<GleamValue, RuntimeError> {
        match expected_type {
            GleamType::Int => {
                if let Some(i) = term.as_i64() {
                    Ok(GleamValue::Int(i))
                } else {
                    Err(RuntimeError::ProtocolViolation {
                        expected_type: "Int".to_string(),
                        term: term.to_string(),
                        reason: "Expected integer term".to_string(),
                    })
                }
            }
            GleamType::Float => {
                if let Some(f) = term.as_f64() {
                    Ok(GleamValue::Float(f))
                } else {
                    Err(RuntimeError::ProtocolViolation {
                        expected_type: "Float".to_string(),
                        term: term.to_string(),
                        reason: "Expected float term".to_string(),
                    })
                }
            }
            GleamType::String => {
                if let Some(s) = term.as_str() {
                    Ok(GleamValue::String(s.to_string()))
                } else {
                    Err(RuntimeError::ProtocolViolation {
                        expected_type: "String".to_string(),
                        term: term.to_string(),
                        reason: "Expected string or binary term".to_string(),
                    })
                }
            }
            GleamType::Bool => match term.as_atom() {
                Some("true") => Ok(GleamValue::Bool(true)),
                Some("false") => Ok(GleamValue::Bool(false)),
                _ => Err(RuntimeError::ProtocolViolation {
                    expected_type: "Bool".to_string(),
                    term: term.to_string(),
                    reason: "Expected boolean atom (:true or :false)".to_string(),
                }),
            },
            GleamType::Nil => match term {
                Term::Nil => Ok(GleamValue::Nil),
                Term::Atom(s) if s == "nil" => Ok(GleamValue::Nil),
                _ => Err(RuntimeError::ProtocolViolation {
                    expected_type: "Nil".to_string(),
                    term: term.to_string(),
                    reason: "Expected nil term".to_string(),
                }),
            },
            GleamType::Tuple(element_types) => match term {
                Term::Tuple(elements) => {
                    if elements.len() != element_types.len() {
                        return Err(RuntimeError::ProtocolViolation {
                            expected_type: format!("{:?}", expected_type),
                            term: term.to_string(),
                            reason: format!(
                                "Tuple arity mismatch: expected {}, got {}",
                                element_types.len(),
                                elements.len()
                            ),
                        });
                    }
                    let mut vals = Vec::new();
                    for (t, elem_type) in elements.iter().zip(element_types.iter()) {
                        vals.push(Self::validate(t, elem_type, env)?);
                    }
                    Ok(GleamValue::Tuple(vals))
                }
                _ => Err(RuntimeError::ProtocolViolation {
                    expected_type: "Tuple".to_string(),
                    term: term.to_string(),
                    reason: "Expected tuple term".to_string(),
                }),
            },
            GleamType::List(inner_type) => match term {
                Term::List(items) => {
                    let mut vals = Vec::new();
                    for item in items {
                        vals.push(Self::validate(item, inner_type, env)?);
                    }
                    Ok(GleamValue::List(vals))
                }
                Term::Nil => Ok(GleamValue::List(Vec::new())),
                _ => Err(RuntimeError::ProtocolViolation {
                    expected_type: "List".to_string(),
                    term: term.to_string(),
                    reason: "Expected list term".to_string(),
                }),
            },
            GleamType::Custom { name, .. } => {
                let type_def = env
                    .types
                    .get(name)
                    .ok_or_else(|| RuntimeError::TypeError(TypeError::UnboundType(name.clone())))?;

                // Determine constructor name and fields from ETF Term
                let (constr_name, field_terms) = match term {
                    Term::Atom(s) => (s.as_str(), &[][..]),
                    Term::Tuple(elements) if !elements.is_empty() => {
                        let tag = elements[0].as_atom().ok_or_else(|| {
                            RuntimeError::ProtocolViolation {
                                expected_type: name.clone(),
                                term: term.to_string(),
                                reason: "First element of constructor tuple must be an atom".to_string(),
                            }
                        })?;
                        (tag, &elements[1..])
                    }
                    _ => {
                        return Err(RuntimeError::ProtocolViolation {
                            expected_type: name.clone(),
                            term: term.to_string(),
                            reason: "Expected atom or tuple tagged constructor".to_string(),
                        });
                    }
                };

                // Match against custom type variants (case-insensitive & snake_case)
                let matched_constructor = type_def
                    .constructors
                    .iter()
                    .find(|c| {
                        c.name.eq_ignore_ascii_case(constr_name)
                            || to_snake_case(&c.name).eq_ignore_ascii_case(constr_name)
                    })
                    .ok_or_else(|| RuntimeError::ProtocolViolation {
                        expected_type: name.clone(),
                        term: term.to_string(),
                        reason: format!(
                            "Constructor '{}' is not a valid variant of '{}'",
                            constr_name, name
                        ),
                    })?;

                if matched_constructor.fields.len() != field_terms.len() {
                    return Err(RuntimeError::ProtocolViolation {
                        expected_type: name.clone(),
                        term: term.to_string(),
                        reason: format!(
                            "Constructor '{}' arity mismatch: expected {}, got {}",
                            matched_constructor.name,
                            matched_constructor.fields.len(),
                            field_terms.len()
                        ),
                    });
                }

                let mut validated_fields = Vec::new();
                for (t, field_spec) in field_terms.iter().zip(matched_constructor.fields.iter()) {
                    validated_fields.push(Self::validate(t, &field_spec.type_, env)?);
                }

                Ok(GleamValue::Constructor {
                    name: matched_constructor.name.clone(),
                    fields: validated_fields,
                })
            }
            _ => Ok(GleamValue::Nil),
        }
    }
}

// -----------------------------------------------------------------------------
// Gleam Evaluator (Direct Execution Engine)
// -----------------------------------------------------------------------------

pub struct GleamEvaluator;

impl GleamEvaluator {
    pub fn eval_expression(
        expr: &Expression,
        vars: &mut HashMap<String, GleamValue>,
        module: &GleamModule,
    ) -> Result<GleamValue, RuntimeError> {
        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Int(i) => GleamValue::Int(*i),
                Literal::Float(f) => GleamValue::Float(*f),
                Literal::String(s) => GleamValue::String(s.clone()),
                Literal::Bool(b) => GleamValue::Bool(*b),
                Literal::Nil => GleamValue::Nil,
            }),
            Expression::Variable(name) => {
                vars.get(name).cloned().ok_or_else(|| {
                    RuntimeError::EvaluationError(format!("Unbound variable '{}'", name))
                })
            }
            Expression::BinOp { op, left, right } => {
                let l = Self::eval_expression(left, vars, module)?;
                let r = Self::eval_expression(right, vars, module)?;
                Self::eval_binop(*op, l, r)
            }
            Expression::RecordCreation { constructor, fields, .. } => {
                let mut field_vals = Vec::new();
                for (_, val_expr) in fields {
                    field_vals.push(Self::eval_expression(val_expr, vars, module)?);
                }
                Ok(GleamValue::Constructor {
                    name: constructor.clone(),
                    fields: field_vals,
                })
            }
            Expression::FieldAccess { record, field } => {
                let rec_val = Self::eval_expression(record, vars, module)?;
                match rec_val {
                    GleamValue::Constructor { name, fields } => {
                        let type_def = module
                            .types
                            .iter()
                            .find(|t| t.constructors.iter().any(|c| c.name == name))
                            .ok_or_else(|| {
                                RuntimeError::EvaluationError(format!("Type for constructor '{}' not found", name))
                            })?;

                        let constructor = type_def.find_constructor(&name).unwrap();
                        for (idx, f_def) in constructor.fields.iter().enumerate() {
                            if f_def.label.as_deref() == Some(field.as_str()) {
                                return Ok(fields[idx].clone());
                            }
                        }
                        Err(RuntimeError::EvaluationError(format!("Field '{}' not found", field)))
                    }
                    GleamValue::Map(m) => m
                        .get(field)
                        .cloned()
                        .ok_or_else(|| RuntimeError::EvaluationError(format!("Field '{}' not found", field))),
                    _ => Err(RuntimeError::EvaluationError(format!(
                        "Cannot access field '{}' on non-record value: {:?}",
                        field, rec_val
                    ))),
                }
            }
            Expression::TupleAccess { tuple, index } => {
                let tup_val = Self::eval_expression(tuple, vars, module)?;
                match tup_val {
                    GleamValue::Tuple(elems) => elems
                        .get(*index)
                        .cloned()
                        .ok_or_else(|| RuntimeError::EvaluationError(format!("Tuple index out of bounds: {}", index))),
                    _ => Err(RuntimeError::EvaluationError("Cannot index non-tuple".to_string())),
                }
            }
            Expression::Case { subject, clauses } => {
                let subject_val = Self::eval_expression(subject, vars, module)?;
                for clause in clauses {
                    let mut clause_vars = vars.clone();
                    if Self::match_pattern(&clause.pattern, &subject_val, &mut clause_vars) {
                        if let Some(ref guard) = clause.guard {
                            let guard_val = Self::eval_expression(guard, &mut clause_vars, module)?;
                            if guard_val != GleamValue::Bool(true) {
                                continue;
                            }
                        }
                        *vars = clause_vars;
                        return Self::eval_expression(&clause.body, vars, module);
                    }
                }
                Err(RuntimeError::EvaluationError(format!(
                    "Unhandled case value: {:?}",
                    subject_val
                )))
            }
            Expression::Block(stmts) => {
                let mut last = GleamValue::Nil;
                for stmt in stmts {
                    match stmt {
                        Statement::Let { pattern, value, .. } => {
                            let val = Self::eval_expression(value, vars, module)?;
                            if !Self::match_pattern(pattern, &val, vars) {
                                return Err(RuntimeError::EvaluationError(format!(
                                    "Let pattern match failed for value {:?}",
                                    val
                                )));
                            }
                        }
                        Statement::Expression(e) => {
                            last = Self::eval_expression(e, vars, module)?;
                        }
                        Statement::Use { .. } => {}
                    }
                }
                Ok(last)
            }
            Expression::FunctionCall { function, arguments } => {
                // Built-in handlers
                if let Expression::Variable(ref fn_name) = function.as_ref() {
                    if fn_name == "actor.continue" && arguments.len() == 1 {
                        let state = Self::eval_expression(&arguments[0].value, vars, module)?;
                        return Ok(GleamValue::Constructor {
                            name: "Continue".to_string(),
                            fields: vec![state],
                        });
                    }
                    if fn_name == "actor.stop" && arguments.len() == 1 {
                        let reason = Self::eval_expression(&arguments[0].value, vars, module)?;
                        return Ok(GleamValue::Constructor {
                            name: "Stop".to_string(),
                            fields: vec![reason],
                        });
                    }

                    // Check if it's a module function
                    if let Some(func) = module.find_function(fn_name) {
                        let mut call_vars = HashMap::new();
                        for (param, arg) in func.parameters.iter().zip(arguments.iter()) {
                            let val = Self::eval_expression(&arg.value, vars, module)?;
                            call_vars.insert(param.name.clone(), val);
                        }
                        let mut ret = GleamValue::Nil;
                        for stmt in &func.body {
                            match stmt {
                                Statement::Let { pattern, value, .. } => {
                                    let val = Self::eval_expression(value, &mut call_vars, module)?;
                                    Self::match_pattern(pattern, &val, &mut call_vars);
                                }
                                Statement::Expression(e) => {
                                    ret = Self::eval_expression(e, &mut call_vars, module)?;
                                }
                                Statement::Use { .. } => {}
                            }
                        }
                        return Ok(ret);
                    }
                }
                Err(RuntimeError::EvaluationError(format!("Call to unsupported function: {:?}", function)))
            }
            Expression::Tuple(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(Self::eval_expression(e, vars, module)?);
                }
                Ok(GleamValue::Tuple(vals))
            }
            Expression::List(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(Self::eval_expression(e, vars, module)?);
                }
                Ok(GleamValue::List(vals))
            }
            Expression::Panic(msg) => {
                let reason = msg.clone().unwrap_or_else(|| "Gleam panic".to_string());
                Err(RuntimeError::ActorCrashed(reason))
            }
            Expression::Todo(msg) => {
                let reason = msg.clone().unwrap_or_else(|| "Gleam todo".to_string());
                Err(RuntimeError::ActorCrashed(reason))
            }
            _ => Err(RuntimeError::EvaluationError(format!("Unsupported expression: {:?}", expr))),
        }
    }

    fn eval_binop(op: BinOp, left: GleamValue, right: GleamValue) -> Result<GleamValue, RuntimeError> {
        match (op, left, right) {
            (BinOp::Add, GleamValue::Int(a), GleamValue::Int(b)) => Ok(GleamValue::Int(a + b)),
            (BinOp::Sub, GleamValue::Int(a), GleamValue::Int(b)) => Ok(GleamValue::Int(a - b)),
            (BinOp::Mul, GleamValue::Int(a), GleamValue::Int(b)) => Ok(GleamValue::Int(a * b)),
            (BinOp::Div, GleamValue::Int(a), GleamValue::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::EvaluationError("Division by zero".to_string()))
                } else {
                    Ok(GleamValue::Int(a / b))
                }
            }
            (BinOp::Mod, GleamValue::Int(a), GleamValue::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::EvaluationError("Modulo by zero".to_string()))
                } else {
                    Ok(GleamValue::Int(a % b))
                }
            }
            (BinOp::AddFloat, GleamValue::Float(a), GleamValue::Float(b)) => Ok(GleamValue::Float(a + b)),
            (BinOp::SubFloat, GleamValue::Float(a), GleamValue::Float(b)) => Ok(GleamValue::Float(a - b)),
            (BinOp::MulFloat, GleamValue::Float(a), GleamValue::Float(b)) => Ok(GleamValue::Float(a * b)),
            (BinOp::DivFloat, GleamValue::Float(a), GleamValue::Float(b)) => Ok(GleamValue::Float(a / b)),
            (BinOp::Concat, GleamValue::String(a), GleamValue::String(b)) => Ok(GleamValue::String(format!("{}{}", a, b))),
            (BinOp::Eq, a, b) => Ok(GleamValue::Bool(a == b)),
            (BinOp::NotEq, a, b) => Ok(GleamValue::Bool(a != b)),
            (BinOp::Lt, GleamValue::Int(a), GleamValue::Int(b)) => Ok(GleamValue::Bool(a < b)),
            (BinOp::LtEq, GleamValue::Int(a), GleamValue::Int(b)) => Ok(GleamValue::Bool(a <= b)),
            (BinOp::Gt, GleamValue::Int(a), GleamValue::Int(b)) => Ok(GleamValue::Bool(a > b)),
            (BinOp::GtEq, GleamValue::Int(a), GleamValue::Int(b)) => Ok(GleamValue::Bool(a >= b)),
            (BinOp::And, GleamValue::Bool(a), GleamValue::Bool(b)) => Ok(GleamValue::Bool(a && b)),
            (BinOp::Or, GleamValue::Bool(a), GleamValue::Bool(b)) => Ok(GleamValue::Bool(a || b)),
            (other_op, l, r) => Err(RuntimeError::EvaluationError(format!(
                "Invalid binary operation {:?} on {:?} and {:?}",
                other_op, l, r
            ))),
        }
    }

    fn match_pattern(pattern: &Pattern, value: &GleamValue, bindings: &mut HashMap<String, GleamValue>) -> bool {
        match (pattern, value) {
            (Pattern::Discard(_), _) => true,
            (Pattern::Variable(name), val) => {
                bindings.insert(name.clone(), val.clone());
                true
            }
            (Pattern::Literal(Literal::Int(i)), GleamValue::Int(val)) => i == val,
            (Pattern::Literal(Literal::Float(f)), GleamValue::Float(val)) => (f - val).abs() < 1e-9,
            (Pattern::Literal(Literal::String(s)), GleamValue::String(val)) => s == val,
            (Pattern::Literal(Literal::Bool(b)), GleamValue::Bool(val)) => b == val,
            (Pattern::Literal(Literal::Nil), GleamValue::Nil) => true,
            (Pattern::Tuple(patterns), GleamValue::Tuple(values)) => {
                if patterns.len() != values.len() {
                    return false;
                }
                for (p, v) in patterns.iter().zip(values.iter()) {
                    if !Self::match_pattern(p, v, bindings) {
                        return false;
                    }
                }
                true
            }
            (Pattern::Constructor { name: pat_name, args, .. }, GleamValue::Constructor { name: val_name, fields }) => {
                if !pat_name.eq_ignore_ascii_case(val_name)
                    && !to_snake_case(pat_name).eq_ignore_ascii_case(val_name)
                {
                    return false;
                }
                if args.len() != fields.len() {
                    return false;
                }
                for (arg, field_val) in args.iter().zip(fields.iter()) {
                    if !Self::match_pattern(&arg.pattern, field_val, bindings) {
                        return false;
                    }
                }
                true
            }
            (Pattern::Or(p1, p2), val) => {
                Self::match_pattern(p1, val, bindings) || Self::match_pattern(p2, val, bindings)
            }
            _ => false,
        }
    }
}

// -----------------------------------------------------------------------------
// Gleam Sovereign Actor Process (Implements OTP GenServer)
// -----------------------------------------------------------------------------

pub struct GleamActor {
    pub name: String,
    pub module: GleamModule,
    pub type_env: TypeEnvironment,
    pub state: GleamValue,
    pub msg_type_name: String,
    pub handled_count: usize,
}

impl GleamActor {
    /// Instantiate a type-safe sovereign Gleam actor from source code
    pub fn from_source(
        name: impl Into<String>,
        source: &str,
        initial_state: GleamValue,
    ) -> Result<Self, RuntimeError> {
        let actor_name = name.into();
        let module = parse_gleam_source(source, &actor_name)
            .map_err(|e| RuntimeError::ParseError(e.to_string()))?;

        let mut type_env = TypeEnvironment::new();
        type_env
            .check_module(&module)
            .map_err(RuntimeError::TypeError)?;

        let msg_type_name = type_env
            .validate_actor_protocol(&module)
            .map_err(RuntimeError::TypeError)?;

        Ok(Self {
            name: actor_name,
            module,
            type_env,
            state: initial_state,
            msg_type_name,
            handled_count: 0,
        })
    }

    /// Convert into a supervised ChildSpec for OTP supervision trees
    pub fn child_spec<F>(id: impl Into<String>, factory: F) -> ChildSpec
    where
        F: Fn() -> GleamActor + Send + Sync + 'static,
    {
        ChildSpec::new(id, factory)
    }
}

#[async_trait]
impl GenServer for GleamActor {
    async fn init(&mut self) -> Result<(), ActorError> {
        // If an `init` function is defined in Gleam, run it
        if let Some(init_fn) = self.module.find_function("init") {
            let mut vars = HashMap::new();
            if let Ok(init_val) = GleamEvaluator::eval_expression(
                &Expression::FunctionCall {
                    function: Box::new(Expression::Variable(init_fn.name.clone())),
                    arguments: Vec::new(),
                },
                &mut vars,
                &self.module,
            ) {
                self.state = init_val;
            }
        }
        Ok(())
    }

    async fn handle_call(&mut self, req: Term) -> Result<Term, ActorError> {
        // Step 1: Enforce strict schema validation on incoming message!
        let msg_type = self
            .type_env
            .types
            .get(&self.msg_type_name)
            .ok_or_else(|| ActorError::Custom(format!("Type '{}' not registered", self.msg_type_name)))?;

        let expected_gleam_type = GleamType::custom(&msg_type.name, Vec::new());
        let validated_msg = match MessageValidator::validate(&req, &expected_gleam_type, &self.type_env) {
            Ok(val) => val,
            Err(e) => {
                // Reject invalid messages before reaching actor logic
                return Err(ActorError::Custom(format!("Type protocol error: {}", e)));
            }
        };

        // Check for intentional crash/panic trigger
        if let GleamValue::Constructor { ref name, .. } = validated_msg {
            if name == "Crash" || name == "Panic" {
                return Err(ActorError::ProcessCrashed(format!(
                    "Gleam actor '{}' intentionally crashed on message: {:?}",
                    self.name, validated_msg
                )));
            }
        }

        // Step 2: Execute Gleam `handle_msg(msg, state)`
        let mut vars = HashMap::new();
        vars.insert("msg".to_string(), validated_msg);
        vars.insert("state".to_string(), self.state.clone());

        let call_expr = Expression::FunctionCall {
            function: Box::new(Expression::Variable("handle_msg".to_string())),
            arguments: vec![
                CallArgument::positional(Expression::Variable("msg".to_string())),
                CallArgument::positional(Expression::Variable("state".to_string())),
            ],
        };

        let result = GleamEvaluator::eval_expression(&call_expr, &mut vars, &self.module)
            .map_err(|e| ActorError::Custom(e.to_string()))?;

        self.handled_count += 1;

        // Step 3: Handle state updates and return reply
        match result {
            GleamValue::Constructor { ref name, ref fields } if name == "Continue" => {
                if let Some(new_state) = fields.first() {
                    self.state = new_state.clone();
                }
                Ok(self.state.to_term())
            }
            GleamValue::Tuple(ref elems) if elems.len() == 2 => {
                // Return #(reply, new_state)
                self.state = elems[1].clone();
                Ok(elems[0].to_term())
            }
            other => {
                self.state = other.clone();
                Ok(other.to_term())
            }
        }
    }

    async fn handle_cast(&mut self, msg: Term) -> Result<(), ActorError> {
        let _ = self.handle_call(msg).await?;
        Ok(())
    }
}
