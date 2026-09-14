//! Type checker, Hindley-Milner inference, and exhaustive pattern verification for Gleam.
//!
//! Validates type unification, detects mismatches, enforces exhaustive pattern matching
//! on algebraic data types, and validates actor messaging protocols.

use std::collections::HashMap;
use crate::gleam::ast::*;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, got {got}")]
    Mismatch { expected: String, got: String },

    #[error("Unbound variable '{0}'")]
    UnboundVariable(String),

    #[error("Unbound type '{0}'")]
    UnboundType(String),

    #[error("Unknown constructor '{0}'")]
    UnknownConstructor(String),

    #[error("Non-exhaustive pattern match: missing constructor variant(s): {missing:?}")]
    NonExhaustivePatternMatch { missing: Vec<String> },

    #[error("Actor message protocol violation: expected '{expected_protocol}', got '{actual}'")]
    ProtocolViolation {
        expected_protocol: String,
        actual: String,
    },

    #[error("Arity mismatch for '{name}': expected {expected} argument(s), got {got}")]
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },

    #[error("Field '{field}' not found on type '{type_name}'")]
    FieldNotFound {
        type_name: String,
        field: String,
    },

    #[error("Cannot unify generic type parameter '{0}'")]
    CannotUnify(String),
}

/// Type environment holding custom types, constructors, and scoped variables
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Registered Custom Types by name
    pub types: HashMap<String, TypeDefinition>,
    /// Constructor mapping: Constructor Name -> (Type Name, Constructor)
    pub constructors: HashMap<String, (String, Constructor)>,
    /// Function signatures: Function Name -> (Parameter Types, Return Type)
    pub functions: HashMap<String, (Vec<GleamType>, GleamType)>,
    /// Scoped variable bindings stack
    pub scopes: Vec<HashMap<String, GleamType>>,
    /// Generic type variable bindings
    pub substitutions: HashMap<String, GleamType>,
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnvironment {
    pub fn new() -> Self {
        let mut env = Self {
            types: HashMap::new(),
            constructors: HashMap::new(),
            functions: HashMap::new(),
            scopes: vec![HashMap::new()],
            substitutions: HashMap::new(),
        };

        env.register_prelude();
        env
    }

    fn register_prelude(&mut self) {
        // Built-in Result type
        let ok_field = ConstructorField::positional(GleamType::Generic("a".to_string()));
        let error_field = ConstructorField::positional(GleamType::Generic("e".to_string()));
        let result_type = TypeDefinition::with_params(
            "Result",
            vec!["a".to_string(), "e".to_string()],
            vec![
                Constructor::new("Ok", vec![ok_field]),
                Constructor::new("Error", vec![error_field]),
            ],
        );
        self.register_type(result_type);

        // Built-in actor & process functions
        // process.send: fn(Subject(msg), msg) -> Nil
        self.functions.insert(
            "process.send".to_string(),
            (
                vec![
                    GleamType::custom_with_module("process", "Subject", vec![GleamType::Generic("msg".to_string())]),
                    GleamType::Generic("msg".to_string()),
                ],
                GleamType::Nil,
            ),
        );

        // actor.continue: fn(state) -> Next(msg, state)
        self.functions.insert(
            "actor.continue".to_string(),
            (
                vec![GleamType::Generic("state".to_string())],
                GleamType::custom_with_module(
                    "actor",
                    "Next",
                    vec![
                        GleamType::Generic("msg".to_string()),
                        GleamType::Generic("state".to_string()),
                    ],
                ),
            ),
        );

        // actor.stop: fn(reason) -> Next(msg, state)
        self.functions.insert(
            "actor.stop".to_string(),
            (
                vec![GleamType::Generic("reason".to_string())],
                GleamType::custom_with_module(
                    "actor",
                    "Next",
                    vec![
                        GleamType::Generic("msg".to_string()),
                        GleamType::Generic("state".to_string()),
                    ],
                ),
            ),
        );

        // String operations
        self.functions.insert(
            "string.length".to_string(),
            (vec![GleamType::String], GleamType::Int),
        );
        self.functions.insert(
            "int.to_string".to_string(),
            (vec![GleamType::Int], GleamType::String),
        );
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn bind_variable(&mut self, name: impl Into<String>, type_: GleamType) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.into(), type_);
        }
    }

    pub fn lookup_variable(&self, name: &str) -> Option<GleamType> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(self.apply_substitutions(t));
            }
        }
        None
    }

    pub fn register_type(&mut self, type_def: TypeDefinition) {
        let type_name = type_def.name.clone();
        for constructor in &type_def.constructors {
            self.constructors.insert(
                constructor.name.clone(),
                (type_name.clone(), constructor.clone()),
            );
        }
        self.types.insert(type_name, type_def);
    }

    pub fn register_function(
        &mut self,
        name: impl Into<String>,
        params: Vec<GleamType>,
        return_type: GleamType,
    ) {
        self.functions.insert(name.into(), (params, return_type));
    }

    // -------------------------------------------------------------------------
    // Unification Engine
    // -------------------------------------------------------------------------

    pub fn apply_substitutions(&self, type_: &GleamType) -> GleamType {
        match type_ {
            GleamType::Generic(name) => {
                if let Some(sub) = self.substitutions.get(name) {
                    self.apply_substitutions(sub)
                } else {
                    type_.clone()
                }
            }
            GleamType::List(inner) => {
                GleamType::List(Box::new(self.apply_substitutions(inner)))
            }
            GleamType::Tuple(elems) => {
                GleamType::Tuple(elems.iter().map(|e| self.apply_substitutions(e)).collect())
            }
            GleamType::Custom { name, module, params } => {
                GleamType::Custom {
                    name: name.clone(),
                    module: module.clone(),
                    params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                }
            }
            GleamType::Function { params, return_type } => {
                GleamType::Function {
                    params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                    return_type: Box::new(self.apply_substitutions(return_type)),
                }
            }
            other => other.clone(),
        }
    }

    pub fn unify(&mut self, expected: &GleamType, actual: &GleamType) -> Result<GleamType, TypeError> {
        let t1 = self.apply_substitutions(expected);
        let t2 = self.apply_substitutions(actual);

        if t1 == t2 {
            return Ok(t1);
        }

        // Generic type variable unification
        if let GleamType::Generic(name) = &t1 {
            self.substitutions.insert(name.clone(), t2.clone());
            return Ok(t2);
        }
        if let GleamType::Generic(name) = &t2 {
            self.substitutions.insert(name.clone(), t1.clone());
            return Ok(t1);
        }

        match (&t1, &t2) {
            (GleamType::List(e1), GleamType::List(e2)) => {
                let unified = self.unify(e1, e2)?;
                Ok(GleamType::List(Box::new(unified)))
            }
            (GleamType::Tuple(v1), GleamType::Tuple(v2)) => {
                if v1.len() != v2.len() {
                    return Err(TypeError::Mismatch {
                        expected: t1.to_string(),
                        got: t2.to_string(),
                    });
                }
                let mut unified = Vec::new();
                for (a, b) in v1.iter().zip(v2.iter()) {
                    unified.push(self.unify(a, b)?);
                }
                Ok(GleamType::Tuple(unified))
            }
            (
                GleamType::Custom { name: n1, module: m1, params: p1 },
                GleamType::Custom { name: n2, module: m2, params: p2 },
            ) => {
                let matches_module = m1.is_some() && m1 == m2;
                if (n1 == n2 || matches_module) && p1.len() == p2.len() {
                    let mut unified_params = Vec::new();
                    for (a, b) in p1.iter().zip(p2.iter()) {
                        unified_params.push(self.unify(a, b)?);
                    }
                    return Ok(GleamType::Custom {
                        name: n1.clone(),
                        module: m1.clone(),
                        params: unified_params,
                    });
                }
                return Err(TypeError::Mismatch {
                    expected: t1.to_string(),
                    got: t2.to_string(),
                });
            }
            (
                GleamType::Function { params: p1, return_type: r1 },
                GleamType::Function { params: p2, return_type: r2 },
            ) => {
                if p1.len() != p2.len() {
                    return Err(TypeError::Mismatch {
                        expected: t1.to_string(),
                        got: t2.to_string(),
                    });
                }
                let mut unified_params = Vec::new();
                for (a, b) in p1.iter().zip(p2.iter()) {
                    unified_params.push(self.unify(a, b)?);
                }
                let unified_ret = self.unify(r1, r2)?;
                Ok(GleamType::Function {
                    params: unified_params,
                    return_type: Box::new(unified_ret),
                })
            }
            _ => Err(TypeError::Mismatch {
                expected: t1.to_string(),
                got: t2.to_string(),
            }),
        }
    }

    // -------------------------------------------------------------------------
    // Exhaustive Pattern Matching Verification
    // -------------------------------------------------------------------------

    pub fn check_pattern_exhaustiveness(
        &self,
        subject_type: &GleamType,
        clauses: &[CaseClause],
    ) -> Result<(), TypeError> {
        let concrete_type = self.apply_substitutions(subject_type);

        // If any clause is an unguarded wildcard / discard or variable, matching is exhaustive
        for clause in clauses {
            if clause.guard.is_none() && clause.pattern.is_wildcard() {
                return Ok(());
            }
        }

        match concrete_type {
            GleamType::Bool => {
                let mut has_true = false;
                let mut has_false = false;
                for clause in clauses {
                    if clause.guard.is_none() {
                        match &clause.pattern {
                            Pattern::Literal(Literal::Bool(true)) => has_true = true,
                            Pattern::Literal(Literal::Bool(false)) => has_false = true,
                            _ => {}
                        }
                    }
                }
                let mut missing = Vec::new();
                if !has_true {
                    missing.push("True".to_string());
                }
                if !has_false {
                    missing.push("False".to_string());
                }
                if missing.is_empty() {
                    Ok(())
                } else {
                    Err(TypeError::NonExhaustivePatternMatch { missing })
                }
            }
            GleamType::Custom { ref name, .. } => {
                let type_def = self.types.get(name).ok_or_else(|| TypeError::UnboundType(name.clone()))?;

                let all_constructors: Vec<String> = type_def
                    .constructors
                    .iter()
                    .map(|c| c.name.clone())
                    .collect();

                let mut matched_constructors = Vec::new();

                for clause in clauses {
                    if clause.guard.is_none() {
                        self.collect_pattern_constructors(&clause.pattern, &mut matched_constructors);
                    }
                }

                let mut missing = Vec::new();
                for expected in &all_constructors {
                    if !matched_constructors.contains(expected) {
                        missing.push(expected.clone());
                    }
                }

                if missing.is_empty() {
                    Ok(())
                } else {
                    Err(TypeError::NonExhaustivePatternMatch { missing })
                }
            }
            GleamType::Tuple(ref elem_types) => {
                // If any clause has a wildcard on all tuple positions, it's exhaustive
                for clause in clauses {
                    if clause.guard.is_none() {
                        match &clause.pattern {
                            Pattern::Tuple(elements) => {
                                if elements.iter().all(|p| p.is_wildcard()) {
                                    return Ok(());
                                }
                            }
                            pat if pat.is_wildcard() => return Ok(()),
                            _ => {}
                        }
                    }
                }

                // If tuple of ADTs (e.g. 2 elements):
                if elem_types.len() == 2 {
                    if let (GleamType::Custom { name: n1, .. }, GleamType::Custom { name: n2, .. }) = (&elem_types[0], &elem_types[1]) {
                        if let (Some(t1), Some(t2)) = (self.types.get(n1), self.types.get(n2)) {
                            let mut missing = Vec::new();
                            for c1 in &t1.constructors {
                                for c2 in &t2.constructors {
                                    let covered = clauses.iter().any(|clause| {
                                        if clause.guard.is_some() {
                                            return false;
                                        }
                                        match &clause.pattern {
                                            Pattern::Tuple(elements) if elements.len() == 2 => {
                                                let m1 = match &elements[0] {
                                                    Pattern::Constructor { name, .. } => name == &c1.name || crate::gleam::codegen::to_snake_case(name) == crate::gleam::codegen::to_snake_case(&c1.name),
                                                    pat if pat.is_wildcard() => true,
                                                    _ => false,
                                                };
                                                let m2 = match &elements[1] {
                                                    Pattern::Constructor { name, .. } => name == &c2.name || crate::gleam::codegen::to_snake_case(name) == crate::gleam::codegen::to_snake_case(&c2.name),
                                                    pat if pat.is_wildcard() => true,
                                                    _ => false,
                                                };
                                                m1 && m2
                                            }
                                            pat if pat.is_wildcard() => true,
                                            _ => false,
                                        }
                                    });

                                    if !covered {
                                        missing.push(format!("#({}, {})", c1.name, c2.name));
                                    }
                                }
                            }

                            if missing.is_empty() {
                                return Ok(());
                            } else {
                                return Err(TypeError::NonExhaustivePatternMatch { missing });
                            }
                        }
                    }
                }

                // General tuple check
                let has_catchall = clauses.iter().any(|c| c.guard.is_none() && match &c.pattern {
                    Pattern::Tuple(elements) => elements.iter().all(|p| p.is_wildcard()),
                    pat => pat.is_wildcard(),
                });
                if has_catchall {
                    Ok(())
                } else {
                    Err(TypeError::NonExhaustivePatternMatch {
                        missing: vec!["_ (wildcard catch-all)".to_string()],
                    })
                }
            }
            _ => {
                // For other types (Int, String, List), without wildcard it's not exhaustive
                let has_catchall = clauses.iter().any(|c| c.guard.is_none() && c.pattern.is_wildcard());
                if has_catchall {
                    Ok(())
                } else {
                    Err(TypeError::NonExhaustivePatternMatch {
                        missing: vec!["_ (wildcard catch-all)".to_string()],
                    })
                }
            }
        }
    }

    fn collect_pattern_constructors(&self, pattern: &Pattern, out: &mut Vec<String>) {
        match pattern {
            Pattern::Constructor { name, .. } => {
                out.push(name.clone());
            }
            Pattern::Or(p1, p2) => {
                self.collect_pattern_constructors(p1, out);
                self.collect_pattern_constructors(p2, out);
            }
            _ => {}
        }
    }

    // -------------------------------------------------------------------------
    // Expression & Statement Type Inference
    // -------------------------------------------------------------------------

    pub fn infer_expression(&mut self, expr: &Expression) -> Result<GleamType, TypeError> {
        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Int(_) => GleamType::Int,
                Literal::Float(_) => GleamType::Float,
                Literal::String(_) => GleamType::String,
                Literal::Bool(_) => GleamType::Bool,
                Literal::Nil => GleamType::Nil,
            }),
            Expression::Variable(name) => {
                if let Some(t) = self.lookup_variable(name) {
                    Ok(t)
                } else if let Some((_, ret)) = self.functions.get(name) {
                    Ok(ret.clone())
                } else {
                    Err(TypeError::UnboundVariable(name.clone()))
                }
            }
            Expression::BinOp { op, left, right } => {
                let left_type = self.infer_expression(left)?;
                let right_type = self.infer_expression(right)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        self.unify(&GleamType::Int, &left_type)?;
                        self.unify(&GleamType::Int, &right_type)?;
                        Ok(GleamType::Int)
                    }
                    BinOp::AddFloat | BinOp::SubFloat | BinOp::MulFloat | BinOp::DivFloat => {
                        self.unify(&GleamType::Float, &left_type)?;
                        self.unify(&GleamType::Float, &right_type)?;
                        Ok(GleamType::Float)
                    }
                    BinOp::Concat => {
                        self.unify(&GleamType::String, &left_type)?;
                        self.unify(&GleamType::String, &right_type)?;
                        Ok(GleamType::String)
                    }
                    BinOp::Eq | BinOp::NotEq => {
                        self.unify(&left_type, &right_type)?;
                        Ok(GleamType::Bool)
                    }
                    BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                        self.unify(&left_type, &right_type)?;
                        Ok(GleamType::Bool)
                    }
                    BinOp::And | BinOp::Or => {
                        self.unify(&GleamType::Bool, &left_type)?;
                        self.unify(&GleamType::Bool, &right_type)?;
                        Ok(GleamType::Bool)
                    }
                    BinOp::Pipe => {
                        // In Gleam, `x |> f(y)` desugars to `f(x, y)`
                        // Here if right is a function call, insert left as the first argument
                        match right.as_ref() {
                            Expression::FunctionCall { function, arguments } => {
                                let mut new_args = vec![CallArgument::positional(*left.clone())];
                                new_args.extend(arguments.clone());
                                let call_expr = Expression::FunctionCall {
                                    function: function.clone(),
                                    arguments: new_args,
                                };
                                self.infer_expression(&call_expr)
                            }
                            Expression::Variable(func_name) => {
                                let call_expr = Expression::FunctionCall {
                                    function: Box::new(Expression::Variable(func_name.clone())),
                                    arguments: vec![CallArgument::positional(*left.clone())],
                                };
                                self.infer_expression(&call_expr)
                            }
                            _ => Err(TypeError::Mismatch {
                                expected: "function call on right side of pipe".to_string(),
                                got: format!("{:?}", right),
                            }),
                        }
                    }
                }
            }
            Expression::RecordCreation { constructor, fields, .. } => {
                let (type_name, constr) = self
                    .constructors
                    .get(constructor)
                    .ok_or_else(|| TypeError::UnknownConstructor(constructor.clone()))?
                    .clone();

                let type_def = self.types.get(&type_name).cloned().unwrap();

                // Validate field count / types
                if constr.fields.len() != fields.len() {
                    return Err(TypeError::ArityMismatch {
                        name: constructor.clone(),
                        expected: constr.fields.len(),
                        got: fields.len(),
                    });
                }

                for (idx, field_def) in constr.fields.iter().enumerate() {
                    let provided_val = &fields[idx].1;
                    let val_type = self.infer_expression(provided_val)?;
                    self.unify(&field_def.type_, &val_type)?;
                }

                let params = type_def
                    .parameters
                    .iter()
                    .map(|p| self.substitutions.get(p).cloned().unwrap_or(GleamType::Generic(p.clone())))
                    .collect();

                Ok(GleamType::Custom {
                    name: type_name,
                    module: None,
                    params,
                })
            }
            Expression::FieldAccess { record, field } => {
                let rec_type = self.infer_expression(record)?;
                match rec_type {
                    GleamType::Generic(_) => {
                        Ok(GleamType::Generic(format!("module_field_{}", field)))
                    }
                    GleamType::Custom { ref name, .. } => {
                        let type_def = self
                            .types
                            .get(name)
                            .ok_or_else(|| TypeError::UnboundType(name.clone()))?;

                        // Find field in constructors
                        for c in &type_def.constructors {
                            for f in &c.fields {
                                if f.label.as_deref() == Some(field.as_str()) {
                                    return Ok(f.type_.clone());
                                }
                            }
                        }

                        Err(TypeError::FieldNotFound {
                            type_name: name.clone(),
                            field: field.clone(),
                        })
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "record type with fields".to_string(),
                        got: rec_type.to_string(),
                    }),
                }
            }
            Expression::TupleAccess { tuple, index } => {
                let tuple_type = self.infer_expression(tuple)?;
                match tuple_type {
                    GleamType::Tuple(elems) => {
                        if *index < elems.len() {
                            Ok(elems[*index].clone())
                        } else {
                            Err(TypeError::ArityMismatch {
                                name: "tuple".to_string(),
                                expected: elems.len(),
                                got: *index + 1,
                            })
                        }
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "tuple".to_string(),
                        got: tuple_type.to_string(),
                    }),
                }
            }
            Expression::FunctionCall { function, arguments } => {
                let func_type = match function.as_ref() {
                    Expression::Variable(name) => {
                        if let Some((params, ret)) = self.functions.get(name).cloned() {
                            GleamType::Function {
                                params,
                                return_type: Box::new(ret),
                            }
                        } else if let Some((type_name, constr)) = self.constructors.get(name).cloned() {
                            // Constructor invoked as function
                            let params = constr.fields.iter().map(|f| f.type_.clone()).collect();
                            GleamType::Function {
                                params,
                                return_type: Box::new(GleamType::custom(type_name, Vec::new())),
                            }
                        } else {
                            self.infer_expression(function)?
                        }
                    }
                    _ => self.infer_expression(function)?,
                };

                match func_type {
                    GleamType::Function { params, return_type } => {
                        if params.len() != arguments.len() {
                            return Err(TypeError::ArityMismatch {
                                name: format!("{:?}", function),
                                expected: params.len(),
                                got: arguments.len(),
                            });
                        }

                        for (param_type, arg) in params.iter().zip(arguments.iter()) {
                            let arg_type = self.infer_expression(&arg.value)?;
                            self.unify(param_type, &arg_type)?;
                        }

                        Ok(self.apply_substitutions(&return_type))
                    }
                    GleamType::Generic(_) => {
                        for arg in arguments {
                            let _ = self.infer_expression(&arg.value)?;
                        }
                        Ok(GleamType::Generic("call_ret".to_string()))
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "function".to_string(),
                        got: func_type.to_string(),
                    }),
                }
            }
            Expression::Case { subject, clauses } => {
                let subject_type = self.infer_expression(subject)?;

                // Exhaustiveness check!
                self.check_pattern_exhaustiveness(&subject_type, clauses)?;

                if clauses.is_empty() {
                    return Ok(GleamType::Nil);
                }

                let mut result_type = None;

                for clause in clauses {
                    self.enter_scope();
                    self.bind_pattern_variables(&clause.pattern, &subject_type)?;

                    if let Some(ref guard) = clause.guard {
                        let guard_type = self.infer_expression(guard)?;
                        self.unify(&GleamType::Bool, &guard_type)?;
                    }

                    let body_type = self.infer_expression(&clause.body)?;
                    self.exit_scope();

                    if let Some(ref current_res) = result_type {
                        result_type = Some(self.unify(current_res, &body_type)?);
                    } else {
                        result_type = Some(body_type);
                    }
                }

                Ok(result_type.unwrap_or(GleamType::Nil))
            }
            Expression::Block(stmts) => {
                self.enter_scope();
                let mut last_type = GleamType::Nil;
                for stmt in stmts {
                    last_type = self.check_statement(stmt)?;
                }
                self.exit_scope();
                Ok(last_type)
            }
            Expression::Tuple(elements) => {
                let mut types = Vec::new();
                for elem in elements {
                    types.push(self.infer_expression(elem)?);
                }
                Ok(GleamType::Tuple(types))
            }
            Expression::List(elements) => {
                if elements.is_empty() {
                    Ok(GleamType::List(Box::new(GleamType::Generic("a".to_string()))))
                } else {
                    let first_type = self.infer_expression(&elements[0])?;
                    for elem in &elements[1..] {
                        let elem_type = self.infer_expression(elem)?;
                        self.unify(&first_type, &elem_type)?;
                    }
                    Ok(GleamType::List(Box::new(first_type)))
                }
            }
            Expression::AnonymousFunction { parameters, return_type, body } => {
                self.enter_scope();
                let mut param_types = Vec::new();
                for p in parameters {
                    let p_type = p.type_annotation.clone().unwrap_or(GleamType::Generic(p.name.clone()));
                    self.bind_variable(p.name.clone(), p_type.clone());
                    param_types.push(p_type);
                }

                let mut body_type = GleamType::Nil;
                for stmt in body {
                    body_type = self.check_statement(stmt)?;
                }
                self.exit_scope();

                if let Some(expected_ret) = return_type {
                    self.unify(expected_ret, &body_type)?;
                }

                Ok(GleamType::Function {
                    params: param_types,
                    return_type: Box::new(body_type),
                })
            }
            Expression::Panic(_) | Expression::Todo(_) => {
                Ok(GleamType::Generic("a".to_string()))
            }
        }
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<GleamType, TypeError> {
        match stmt {
            Statement::Let { pattern, type_annotation, value } => {
                let val_type = self.infer_expression(value)?;
                if let Some(expected) = type_annotation {
                    self.unify(expected, &val_type)?;
                }
                self.bind_pattern_variables(pattern, &val_type)?;
                Ok(GleamType::Nil)
            }
            Statement::Expression(expr) => self.infer_expression(expr),
            Statement::Use { patterns: _, call } => {
                self.infer_expression(call)
            }
        }
    }

    pub fn bind_pattern_variables(
        &mut self,
        pattern: &Pattern,
        inferred_type: &GleamType,
    ) -> Result<(), TypeError> {
        let concrete_type = self.apply_substitutions(inferred_type);

        match pattern {
            Pattern::Variable(name) => {
                self.bind_variable(name.clone(), concrete_type);
                Ok(())
            }
            Pattern::Discard(_) => Ok(()),
            Pattern::Literal(lit) => {
                let lit_type = match lit {
                    Literal::Int(_) => GleamType::Int,
                    Literal::Float(_) => GleamType::Float,
                    Literal::String(_) => GleamType::String,
                    Literal::Bool(_) => GleamType::Bool,
                    Literal::Nil => GleamType::Nil,
                };
                self.unify(&concrete_type, &lit_type)?;
                Ok(())
            }
            Pattern::Constructor { name, args, .. } => {
                let (type_name, constr) = self
                    .constructors
                    .get(name)
                    .ok_or_else(|| TypeError::UnknownConstructor(name.clone()))?
                    .clone();

                self.unify(
                    &concrete_type,
                    &GleamType::Custom {
                        name: type_name,
                        module: None,
                        params: Vec::new(),
                    },
                )?;

                for (idx, arg) in args.iter().enumerate() {
                    if idx < constr.fields.len() {
                        let field_type = &constr.fields[idx].type_;
                        self.bind_pattern_variables(&arg.pattern, field_type)?;
                    }
                }
                Ok(())
            }
            Pattern::Tuple(elements) => match concrete_type {
                GleamType::Tuple(elem_types) => {
                    for (pat, t) in elements.iter().zip(elem_types.iter()) {
                        self.bind_pattern_variables(pat, t)?;
                    }
                    Ok(())
                }
                _ => Err(TypeError::Mismatch {
                    expected: "tuple".to_string(),
                    got: concrete_type.to_string(),
                }),
            },
            Pattern::List { elements, tail } => match concrete_type {
                GleamType::List(inner) => {
                    for elem in elements {
                        self.bind_pattern_variables(elem, &inner)?;
                    }
                    if let Some(t) = tail {
                        self.bind_pattern_variables(t, &GleamType::List(inner))?;
                    }
                    Ok(())
                }
                _ => Err(TypeError::Mismatch {
                    expected: "list".to_string(),
                    got: concrete_type.to_string(),
                }),
            },
            Pattern::Or(p1, _p2) => {
                self.bind_pattern_variables(p1, &concrete_type)
            }
        }
    }

    // -------------------------------------------------------------------------
    // Module Verification
    // -------------------------------------------------------------------------

    pub fn check_module(&mut self, module: &GleamModule) -> Result<(), TypeError> {
        // Step 0: Register all imported modules
        for imp in &module.imports {
            let mod_name = imp.alias.clone().unwrap_or_else(|| {
                imp.module.split('/').last().unwrap_or(&imp.module).to_string()
            });
            self.bind_variable(mod_name, GleamType::Generic("module".to_string()));
            for unq in &imp.unqualified {
                self.bind_variable(unq.clone(), GleamType::Generic(unq.clone()));
            }
        }

        // Step 1: Register all types
        for type_def in &module.types {
            self.register_type(type_def.clone());
        }

        // Step 1.5: Register all module constants
        for c in &module.constants {
            let c_type = if let Some(ref t) = c.type_annotation {
                t.clone()
            } else {
                self.infer_expression(&c.value).unwrap_or(GleamType::Generic("const".to_string()))
            };
            self.bind_variable(c.name.clone(), c_type);
        }

        // Step 2: Register all function signatures
        for func in &module.functions {
            let params = func
                .parameters
                .iter()
                .map(|p| p.type_annotation.clone().unwrap_or(GleamType::Generic(p.name.clone())))
                .collect();
            let return_type = func
                .return_type
                .clone()
                .unwrap_or(GleamType::Generic("ret".to_string()));
            self.register_function(func.name.clone(), params, return_type);
        }

        // Step 3: Check all function bodies
        for func in &module.functions {
            self.substitutions.clear();
            self.enter_scope();
            for param in &func.parameters {
                let p_type = param
                    .type_annotation
                    .clone()
                    .unwrap_or(GleamType::Generic(param.name.clone()));
                self.bind_variable(param.name.clone(), p_type);
            }

            let mut body_type = GleamType::Nil;
            for stmt in &func.body {
                body_type = self.check_statement(stmt)?;
            }
            self.exit_scope();

            if let Some(ref declared_ret) = func.return_type {
                self.unify(declared_ret, &body_type)?;
            }
        }

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Actor Message Protocol Validation
    // -------------------------------------------------------------------------

    /// Validates that an actor's declared `Msg` type protocol is adhered to
    pub fn validate_actor_protocol(&self, module: &GleamModule) -> Result<String, TypeError> {
        // 1. Locate the `Msg` type definition
        let msg_type = module
            .find_type("Msg")
            .or_else(|| module.types.iter().find(|t| t.name.ends_with("Msg")))
            .ok_or_else(|| TypeError::UnboundType("Actor module must define 'pub type Msg'".to_string()))?;

        if msg_type.constructors.is_empty() {
            return Err(TypeError::ProtocolViolation {
                expected_protocol: msg_type.name.clone(),
                actual: "Actor Msg type has 0 variant constructors".to_string(),
            });
        }

        // 2. Locate `handle_msg` function
        let handle_fn = module.find_function("handle_msg").ok_or_else(|| {
            TypeError::UnboundVariable("Actor module must define 'handle_msg(msg, state)'".to_string())
        })?;

        if handle_fn.parameters.len() < 2 {
            return Err(TypeError::ArityMismatch {
                name: "handle_msg".to_string(),
                expected: 2,
                got: handle_fn.parameters.len(),
            });
        }

        Ok(msg_type.name.clone())
    }
}
