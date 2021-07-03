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

use self::{
    boxed_val::BoxedValue,
    expression::{
        Expression,
        ExpressionBase,
    },
    fn_call::{
        FnCall,
        FnCallBase,
    },
    reference::{
        Reference,
        ReferenceValueBase,
    },
    result::{
        ResultExpression,
        ResultExpressionBase,
    },
};

pub mod ast_base;
pub mod boxed_val;
pub mod break_ast;
pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod if_ast;
pub mod module;
pub mod reference;
pub mod result;
pub mod return_ast;
pub mod var_assign;
pub mod var_def;
pub mod while_block;

use return_ast::ReturnStatement;

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
                        // Get the next token when reading from left to right
                        Directions::LeftToRight => tokens[token_n + 1].clone(),
                        // Get the previous token when reading from right to left
                        Directions::RightToLeft => tokens[token_n - 1].clone(),
                    }
                };

                let reference_type = match next_token.ast_type {
                    // Having a ( or ) means there is a function call
                    Ops::OpenParent => Ops::FnCall,
                    Ops::CloseParent => Ops::FnCall,
                    // Having a . means it's referencing to another value
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
                    /*
                     * Handle function calls used as assignment value
                     */
                    Ops::FnCall => {
                        let mut group = Expression::new();

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
                                /*
                                 * Get all the next tokens from the starting token to the first CloseParent token
                                 */
                                Directions::LeftToRight => get_tokens_from_to_fn(
                                    starting_token,
                                    Ops::CloseParent,
                                    tokens.clone(),
                                    direction.clone(),
                                ),
                                /*
                                 * get all previous the tokens the starting token until an IfConditional token
                                 * [WIP]
                                 */
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

                                        // Now we can remove the function name from the arguments token
                                        arguments_tokens.remove(0);
                                        fn_name
                                    }
                                }
                            },
                            {
                                if token_n > 0 {
                                    let previous_token = tokens[token_n - 1].clone();
                                    match previous_token.ast_type {
                                        // Is referencing to a value on the left side
                                        Ops::PropAccess => {
                                            let before_previous_token = tokens[token_n - 2].clone();
                                            match before_previous_token.ast_type {
                                                // Reference to a variable
                                                Ops::Reference => Some(before_previous_token.value),
                                                // Reference to the result of a function call
                                                Ops::CloseParent => Some("_".to_string()),
                                                _ => None,
                                            }
                                        }
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

                        let mut left_length = arguments_tokens.len() + 3;

                        // Assign a expression to the variable
                        group.body.push(Box::new(var_def::VarDefinition {
                            def_name: "_".to_string(),
                            assignment: BoxedValue {
                                interface: Ops::FnCall,
                                value: Box::new(ast_token),
                            },
                        }));

                        let mut next_token_n = token_n + left_length;

                        if next_token_n < tokens.len() - 1 {
                            let next_token = &tokens[next_token_n];

                            /*
                             * If the next token is also referencing to the value on the left
                             * then add it to the expression block
                             */
                            if next_token.ast_type == Ops::PropAccess {
                                next_token_n += 1;
                                let next_token = &tokens[next_token_n];

                                /*
                                 * Get the token next to the PropAccess token
                                 */
                                let (length, val) = get_assignment_token_fn(
                                    next_token.value.clone(),
                                    next_token_n,
                                    tokens,
                                    Directions::LeftToRight,
                                );

                                /*
                                 * Return the token from the expression
                                 */
                                group.body.push(Box::new(ReturnStatement { value: val }));
                                left_length += length;
                            } else {
                                /*
                                 * If the next token is no longer a PropAccess
                                 * it means it is not referencing to the function
                                 * call's result
                                 */
                                group.body.push(Box::new(ReturnStatement {
                                    value: BoxedValue {
                                        interface: Ops::Reference,
                                        value: Box::new(Reference::new("_".to_string())),
                                    },
                                }));
                            }
                        } else {
                            /*
                             * If there aren't any more tokens then just return a reference to the variable
                             */
                            group.body.push(Box::new(ReturnStatement {
                                value: BoxedValue {
                                    interface: Ops::Reference,
                                    value: Box::new(Reference::new("_".to_string())),
                                },
                            }));
                        }

                        (
                            left_length,
                            BoxedValue {
                                interface: Ops::Expression,
                                value: Box::new(group),
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
                let (size, val) = get_assignment_token_fn(
                    token.value.clone(),
                    token_n,
                    tokens.clone(),
                    Directions::LeftToRight,
                );

                token_n += size + 1;

                args.push(val);
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
