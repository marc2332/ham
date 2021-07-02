use crate::{
    ast_types::{
        boxed_val::BoxedValue,
        break_ast::{
            Break,
            BreakBase,
        },
        convert_tokens_into_arguments,
        convert_tokens_into_res_expressions,
        expression::{
            Expression,
            ExpressionBase,
        },
        fn_call::{
            FnCall,
            FnCallBase,
        },
        fn_def::{
            FnDefinition,
            FnDefinitionBase,
        },
        get_assignment_token_fn,
        get_tokens_from_to_fn,
        if_ast::{
            IfConditional,
            IfConditionalBase,
        },
        module::Module,
        return_ast::ReturnStatement,
        var_assign::{
            VarAssignment,
            VarAssignmentBase,
        },
        var_def::{
            VarDefinition,
            VarDefinitionBase,
        },
        while_block::{
            While,
            WhileBase,
        },
    },
    runtime::{
        downcast_val,
        get_methods_in_type,
        resolve_reference,
        value_to_string,
        values_to_strings,
    },
    stack::{
        FunctionDef,
        FunctionsContainer,
        Stack,
        VariableDef,
    },
    types::{
        BoxedPrimitiveValue,
        IndexedTokenList,
        LinesList,
        Token,
        TokensList,
    },
    utils::{
        errors,
        Directions,
        Ops,
    },
};
use regex::Regex;
use std::{
    collections::HashMap,
    fs,
    sync::Mutex,
};
use uuid::Uuid;

pub mod ast_types;
pub mod primitive_values;
pub mod runtime;
pub mod stack;
pub mod types;
pub mod utils;

use primitive_values::string::StringVal;

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
    for line in code.split('\n') {
        // Ignore // comments
        if line.starts_with("//") {
            continue;
        }

        let mut line_ast = Vec::new();

        let re = Regex::new(r#"([\s+,.:])|("(.*?)")|([()])"#).unwrap();

        // Every detected word
        for word in split(&re, line) {
            // Prevent empty words
            if word.trim() != "" {
                line_ast.push(String::from(word.trim()));
            }
        }
        lines.push(line_ast);
    }

    lines
}

/*
 * Trasnform a list of lines into a tokens list
 */
fn transform_into_tokens(lines: LinesList) -> TokensList {
    let mut tokens = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        for word in line {
            let token_type: Ops = match word.as_str() {
                "let" => Ops::VarDef,
                "=" => Ops::LeftAssign,
                "(" => Ops::OpenParent,
                ")" => Ops::CloseParent,
                "fn" => Ops::FnDef,
                "{" => Ops::OpenBlock,
                "}" => Ops::CloseBlock,
                "if" => Ops::IfConditional,
                "==" => Ops::EqualCondition,
                "return" => Ops::Return,
                "." => Ops::PropAccess,
                "," => Ops::CommaDelimiter,
                "while" => Ops::WhileDef,
                "!=" => Ops::NotEqualCondition,
                "import" => Ops::Import,
                "from" => Ops::FromModule,
                "break" => Ops::Break,
                _ => Ops::Reference,
            };

            let ast_token = Token {
                ast_type: token_type,
                value: word.clone(),
                line: i + 1,
            };

            tokens.push(ast_token);
        }
    }

    tokens
}

/*
 * Transform the code into a list of tokens
 */
pub fn get_tokens(code: String) -> TokensList {
    let lines = self::get_lines(code);
    self::transform_into_tokens(lines)
}

pub fn move_tokens_into_ast(tokens: TokensList, ast_tree: &Mutex<Expression>, filedir: String) {
    let mut ast_tree = ast_tree.lock().unwrap();

    // Closure version of above
    let get_tokens_from_to = |from: usize, to: Ops| -> IndexedTokenList {
        get_tokens_from_to_fn(from, to, tokens.clone(), Directions::LeftToRight)
    };

    // Get all the tokens in a group (expression blocks, arguments)
    let get_tokens_in_group_of = |from: usize, open_tok: Ops, close_tok: Ops| -> TokensList {
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

    let get_assignment_token = |val: String, token_n: usize| -> (usize, BoxedValue) {
        get_assignment_token_fn(val, token_n, tokens.clone(), Directions::LeftToRight)
    };

    let mut token_n = 0;

    while token_n < tokens.len() {
        let current_token = &tokens[token_n];
        match current_token.ast_type {
            Ops::Break => {
                let break_ast = Break::new();
                ast_tree.body.push(Box::new(break_ast));
                token_n += 1;
            }

            // Import statement
            Ops::Import => {
                let module_name = &tokens[token_n + 1].value;
                let module_direction = &tokens[token_n + 2];
                let module_origin = &tokens[token_n + 3].value;
                /*
                 * import x from "./x.ham"
                 *
                 * module_name: x
                 * module_direction: from
                 * module_origin= "./x.ham"
                 */

                if module_direction.ast_type != Ops::FromModule {
                    errors::raise_error(
                        errors::CODES::UnexpectedKeyword,
                        vec![module_direction.value.clone()],
                    )
                }
                // Module's path
                let filepath = format!("{}/{}", filedir, module_origin.replace('"', ""));

                // Module's code
                let filecontent = fs::read_to_string(filepath.as_str());

                if let Ok(filecontent) = filecontent {
                    let tokens = get_tokens(filecontent);

                    // Move all the tokens into a expression
                    let scope_tree = Mutex::new(Expression::new());
                    move_tokens_into_ast(tokens.clone(), &scope_tree, filedir.clone());

                    // Copy all root-functions (public by default) from the expression body to the vector
                    let mut public_functions = Vec::new();

                    for op in scope_tree.lock().unwrap().body.iter() {
                        if op.get_type() == Ops::FnDef {
                            public_functions
                                .push(downcast_val::<FnDefinition>(op.as_self()).clone());
                        }
                    }

                    let module = Module {
                        name: module_name.to_string(),
                        functions: public_functions,
                    };

                    ast_tree.body.push(Box::new(module));
                } else {
                    errors::raise_error(errors::CODES::ModuleNotFound, vec![filepath])
                }

                token_n += 4
            }

            // While block
            Ops::WhileDef => {
                // Get the if condition tokens
                let condition_tokens = get_tokens_from_to(token_n + 1, Ops::OpenBlock);

                // Transform those tokens into result expressions
                let exprs = convert_tokens_into_res_expressions(
                    condition_tokens
                        .clone()
                        .iter()
                        .map(|(_, token)| token.clone())
                        .collect(),
                );

                // Scope tree
                let scope_tree = Mutex::new(Expression::new());

                // Ignore the if conditions and {
                let open_block_index = token_n + condition_tokens.len() + 1;

                // Get all tokens inside the if block
                let block_tokens =
                    get_tokens_in_group_of(open_block_index, Ops::OpenBlock, Ops::CloseBlock);

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree, filedir.clone());

                // Ignore the whilte body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a while definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token = While::new(exprs.clone(), body.to_vec());
                ast_tree.body.push(Box::new(ast_token));
            }

            // Return statement
            Ops::Return => {
                let next_token = tokens[token_n + 1].clone();

                let (size, return_val) =
                    get_assignment_token(next_token.value.clone(), token_n + 1);

                let ast_token = ReturnStatement { value: return_val };
                ast_tree.body.push(Box::new(ast_token));

                token_n += 2 + size;
            }

            // If statement
            Ops::IfConditional => {
                // Get the if condition tokens
                let condition_tokens = get_tokens_from_to(token_n + 1, Ops::OpenBlock);

                // Transform those tokens into result expressions
                let exprs = convert_tokens_into_res_expressions(
                    condition_tokens
                        .clone()
                        .iter()
                        .map(|(_, token)| token.clone())
                        .collect(),
                );

                // Scope tree
                let scope_tree = Mutex::new(Expression::new());

                // Ignore the if conditions and {
                let open_block_index = token_n + condition_tokens.len() + 1;

                // Get all tokens inside the if block
                let block_tokens =
                    get_tokens_in_group_of(open_block_index, Ops::OpenBlock, Ops::CloseBlock);

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree, filedir.clone());

                // Ignore the block body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a if block definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token = IfConditional::new(exprs.clone(), body.to_vec());
                ast_tree.body.push(Box::new(ast_token));
            }

            Ops::PropAccess => {
                let previous_token = tokens[token_n - 1].clone();
                let next_token = tokens[token_n + 1].clone();

                if next_token.ast_type == Ops::Reference {
                    let after_next_token = tokens[token_n + 2].clone();

                    let reference_type = match after_next_token.ast_type {
                        Ops::OpenParent => Ops::FnCall,
                        _ => Ops::Invalid,
                    };

                    match reference_type {
                        /*
                         * Call to functions from variables
                         */
                        Ops::FnCall => {
                            let mut ast_token =
                                FnCall::new(next_token.value.clone(), Some(previous_token.value));

                            // Ignore itself and the (
                            let starting_token = token_n + 2;

                            let arguments_tokens = get_tokens_in_group_of(
                                starting_token,
                                Ops::OpenParent,
                                Ops::CloseParent,
                            );
                            let arguments = convert_tokens_into_arguments(arguments_tokens.clone());

                            token_n += 2 + arguments_tokens.len();

                            ast_token.arguments = arguments;
                            ast_tree.body.push(Box::new(ast_token));
                        }
                        /*
                         * TODO: Access properties from varibles
                         */
                        Ops::Reference => {}
                        _ => (),
                    };
                }
            }

            // Function definition
            Ops::FnDef => {
                let def_name = String::from(&tokens[token_n + 1].value.clone());

                // Scope tree
                let scope_tree = Mutex::new(Expression::new());

                // Ignore function name and the (
                let starting_token = token_n + 2;

                // Get function arguments, WIP
                let arguments: Vec<String> =
                    get_tokens_in_group_of(starting_token, Ops::OpenParent, Ops::CloseParent)
                        .iter()
                        .map(|token| token.value.clone())
                        .collect();

                // Ignore function name, (, arguments and )
                let open_block_index = starting_token + arguments.len() + 2;

                // Get all tokens inside the function block

                let block_tokens =
                    get_tokens_in_group_of(open_block_index, Ops::OpenBlock, Ops::CloseBlock);

                // Move the tokens into the tree
                move_tokens_into_ast(block_tokens.clone(), &scope_tree, filedir.clone());

                // Ignore the function body
                token_n = block_tokens.len() + open_block_index + 1;

                // Create a function definition
                let body = &scope_tree.lock().unwrap().body.clone();
                let ast_token = FnDefinition::new(def_name, body.to_vec(), arguments);
                ast_tree.body.push(Box::new(ast_token));
            }
            // Variable definition
            Ops::VarDef => {
                let next_token = tokens[token_n + 1].clone();

                // Variable name
                let def_name = next_token.value.clone();

                // Value token position
                let val_index = token_n + 3;

                // Stringified value
                let def_value = String::from(&tokens[val_index].value.clone());

                let (size, assignment) = get_assignment_token(def_value, val_index);

                let ast_token = VarDefinition::new(def_name, assignment);
                ast_tree.body.push(Box::new(ast_token));

                token_n += 3 + size;
            }
            // References (fn calls, variable reassignation...)
            Ops::Reference => {
                let next_token = &tokens[token_n + 1];

                let reference_type = match next_token.ast_type {
                    Ops::OpenParent => Ops::FnCall,
                    Ops::LeftAssign => Ops::VarAssign,
                    _ => Ops::Invalid,
                };

                match reference_type {
                    Ops::VarAssign => {
                        let token_after_equal = tokens[token_n + 2].clone();

                        let (size, assignment) =
                            get_assignment_token(token_after_equal.value.clone(), token_n + 2);

                        let ast_token = VarAssignment::new(current_token.value.clone(), assignment);

                        ast_tree.body.push(Box::new(ast_token));

                        token_n += 2 + size;
                    }
                    Ops::FnCall => {
                        let mut ast_token = FnCall::new(current_token.value.clone(), None);

                        // Ignore itself and the (
                        let starting_token = token_n + 1;

                        let arguments_tokens = get_tokens_in_group_of(
                            starting_token,
                            Ops::OpenParent,
                            Ops::CloseParent,
                        );
                        let arguments = convert_tokens_into_arguments(arguments_tokens.clone());

                        token_n += 3 + arguments_tokens.len();

                        ast_token.arguments = arguments;

                        ast_tree.body.push(Box::new(ast_token));
                    }
                    _ => {
                        token_n += 1;
                    }
                }
            }
            _ => {
                token_n += 1;
            }
        }
    }
}

fn get_function_from_def(function: &FnDefinition) -> FunctionDef {
    FunctionDef {
        name: function.def_name.clone(),
        body: function.body.clone(),
        arguments: function.arguments.clone(),
        cb: |args, args_vals, body, stack, ast| {
            let expr = Expression::from_body(body.clone());
            let expr_id = expr.expr_id.clone();

            for (i, arg) in args_vals.iter().enumerate() {
                let arg_name = args[i].clone();
                let var_id = stack.lock().unwrap().reseve_index();
                stack.lock().unwrap().push_variable(VariableDef {
                    name: arg_name,
                    value: arg.value.clone(),
                    val_type: arg.interface,
                    expr_id: expr_id.clone(),
                    functions: get_methods_in_type(arg.interface),
                    var_id,
                })
            }

            let return_val = run_ast(&Mutex::new(expr), stack);

            stack.lock().unwrap().drop_ops_from_id(expr_id);

            if let Some(return_val) = return_val {
                resolve_reference(stack, return_val.interface, return_val.value, &ast)
            } else {
                return_val
            }
        },
        // TODO: Move away from Uuid
        expr_id: Uuid::new_v4().to_string(),
    }
}

pub fn run_ast(ast: &Mutex<Expression>, stack: &Mutex<Stack>) -> Option<BoxedValue> {
    let ast = ast.lock().unwrap();

    // Closure version of resolve_reference
    let resolve_ref = |val_type: Ops, ref_val: BoxedPrimitiveValue| -> Option<BoxedValue> {
        resolve_reference(stack, val_type, ref_val, &ast)
    };

    // Check if a conditional is true or not
    let eval_condition =
        |condition_code: Ops, left_val: BoxedValue, right_val: BoxedValue| -> bool {
            let left_val = resolve_ref(left_val.interface, left_val.value.clone());
            let right_val = resolve_ref(right_val.interface, right_val.value.clone());

            if let (Some(left_val), Some(right_val)) = (left_val, right_val) {
                match condition_code {
                    // Handle !=
                    Ops::NotEqualCondition => {
                        let left_val = value_to_string(left_val, stack).unwrap();
                        let right_val = value_to_string(right_val, stack).unwrap();

                        left_val != right_val
                    }
                    // Handle ==
                    Ops::EqualCondition => {
                        let left_val = value_to_string(left_val, stack).unwrap();
                        let right_val = value_to_string(right_val, stack).unwrap();

                        left_val == right_val
                    }
                    _ => false,
                }
            } else {
                false
            }
        };

    for operation in &ast.body {
        match operation.get_type() {
            /*
             * Handle breaks
             */
            Ops::Break => {
                return Some(BoxedValue {
                    interface: Ops::Break,
                    value: Box::new(StringVal("break".to_string())),
                })
            }

            /*
             * Handle module definitions
             */
            Ops::Module => {
                let module = downcast_val::<Module>(operation.as_self());

                let var_id = stack.lock().unwrap().reseve_index();

                let mut functions = HashMap::new();

                for function in &module.functions {
                    let mut function = function.clone();
                    function.arguments.insert(0, "_".to_string());
                    functions.insert(function.def_name.clone(), get_function_from_def(&function));
                }

                // Push the variable into the stack
                stack.lock().unwrap().push_variable(VariableDef {
                    name: module.name.clone(),
                    val_type: Ops::String,
                    value: Box::new(StringVal(module.name.clone())),
                    expr_id: ast.expr_id.clone(),
                    functions,
                    var_id,
                });
            }

            /*
             * Handle if block
             */
            Ops::WhileDef => {
                let while_block = downcast_val::<While>(operation.as_self());

                let check_while = |while_block: &While| -> Option<BoxedValue> {
                    /*
                     * Evaluate all conditions,
                     * If all they return true then execute the IF's expression block
                     */
                    let mut true_count = 0;

                    for condition in while_block.conditions.clone() {
                        let res =
                            eval_condition(condition.relation, condition.left, condition.right);

                        if res {
                            true_count += 1;
                        }
                    }

                    if true_count == while_block.conditions.len() {
                        let expr = Expression::from_body(while_block.body.clone());
                        let expr_id = expr.expr_id.clone();

                        // Execute the expression block
                        let if_block_return = run_ast(&Mutex::new(expr), stack);

                        /*
                         * While's loop will stop when something is returned forcefully
                         */
                        if let Some(if_block_return) = if_block_return {
                            return Some(if_block_return);
                        }

                        // Clean the expression definitions from the stack
                        stack.lock().unwrap().drop_ops_from_id(expr_id);
                        Some(BoxedValue {
                            value: Box::new(StringVal("while".to_string())),
                            interface: Ops::WhileDef,
                        })
                    } else {
                        None
                    }
                };

                let mut stopped = false;

                while !stopped {
                    let res = check_while(while_block);

                    if let Some(res) = res {
                        match res.interface {
                            Ops::WhileDef => {
                                // Ignore non-returning whiles
                            }
                            Ops::Break => {
                                // Simply stop the while
                                stopped = true;
                            }
                            _ => {
                                // Stop and return the value
                                return Some(res);
                            }
                        }
                    } else {
                        stopped = true;
                    }
                }
            }

            /*
             * Handle return statements
             */
            Ops::Return => {
                let statement = downcast_val::<ReturnStatement>(operation.as_self());

                // Type of return
                let return_type = statement.value.interface;

                // Value returning
                let return_val = statement.value.value.clone();

                // Pimitive value to return
                let return_val = resolve_ref(return_type, return_val);

                return return_val;
            }

            /*
             * Handle if statements
             */
            Ops::IfConditional => {
                let if_statement = downcast_val::<IfConditional>(operation.as_self());

                /*
                 * Evaluate all conditions,
                 * If all they return true then execute the IF's expression block
                 */
                let mut true_count = 0;

                for condition in if_statement.conditions.clone() {
                    let res = eval_condition(condition.relation, condition.left, condition.right);

                    if res {
                        true_count += 1;
                    }
                }
                if true_count == if_statement.conditions.len() {
                    let expr = Expression::from_body(if_statement.body.clone());
                    let expr_id = expr.expr_id.clone();

                    // Execute the expression block
                    let if_block_return = run_ast(&Mutex::new(expr), stack);

                    if let Some(if_block_return) = if_block_return {
                        return Some(if_block_return);
                    }

                    // Clean the expression definitions from the stack
                    stack.lock().unwrap().drop_ops_from_id(expr_id.clone());
                }
            }

            /*
             * Handle function definitions
             */
            Ops::FnDef => {
                let function = downcast_val::<FnDefinition>(operation.as_self());

                stack
                    .lock()
                    .unwrap()
                    .push_function(get_function_from_def(function));
            }

            /*
             * Handle variables definitions
             */
            Ops::VarDef => {
                let variable = downcast_val::<VarDefinition>(operation.as_self());

                let val_type = variable.assignment.interface;
                let ref_val = variable.assignment.value.clone();

                let var_ref = resolve_ref(val_type, ref_val);

                if let Some(var_ref) = var_ref {
                    // Take a id for the stack
                    let var_id = stack.lock().unwrap().reseve_index();

                    // Push the variable into the stack
                    stack.lock().unwrap().push_variable(VariableDef {
                        name: variable.def_name.clone(),
                        val_type: var_ref.interface,
                        value: var_ref.value,
                        expr_id: ast.expr_id.clone(),
                        functions: get_methods_in_type(var_ref.interface),
                        var_id,
                    });
                }
            }

            /*
             * Handle variable assignments
             */
            Ops::VarAssign => {
                let variable = downcast_val::<VarAssignment>(operation.as_self());

                let is_pointer = variable.var_name.starts_with('&');

                let variable_name = if is_pointer {
                    // Remove & from it's name
                    let mut variable_name = variable.var_name.clone();
                    variable_name.remove(0);
                    variable_name
                } else {
                    variable.var_name.clone()
                };

                let ref_val = resolve_ref(
                    variable.assignment.interface,
                    variable.assignment.value.clone(),
                );

                if let Some(ref_val) = ref_val {
                    stack.lock().unwrap().modify_var(variable_name, ref_val);
                }
            }

            /*
             * Handle function calls
             */
            Ops::FnCall => {
                let fn_call = downcast_val::<FnCall>(operation.as_self());

                let is_referenced = fn_call.reference_to.is_some();

                let function = if is_referenced {
                    let reference_to = fn_call.reference_to.as_ref().unwrap();
                    let variable = stack
                        .lock()
                        .unwrap()
                        .get_variable_by_name(reference_to.as_str());
                    variable
                        .unwrap()
                        .get_function_by_name(fn_call.fn_name.as_str())
                } else {
                    stack
                        .lock()
                        .unwrap()
                        .get_function_by_name(fn_call.fn_name.as_str())
                };

                // If the calling function is found
                if let Some(function) = function {
                    let mut arguments = Vec::new();

                    if is_referenced {
                        let reference_to = fn_call.reference_to.as_ref().unwrap();
                        arguments.push(BoxedValue {
                            interface: Ops::String,
                            value: Box::new(StringVal(reference_to.to_string())),
                        });
                    }

                    for argument in &fn_call.arguments {
                        let arg_ref = resolve_ref(argument.interface, argument.value.clone());
                        if let Some(arg_ref) = arg_ref {
                            arguments.push(arg_ref);
                        } else {
                            // Broken argument
                        }
                    }

                    let res_func = (function.cb)(
                        function.arguments,
                        arguments.clone(),
                        function.body,
                        &stack,
                        &ast,
                    );

                    if let Some(ret_val) = res_func {
                        let val_stringified = value_to_string(ret_val, stack);

                        if let Ok(val_stringified) = val_stringified {
                            // The function returned something that ends up not being used, throw error

                            errors::raise_error(
                                errors::CODES::ReturnedValueNotUsed,
                                vec![
                                    val_stringified,
                                    fn_call.fn_name.clone(),
                                    values_to_strings(arguments, stack).join(" "),
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
    None
}
