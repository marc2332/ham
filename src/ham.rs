use crate::ast::ast_operations;
use crate::ast::ast_operations::{
    AstBase, BoxedValue, ExpressionBase, FnCallBase, FnDefinitionBase, IfConditionalBase,
    VarAssignmentBase, VarDefinitionBase,
};
use crate::runtime::{
    convert_tokens_into_arguments, convert_tokens_into_res_expressions, downcast_val,
    get_assignment_token_fn, get_func_fn, get_methods_in_type, get_tokens_from_to_fn,
    get_var_reference_fn, modify_var, resolve_reference, value_to_string,
};
use crate::stack::{FunctionDef, Stack, VariableDef};
use crate::types::{IndexedTokenList, LinesList, Token, TokensList};
use crate::utils::op_codes::Directions;
use crate::utils::{errors, op_codes, primitive_values};
use regex::Regex;
use std::sync::Mutex;

/*
 * Split the text by the passed regex but also keep these words which are removed when splitting
 */
fn split<'a>(r: &'a Regex, text: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in text.match_indices(r) {
        if last != index {
            result.push(&text[last..index]);
        }
        result.push(matched);

        last = index + matched.len();
    }
    if last < text.len() {
        result.push(&text[last..]);
    }
    result
}

/*
 * Transform the code into lines
 */
fn get_lines(code: String) -> LinesList {
    let mut lines = Vec::new();

    // Every line
    for line in code.split("\n") {
        let line = String::from(line);
        let mut line_ast = Vec::new();

        let re = Regex::new(r"[\s+,.:]|([()])").unwrap();

        // Every detected word
        for word in split(&re, &line) {
            // Prevent empty words
            if word.trim() != "" {
                line_ast.push(String::from(word.trim()));
            }
        }
        lines.push(line_ast);
    }

    return lines;
}

/*
 * Trasnform a list of lines into a tokens list
 */
fn transform_into_tokens(lines: LinesList) -> TokensList {
    let mut tokens = Vec::new();

    for line in lines {
        for word in line {
            let word = String::from(word);

            let token_type: op_codes::Val = match word.as_str() {
                "let" => op_codes::VAR_DEF,
                "=" => op_codes::LEFT_ASSIGN,
                "(" => op_codes::OPEN_PARENT,
                ")" => op_codes::CLOSE_PARENT,
                "fn" => op_codes::FN_DEF,
                "{" => op_codes::OPEN_BLOCK,
                "}" => op_codes::CLOSE_BLOCK,
                "if" => op_codes::IF_CONDITIONAL,
                "==" => op_codes::EQUAL_CONDITION,
                "return" => op_codes::RETURN,
                "." => op_codes::PROP_ACCESS,
                _ => op_codes::REFERENCE,
            };

            let ast_token = Token {
                ast_type: token_type,
                value: word.clone(),
            };

            tokens.push(ast_token);
        }
    }

    return tokens;
}

/*
 * Transform the code into a list of tokens
 */
pub fn get_tokens(code: String) -> TokensList {
    let lines = self::get_lines(code);
    return self::transform_into_tokens(lines);
}

pub fn move_tokens_into_ast(tokens: TokensList, ast_tree: &Mutex<ast_operations::Expression>) {
    let mut ast_tree = ast_tree.lock().unwrap();

    // Closure version of above
    let get_tokens_from_to = |from: usize, to: op_codes::Val| -> IndexedTokenList {
        return get_tokens_from_to_fn(from, to, tokens.clone(), Directions::LeftToRight);
    };

    // Get all the tokens in a group (expression blocks, arguments)
    let get_tokens_in_group_of =
        |from: usize, open_tok: op_codes::Val, close_tok: op_codes::Val| -> TokensList {
            let mut found_tokens = Vec::new();

            let mut count = 0;

            let mut token_n = from;

            while token_n < tokens.len() {
                let token = tokens[token_n].clone();

                if token.ast_type == open_tok {
                    count += 1;
                } else if token.ast_type == close_tok {
                    count -= 1;
                }

                if count == 0 {
                    break;
                } else if token_n > from {
                    found_tokens.push(token.clone());
                }
                token_n += 1;
            }

            found_tokens
        };

    let get_assignment_token = |val: String, token_n: usize| -> ast_operations::BoxedValue {
        return get_assignment_token_fn(val, token_n, tokens.clone(), Directions::LeftToRight);
    };

    let mut token_n = 0;

    while token_n < tokens.len() {
        let current_token = &tokens[token_n];
        match current_token.ast_type {
            // Return statement
            op_codes::RETURN => {
                let next_token = tokens[token_n + 1].clone();

                let return_val = get_assignment_token(next_token.value.clone(), token_n.clone());

                let ast_token = ast_operations::ReturnStatement { value: return_val };
                ast_tree.body.push(Box::new(ast_token));

                token_n += 2;
            }

            // If statement
            op_codes::IF_CONDITIONAL => {
                // Get the if condition tokens
                let condition_tokens = get_tokens_from_to(token_n + 1, op_codes::OPEN_BLOCK);

                // Transform those tokens into result expressions
                let exprs = convert_tokens_into_res_expressions(
                    condition_tokens
                        .clone()
                        .iter()
                        .map(|(_, token)| token.clone())
                        .collect(),
                );

                // Scope tree
                let scope_tree = Mutex::new(ast_operations::Expression::new());

                // Ignore the if conditions and {
                let open_block_index = token_n + condition_tokens.len() + 1;

                // Get all tokens inside the if block
                let block_tokens = get_tokens_in_group_of(
                    open_block_index,
                    op_codes::OPEN_BLOCK,
                    op_codes::CLOSE_BLOCK,
                );

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree);

                // Ignore the function body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a function definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token = ast_operations::IfConditional::new(exprs.clone(), body.to_vec());
                ast_tree.body.push(Box::new(ast_token));
            }

            op_codes::PROP_ACCESS => {
                let previous_token = tokens[token_n - 1].clone();
                let next_token = tokens[token_n + 1].clone();

                match next_token.ast_type {
                    op_codes::REFERENCE => {
                        let after_next_token = tokens[token_n + 2].clone();

                        let reference_type = match after_next_token.ast_type {
                            op_codes::OPEN_PARENT => op_codes::FN_CALL,
                            _ => 0,
                        };

                        match reference_type {
                            op_codes::FN_CALL => {
                                let mut ast_token = ast_operations::FnCall::new(
                                    next_token.value.clone(),
                                    previous_token.value,
                                );

                                // Ignore itself and the (
                                let starting_token = token_n + 2;

                                let arguments_tokens = get_tokens_in_group_of(
                                    starting_token,
                                    op_codes::OPEN_PARENT,
                                    op_codes::CLOSE_PARENT,
                                );
                                let arguments =
                                    convert_tokens_into_arguments(arguments_tokens.clone());

                                token_n += 2 + arguments_tokens.len();

                                ast_token.arguments = arguments;
                                ast_tree.body.push(Box::new(ast_token));
                            }
                            _ => (),
                        };
                    }
                    _ => (),
                }
            }

            // Function definition
            op_codes::FN_DEF => {
                let def_name = String::from(&tokens[token_n + 1].value.clone());

                // Scope tree
                let scope_tree = Mutex::new(ast_operations::Expression::new());

                // Ignore function name and the (
                let starting_token = token_n + 2;

                // Get function arguments, WIP
                let arguments: Vec<String> = get_tokens_in_group_of(
                    starting_token,
                    op_codes::OPEN_PARENT,
                    op_codes::CLOSE_PARENT,
                )
                .iter()
                .map(|token| token.value.clone())
                .collect();

                // Ignore function name, (, arguments and )
                let open_block_index = starting_token + arguments.len() + 2;

                // Get all tokens inside the function block

                let block_tokens = get_tokens_in_group_of(
                    open_block_index,
                    op_codes::OPEN_BLOCK,
                    op_codes::CLOSE_BLOCK,
                );

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree);

                // Ignore the function body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a function definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token =
                    ast_operations::FnDefinition::new(def_name, body.to_vec(), arguments);
                ast_tree.body.push(Box::new(ast_token));
            }
            // Variable definition
            op_codes::VAR_DEF => {
                let next_token = tokens[token_n + 1].clone();

                // Variable name
                let def_name = String::from(next_token.value.clone());

                // Value token position
                let val_index = token_n + 3;

                // Stringified value
                let def_value = String::from(&tokens[val_index].value.clone());

                let assignment = get_assignment_token(def_value, val_index);

                let ast_token = ast_operations::VarDefinition::new(def_name, assignment);
                ast_tree.body.push(Box::new(ast_token));

                token_n += 4;
            }
            // References (fn calls, variable reassignation...)
            op_codes::REFERENCE => {
                let next_token = tokens[token_n + 1].ast_type;

                let reference_type = match next_token {
                    op_codes::OPEN_PARENT => op_codes::FN_CALL,
                    op_codes::LEFT_ASSIGN => op_codes::VAR_ASSIGN,
                    _ => 0,
                };

                match reference_type {
                    op_codes::VAR_ASSIGN => {
                        let token_after_equal = tokens[token_n + 2].clone();

                        let assignment =
                            get_assignment_token(token_after_equal.value.clone(), token_n.clone());

                        let ast_token = ast_operations::VarAssignment::new(
                            current_token.value.clone(),
                            assignment,
                        );

                        ast_tree.body.push(Box::new(ast_token));

                        token_n += 3;
                    }
                    op_codes::FN_CALL => {
                        let mut ast_token = ast_operations::FnCall::new(
                            current_token.value.clone(),
                            String::from(""),
                        );

                        // Ignore itself and the (
                        let starting_token = token_n + 1;

                        let arguments_tokens = get_tokens_in_group_of(
                            starting_token,
                            op_codes::OPEN_PARENT,
                            op_codes::CLOSE_PARENT,
                        );
                        let arguments = convert_tokens_into_arguments(arguments_tokens.clone());

                        token_n += 3 + arguments_tokens.len();

                        ast_token.arguments = arguments;
                        ast_tree.body.push(Box::new(ast_token));
                    }
                    _ => {
                        token_n += 1;
                        ()
                    }
                }
            }
            _ => {
                token_n += 1;
            }
        }
    }
}

pub fn run_ast(
    ast: &Mutex<ast_operations::Expression>,
    stack: &Mutex<Stack>,
) -> Result<ast_operations::BoxedValue, ()> {
    let ast = ast.lock().unwrap();

    // Closure version of above
    let get_func = |fn_name: String| -> Result<FunctionDef, ()> {
        return get_func_fn(fn_name, stack.lock().unwrap().functions.clone());
    };

    // Closure version of above
    let resolve_ref =
        |val_type: op_codes::Val,
         ref_val: Box<dyn primitive_values::PrimitiveValueBase>|
         -> Result<(op_codes::Val, Box<dyn primitive_values::PrimitiveValueBase>), ()> {
            return resolve_reference(val_type, ref_val, stack, &ast);
        };

    // Check if a conditional is true or not
    let eval_condition = |condition_code: op_codes::Val,
                          left_val: ast_operations::BoxedValue,
                          right_val: ast_operations::BoxedValue|
     -> bool {
        let left_val = resolve_ref(left_val.interface.clone(), left_val.value.clone());
        let right_val = resolve_ref(right_val.interface.clone(), right_val.value.clone());

        if left_val.is_ok() && right_val.is_ok() {
            let left_val = left_val.unwrap();
            let right_val = right_val.unwrap();

            match condition_code {
                op_codes::EQUAL_CONDITION => {
                    let left_val = value_to_string(left_val);
                    let right_val = value_to_string(right_val);

                    if left_val == right_val {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    };

    /*
     * print() function
     */
    stack.lock().unwrap().functions.push(FunctionDef {
        name: String::from("print"),
        body: vec![],
        arguments: vec![],
        cb: |_, args, _, _, _| {
            print!("{}", args.join(""));
            return Err(());
        },
        expr_id: ast.expr_id.clone(),
    });

    /*
     * println() function
     */
    stack.lock().unwrap().functions.push(FunctionDef {
        name: String::from("println"),
        body: vec![],
        arguments: vec![],
        cb: |_, args, _, _, _| {
            println!("{}", args.join(""));
            return Err(());
        },
        expr_id: ast.expr_id.clone(),
    });

    for op in &ast.body {
        match op.get_type() {
            /*
             * Handle return statements
             */
            op_codes::RETURN => {
                let statement = downcast_val::<ast_operations::ReturnStatement>(op.as_self());

                let ret_type = statement.value.interface;
                let ret_val = statement.value.value.clone();

                let ret_ref = resolve_ref(ret_type, ret_val);

                if ret_ref.is_ok() {
                    let (ref_type, ref_val) = ret_ref.unwrap();

                    return Ok(BoxedValue {
                        interface: ref_type,
                        value: ref_val.clone(),
                    });
                }
            }

            /*
             * Handle if statements
             */
            op_codes::IF_CONDITIONAL => {
                let if_statement = downcast_val::<ast_operations::IfConditional>(op.as_self());

                let mut true_count = 0;

                for condition in if_statement.conditions.clone() {
                    let res = eval_condition(condition.relation, condition.left, condition.right);

                    if res == true {
                        true_count += 1;
                    }
                }
                if true_count == if_statement.conditions.len() {
                    let expr = ast_operations::Expression::from_body(if_statement.body.clone());
                    let expr_id = expr.expr_id.clone();

                    let if_block_return = run_ast(&Mutex::new(expr), stack);

                    if if_block_return.is_ok() {
                        return Ok(if_block_return.unwrap());
                    }

                    stack.lock().unwrap().drop_ops_from_id(expr_id.clone());
                }
            }

            /*
             * Handle function definitions
             */
            op_codes::FN_DEF => {
                let function = downcast_val::<ast_operations::FnDefinition>(op.as_self());

                stack.lock().unwrap().functions.push(FunctionDef {
                    name: String::from(function.def_name.clone()),
                    body: function.body.clone(),
                    arguments: function.arguments.clone(),
                    cb: |args, args_vals, body: Vec<Box<dyn self::AstBase>>, stack, _| {
                        let expr = ast_operations::Expression::from_body(body.clone());
                        let expr_id = expr.expr_id.clone();

                        for (i, arg) in args_vals.clone().iter().enumerate() {
                            let arg_name = args[i].clone();

                            stack.lock().unwrap().variables.push(VariableDef {
                                name: String::from(arg_name),
                                value: Box::new(primitive_values::StringVal(String::from(arg))),
                                val_type: op_codes::STRING,
                                expr_id: expr_id.clone(),
                                methods: Vec::new(),
                            });
                        }

                        let return_val = run_ast(&Mutex::new(expr), stack);

                        stack.lock().unwrap().drop_ops_from_id(expr_id.clone());

                        return return_val;
                    },
                    expr_id: ast.expr_id.clone(),
                });
            }

            /*
             * Handle variables definitions
             */
            op_codes::VAR_DEF => {
                let variable = downcast_val::<ast_operations::VarDefinition>(op.as_self());

                let val_type = variable.assignment.interface;
                let ref_val = variable.assignment.value.clone();

                let var_ref = resolve_ref(val_type, ref_val);

                if var_ref.is_ok() {
                    let (ref_type, ref_val) = var_ref.unwrap();

                    if !op_codes::is_valid(ref_type) {
                        /*
                         * If value code is not valid then raise an error
                         */
                        errors::raise_error(
                            errors::UNHANDLED_VALUE_TYPE_CODE,
                            vec![val_type.to_string()],
                        )
                    } else {
                        stack.lock().unwrap().variables.push(VariableDef {
                            name: variable.def_name.clone(),
                            val_type: ref_type,
                            value: ref_val,
                            expr_id: ast.expr_id.clone(),
                            methods: get_methods_in_type(ref_type),
                        });
                    }
                }
            }

            /*
             * Handle variable assignments
             */
            op_codes::VAR_ASSIGN => {
                let variable = downcast_val::<ast_operations::VarAssignment>(op.as_self());

                modify_var(
                    stack,
                    variable.var_name.clone(),
                    variable.assignment.value.clone(),
                );
            }

            /*
             * Handle function calls
             */
            op_codes::FN_CALL => {
                let fn_call = downcast_val::<ast_operations::FnCall>(op.as_self());

                let is_referenced = fn_call.reference_to != "";

                let ref_fn = if is_referenced {
                    let ref_var = get_var_reference_fn(stack, fn_call.reference_to.clone());

                    get_func_fn(fn_call.fn_name.clone(), ref_var.unwrap().methods.clone())
                } else {
                    get_func(fn_call.fn_name.clone())
                };

                // If the calling function is found
                if ref_fn.is_ok() {
                    let mut arguments = Vec::new();

                    if is_referenced {
                        arguments.push(fn_call.reference_to.clone());
                    }

                    for argument in &fn_call.arguments {
                        let arg_ref = resolve_ref(argument.interface, argument.value.clone());

                        if arg_ref.is_ok() {
                            let arg_ref = arg_ref.unwrap();

                            let argument_stringified = value_to_string(arg_ref.clone());

                            if argument_stringified.is_ok() {
                                arguments.push(argument_stringified.unwrap());
                            } else {
                                errors::raise_error(
                                    errors::BROKEN_ARGUMENT,
                                    vec![argument.interface.to_string()],
                                )
                            }
                        }
                    }

                    let func = ref_fn.unwrap();

                    let res_func =
                        (func.cb)(func.arguments, arguments.clone(), func.body, &stack, &ast);

                    if res_func.is_ok() {
                        let ret_val = res_func.unwrap();

                        let val_stringified = value_to_string((ret_val.interface, ret_val.value));

                        if val_stringified.is_ok() {
                            let val_stringified = val_stringified.unwrap();

                            // The function returned something that ends up not being used, throw error
                            errors::raise_error(
                                errors::RETURNED_VALUE_NOT_USED,
                                vec![
                                    val_stringified,
                                    fn_call.fn_name.clone(),
                                    arguments.join(" "),
                                ],
                            )
                        }
                    } else {
                        // No value returned, OK
                    }
                }
            }
            _ => {
                panic!("Unhandled code operation")
            }
        }
    }
    Err(())
}
