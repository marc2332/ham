use crate::{
    primitive_values::{
        boolean::{
            Boolean,
            BooleanValueBase,
        },
        number::{
            Number,
            NumberValueBase,
        },
        primitive_base::PrimitiveValueBase,
        string::{
            StringVal,
            StringValueBase,
        },
    },
    types::{
        IndexedTokenList,
        Token,
        TokensList,
    },
    utils::{
        Directions,
        Ops,
    },
};
use erased_serde::serialize_trait_object;
use serde::Serialize;
use std::any::Any;
use uuid::Uuid;

/* BASE */
pub trait AstBase: dyn_clone::DynClone + erased_serde::Serialize + std::fmt::Debug {
    fn get_type(&self) -> Ops;
    fn as_self(&self) -> &dyn Any;
}

dyn_clone::clone_trait_object!(AstBase);
serialize_trait_object!(AstBase);

/*
 * Break statement
 *
 * Used to break from while loops
 *
 */

#[derive(Clone, Debug, Serialize)]
pub struct Break();

// Implement base methods for REFERENCE
impl AstBase for Break {
    fn get_type(&self) -> Ops {
        Ops::Break
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait BreakBase {
    fn new() -> Self;
}

impl BreakBase for Break {
    fn new() -> Self {
        Self()
    }
}

/*
 * Reference by name
 *
 * This happens when a variable instead of having an static value,
 * it references to another variable by it's name
 *
 */

#[derive(Clone, Debug, Serialize)]
pub struct Reference(pub String);

// Implement base methods for REFERENCE
impl PrimitiveValueBase for Reference {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

// Custom methods for REFERENCE
pub trait ReferenceValueBase {
    fn new(val: String) -> Self;
}

impl ReferenceValueBase for Reference {
    fn new(val: String) -> Self {
        Self(val)
    }
}

/* MODULE STATEMENT */

#[derive(Clone, Debug, Serialize)]
pub struct Module {
    pub name: String,
    pub functions: Vec<FnDefinition>,
}

impl AstBase for Module {
    fn get_type(&self) -> Ops {
        Ops::Module
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* RETURN STATEMENT */

#[derive(Clone, Debug, Serialize)]
pub struct ReturnStatement {
    pub value: BoxedValue,
}

impl AstBase for ReturnStatement {
    fn get_type(&self) -> Ops {
        Ops::Return
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* RESULT EXPRESSION */
pub trait ResultExpressionBase {
    fn new(relation: Ops, left: BoxedValue, right: BoxedValue) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct ResultExpression {
    pub left: BoxedValue,
    pub relation: Ops,
    pub right: BoxedValue,
}

impl ResultExpressionBase for ResultExpression {
    fn new(relation: Ops, left: BoxedValue, right: BoxedValue) -> Self {
        Self {
            left,
            relation,
            right,
        }
    }
}

impl AstBase for ResultExpression {
    fn get_type(&self) -> Ops {
        Ops::ResExpression
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* IF STATEMENT */
pub trait IfConditionalBase {
    fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct IfConditional {
    pub conditions: Vec<ResultExpression>,
    pub body: Vec<Box<dyn self::AstBase>>,
}

impl IfConditionalBase for IfConditional {
    fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self {
        Self { conditions, body }
    }
}

impl AstBase for IfConditional {
    fn get_type(&self) -> Ops {
        Ops::IfConditional
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* FUNCTION DEFINITION */
pub trait FnDefinitionBase {
    fn get_def_name(&self) -> String;
    fn new(def_name: String, body: Vec<Box<dyn self::AstBase>>, arguments: Vec<String>) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct FnDefinition {
    pub def_name: String,
    pub body: Vec<Box<dyn self::AstBase>>,
    pub arguments: Vec<String>,
}

impl FnDefinitionBase for FnDefinition {
    fn get_def_name(&self) -> String {
        self.def_name.clone()
    }
    fn new(def_name: String, body: Vec<Box<dyn self::AstBase>>, arguments: Vec<String>) -> Self {
        Self {
            def_name,
            body,
            arguments,
        }
    }
}

impl AstBase for FnDefinition {
    fn get_type(&self) -> Ops {
        Ops::FnDef
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* VARIABLE DEFINITION */
pub trait VarDefinitionBase {
    fn get_def_name(&self) -> String;
    fn new(def_name: String, assignment: BoxedValue) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct VarDefinition {
    pub def_name: String,
    pub assignment: BoxedValue,
}

impl VarDefinitionBase for VarDefinition {
    fn get_def_name(&self) -> String {
        self.def_name.clone()
    }
    fn new(def_name: String, assignment: BoxedValue) -> Self {
        Self {
            def_name,
            assignment,
        }
    }
}

impl AstBase for VarDefinition {
    fn get_type(&self) -> Ops {
        Ops::VarDef
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* VARIABLE ASSIGNMENT */
pub trait VarAssignmentBase {
    fn get_def_name(&self) -> String;
    fn new(var_name: String, assignment: BoxedValue) -> Self;
}

#[derive(Clone, Debug, Serialize)]
pub struct VarAssignment {
    pub var_name: String,
    pub assignment: BoxedValue,
}

impl VarAssignmentBase for VarAssignment {
    fn get_def_name(&self) -> String {
        self.var_name.clone()
    }
    fn new(var_name: String, assignment: BoxedValue) -> Self {
        Self {
            var_name,
            assignment,
        }
    }
}

impl AstBase for VarAssignment {
    fn get_type(&self) -> Ops {
        Ops::VarAssign
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* BOXED VALUE */

#[derive(Clone, Debug, Serialize)]
pub struct BoxedValue {
    pub interface: Ops,
    pub value: Box<dyn PrimitiveValueBase>,
}

impl AstBase for BoxedValue {
    fn get_type(&self) -> Ops {
        Ops::LeftAssign
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

impl PrimitiveValueBase for BoxedValue {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* EXPRESSION  */

#[derive(Clone, Debug, Serialize)]
pub struct Expression {
    pub body: Vec<Box<dyn self::AstBase>>,
    pub token_type: Ops,
    pub expr_id: String,
}

impl AstBase for Expression {
    fn get_type(&self) -> Ops {
        Ops::Expression
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait ExpressionBase {
    fn new() -> Self;
    fn from_body(body: Vec<Box<dyn self::AstBase>>) -> Self;
}

impl ExpressionBase for Expression {
    fn new() -> Self {
        Self {
            token_type: Ops::Expression,
            body: Vec::new(),
            expr_id: Uuid::new_v4().to_string(),
        }
    }
    fn from_body(body: Vec<Box<dyn self::AstBase>>) -> Self {
        Self {
            token_type: Ops::Expression,
            body,
            // TODO: Move away from Uuid
            expr_id: Uuid::new_v4().to_string(),
        }
    }
}

/* FUNCTION CALL  */

#[derive(Clone, Serialize, Debug)]
pub struct FnCall {
    pub token_type: Ops,
    pub fn_name: String,
    pub arguments: Vec<BoxedValue>,
    pub reference_to: Option<String>,
}

impl AstBase for FnCall {
    fn get_type(&self) -> Ops {
        Ops::FnCall
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait FnCallBase {
    fn new(fn_name: String, reference_to: Option<String>) -> Self;
}

impl FnCallBase for FnCall {
    fn new(fn_name: String, reference_to: Option<String>) -> Self {
        Self {
            token_type: Ops::FnCall,
            fn_name,
            arguments: Vec::new(),
            reference_to,
        }
    }
}

impl PrimitiveValueBase for FnCall {
    fn as_self(&self) -> &dyn Any {
        self
    }
}

/* WHILE BLOCK  */

#[derive(Clone, Debug, Serialize)]
pub struct While {
    pub body: Vec<Box<dyn self::AstBase>>,
    pub conditions: Vec<ResultExpression>,
}

impl AstBase for While {
    fn get_type(&self) -> Ops {
        Ops::WhileDef
    }
    fn as_self(&self) -> &dyn Any {
        self
    }
}

pub trait WhileBase {
    fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self;
}

impl WhileBase for While {
    fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self {
        Self { body, conditions }
    }
}

/*
 * Get all the tokens with index starting on `from` until a token matches its type to `to`
 */
pub fn get_tokens_from_to_fn(
    from: usize,
    to: Ops,
    tokens: TokensList,
    direction: Directions,
) -> IndexedTokenList {
    let mut found_tokens = Vec::new();

    // Init token position
    let mut token_n = from;

    match direction {
        // Get tokens from left to right
        Directions::LeftToRight => {
            while token_n < tokens.len() {
                if tokens[token_n].ast_type == to {
                    break;
                } else {
                    found_tokens.push((token_n, tokens[token_n].clone()))
                }
                token_n += 1;
            }
        }

        // Get tokens from right to left
        Directions::RightToLeft => {
            while token_n > 0 {
                if tokens[token_n - 1].ast_type == to {
                    break;
                } else {
                    found_tokens.push((token_n - 1, tokens[token_n - 1].clone()))
                }
                token_n -= 1
            }

            found_tokens.reverse();
        }
    }

    found_tokens
}

/*
 * Get the right AST token given a simple token
 */
pub fn get_assignment_token_fn(
    val: String,
    token_n: usize,
    tokens: TokensList,
    direction: Directions,
) -> (usize, BoxedValue) {
    match val.as_str() {
        // True boolean
        "true" => (
            1,
            BoxedValue {
                interface: Ops::Boolean,
                value: Box::new(Boolean::new(true)),
            },
        ),
        // False boolean
        "false" => (
            1,
            BoxedValue {
                interface: Ops::Boolean,
                value: Box::new(Boolean::new(false)),
            },
        ),
        // Numeric values
        val if val.parse::<usize>().is_ok() => (
            1,
            BoxedValue {
                interface: Ops::Number,
                value: Box::new(Number::new(val.parse::<usize>().unwrap())),
            },
        ),
        // String values
        val if val.starts_with('"') && val.ends_with('"') => (
            1,
            BoxedValue {
                interface: Ops::String,
                value: Box::new(StringVal::new(val.replace('"', ""))),
            },
        ),
        // References to other values (ej: referencing to a variable)
        val => {
            if token_n < tokens.len() - 1 {
                let next_token = {
                    match direction {
                        Directions::LeftToRight => tokens[token_n + 1].clone(),
                        _ => tokens[token_n - 1].clone(),
                    }
                };

                let reference_type = match next_token.ast_type {
                    Ops::OpenParent => Ops::FnCall,
                    Ops::CloseParent => Ops::FnCall,
                    Ops::PropAccess => Ops::PropAccess,
                    _ => Ops::Invalid,
                };

                match reference_type {
                    Ops::PropAccess => {
                        let after_next_token = tokens[token_n + 2].clone();
                        let (size, val) = get_assignment_token_fn(
                            after_next_token.value,
                            token_n + 2,
                            tokens,
                            Directions::LeftToRight,
                        );

                        (size + 2, val)
                    }

                    Ops::FnCall => {
                        // Position where it will be starting getting the argument tokens
                        let starting_token: usize = {
                            match direction {
                                Directions::LeftToRight => token_n + 2,
                                _ => token_n,
                            }
                        };

                        // Get argument tokens
                        let mut arguments_tokens: Vec<(usize, Token)> = {
                            match direction {
                                Directions::LeftToRight => get_tokens_from_to_fn(
                                    starting_token,
                                    Ops::CloseParent,
                                    tokens.clone(),
                                    direction.clone(),
                                ),
                                // WIP
                                Directions::RightToLeft => get_tokens_from_to_fn(
                                    starting_token,
                                    Ops::IfConditional,
                                    tokens.clone(),
                                    direction.clone(),
                                ),
                            }
                        };

                        let mut ast_token = FnCall::new(
                            {
                                match direction {
                                    // When reading from left to right, we know current token.value is it's name
                                    Directions::LeftToRight => String::from(val),

                                    // But when reading from right to left we need to first get all the tokens which are part of the function
                                    Directions::RightToLeft => {
                                        let fn_name = arguments_tokens[0].1.value.clone();

                                        // Now we can remove thefunction name from the arguments token
                                        arguments_tokens.remove(0);
                                        fn_name
                                    }
                                }
                            },
                            {
                                if token_n > 0 {
                                    let previous_token = tokens[token_n - 1].clone();
                                    match previous_token.ast_type {
                                        Ops::PropAccess => Some(tokens[token_n - 2].value.clone()),
                                        _ => None,
                                    }
                                } else {
                                    None
                                }
                            },
                        );

                        // Transfrom the tokens into arguments
                        ast_token.arguments = convert_tokens_into_arguments(
                            arguments_tokens
                                .iter()
                                .map(|(_, token)| token.clone())
                                .collect(),
                        );

                        (
                            arguments_tokens.len() + 3,
                            BoxedValue {
                                interface: Ops::FnCall,
                                value: Box::new(ast_token),
                            },
                        )
                    }
                    _ => (
                        1,
                        BoxedValue {
                            interface: Ops::Reference,
                            value: Box::new(Reference::new(String::from(val))),
                        },
                    ),
                }
            } else {
                (
                    1,
                    BoxedValue {
                        interface: Ops::Reference,
                        value: Box::new(Reference::new(String::from(val))),
                    },
                )
            }
        }
    }
}

/*
 * Convert some tokens into function arguments
 */
pub fn convert_tokens_into_arguments(tokens: TokensList) -> Vec<BoxedValue> {
    let mut args = Vec::new();

    let mut token_n = 0;

    while token_n < tokens.len() {
        let token = tokens[token_n].clone();

        match token.ast_type {
            // Ignore ( ) and ,
            Ops::OpenParent => token_n += 1,
            Ops::CloseParent => token_n += 1,
            Ops::CommaDelimiter => token_n += 1,
            _ => {
                let assigned_token = get_assignment_token_fn(
                    token.value.clone(),
                    token_n,
                    tokens.clone(),
                    Directions::LeftToRight,
                );

                match assigned_token.1.interface {
                    Ops::FnCall => token_n += assigned_token.0 + 1,
                    _ => token_n += 1,
                }

                args.push(assigned_token.1);
            }
        }
    }

    args
}

/*
 * Convert some tokens into a list of boolean expressions
 */
pub fn convert_tokens_into_res_expressions(tokens: TokensList) -> Vec<ResultExpression> {
    let mut exprs = Vec::new();

    let mut token_n = 1;

    while token_n < tokens.len() {
        let left_token = tokens[token_n - 1].clone();
        let token = tokens[token_n].clone();

        match token.ast_type {
            Ops::EqualCondition | Ops::NotEqualCondition => {
                let right_token = tokens[token_n + 1].clone();

                let left_token = get_assignment_token_fn(
                    left_token.value.clone(),
                    token_n,
                    tokens.clone(),
                    Directions::RightToLeft,
                );

                let right_token = get_assignment_token_fn(
                    right_token.value.clone(),
                    token_n + 1,
                    tokens.clone(),
                    Directions::LeftToRight,
                );

                exprs.push(ResultExpression::new(
                    token.ast_type,
                    left_token.1.clone(),
                    right_token.1.clone(),
                ));

                token_n += 2;
            }
            _ => {
                token_n += 1;
            }
        }
    }

    exprs
}
