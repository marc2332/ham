use crate::ast::ast_operations;
use crate::ast::ast_operations::{
    AstBase, BoxedValue, ExpressionBase, FnCallBase, FnDefinitionBase, IfConditionalBase,
    ResultExpressionBase, VarAssignmentBase, VarDefinitionBase,
};
use crate::tokenizer::{LinesList, Token, TokensList};
use crate::utils::primitive_values::{
    BooleanValueBase, NumberValueBase, PrimitiveValueBase, ReferenceValueBase, StringValueBase,
};
use crate::utils::{errors, op_codes, primitive_values};

use regex::Regex;
use std::any::Any;
use std::sync::{Mutex, MutexGuard};

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

fn get_lines(code: String) -> LinesList {
    let mut lines = Vec::new();

    // Every line
    for line in code.split("\n") {
        let line = String::from(line);
        let mut line_ast = Vec::new();

        let re = Regex::new(r"[\s+,:]|([()])").unwrap();

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

pub fn get_tokens(code: String) -> TokensList {
    let lines = self::get_lines(code);
    return self::transform_into_tokens(lines);
}

pub fn move_tokens_into_ast(tokens: TokensList, ast_tree: &Mutex<ast_operations::Expression>) {
    let mut ast_tree = ast_tree.lock().unwrap();

    // Get tokens with index starting on `from` until a token matches its type to `to`
    fn get_tokens_from_to_fn(
        from: usize,
        to: op_codes::Val,
        tokens: TokensList,
    ) -> Vec<(usize, Token)> {
        let mut found_tokens = Vec::new();

        let mut tok_n = from;

        while tok_n < tokens.len() {
            if tokens[tok_n].ast_type == to {
                break;
            } else {
                found_tokens.push((tok_n, tokens[tok_n].clone()))
            }
            tok_n += 1;
        }

        found_tokens
    }

    let get_tokens_from_to = |from: usize, to: op_codes::Val| -> Vec<(usize, Token)> {
        return get_tokens_from_to_fn(from, to, tokens.clone());
    };

    // Get all the tokens in a group (expression blocks, arguments)
    let get_tokens_in_group_of =
        |from: usize, open_tok: op_codes::Val, close_tok: op_codes::Val| -> TokensList {
            let mut found_tokens = Vec::new();

            let mut count = 0;

            let mut tok_n = from;

            while tok_n < tokens.len() {
                let token = tokens[tok_n].clone();

                if token.ast_type == open_tok {
                    count += 1;
                } else if token.ast_type == close_tok {
                    count -= 1;
                }

                if count == 0 {
                    break;
                } else if tok_n > from {
                    found_tokens.push(token.clone());
                }
                tok_n += 1;
            }

            found_tokens
        };

    let mut token_n = 0;

    fn get_assignment_token_fn(
        val: String,
        tok_n: usize,
        tokens: TokensList,
    ) -> ast_operations::BoxedValue {
        match val.as_str() {
            // True boolean
            "true" => ast_operations::BoxedValue {
                interface: op_codes::BOOLEAN,
                value: Box::new(primitive_values::Boolean::new(true)),
            },
            // False boolean
            "false" => ast_operations::BoxedValue {
                interface: op_codes::BOOLEAN,
                value: Box::new(primitive_values::Boolean::new(false)),
            },
            // Numeric values
            val if val.parse::<usize>().is_ok() => ast_operations::BoxedValue {
                interface: op_codes::NUMBER,
                value: Box::new(primitive_values::Number::new(val.parse::<usize>().unwrap())),
            },
            // String values
            val if val.chars().nth(0).unwrap() == '"'
                && val.chars().nth(val.len() - 1).unwrap() == '"' =>
            {
                ast_operations::BoxedValue {
                    interface: op_codes::STRING,
                    value: Box::new(primitive_values::StringVal::new(String::from(val))),
                }
            }
            // References to other values (ej: referencing to a variable)
            val => {
                if tok_n < tokens.len() - 1 {
                    let next_token = tokens[tok_n + 1].clone();

                    let reference_type = match next_token.ast_type {
                        op_codes::OPEN_PARENT => op_codes::FN_CALL,
                        _ => 0,
                    };

                    match reference_type {
                        op_codes::FN_CALL => {
                            let mut ast_token = ast_operations::FnCall::new(String::from(val));

                            // Ignore itself and the (
                            let starting_token = tok_n + 2;

                            let arguments_tokens: Vec<(usize, Token)> = get_tokens_from_to_fn(
                                starting_token,
                                op_codes::CLOSE_PARENT,
                                tokens.clone(),
                            );

                            let arguments = convert_tokens_into_arguments(
                                arguments_tokens
                                    .clone()
                                    .iter()
                                    .map(|(_, token)| token.clone())
                                    .collect(),
                            );

                            ast_token.arguments = arguments;

                            ast_operations::BoxedValue {
                                interface: op_codes::FN_CALL,
                                value: Box::new(ast_token.clone()),
                            }
                        }
                        _ => ast_operations::BoxedValue {
                            interface: op_codes::REFERENCE,
                            value: Box::new(primitive_values::Reference::new(String::from(val))),
                        },
                    }
                } else {
                    ast_operations::BoxedValue {
                        interface: op_codes::REFERENCE,
                        value: Box::new(primitive_values::Reference::new(String::from(val))),
                    }
                }
            }
        }
    }

    let get_assignment_token = |val: String, tok_n: usize| -> ast_operations::BoxedValue {
        return get_assignment_token_fn(val, tok_n, tokens.clone());
    };

    fn convert_tokens_into_arguments(tokens: TokensList) -> Vec<ast_operations::BoxedValue> {
        let mut args = Vec::new();

        let mut tok_n = 0;

        while tok_n < tokens.len() {
            let token = tokens[tok_n].clone();

            match token.ast_type {
                // Ignore ( and )
                op_codes::OPEN_PARENT => tok_n += 1,
                op_codes::CLOSE_PARENT => tok_n += 1,
                _ => {
                    let arguments_tokens: Vec<(usize, Token)> =
                        get_tokens_from_to_fn(tok_n, op_codes::CLOSE_PARENT, tokens.clone());

                    args.push(get_assignment_token_fn(
                        token.value.clone(),
                        tok_n,
                        tokens.clone(),
                    ));

                    tok_n += arguments_tokens.len() + 1;
                }
            }
        }

        args
    }

    fn convert_tokens_into_res_expressions(
        tokens: TokensList,
    ) -> Vec<ast_operations::ResultExpression> {
        let mut exprs = Vec::new();

        let left_token = tokens[0].clone();

        let mut tok_n = 1;

        while tok_n < tokens.len() {
            let token = tokens[tok_n].clone();

            match token.ast_type {
                op_codes::EQUAL_CONDITION => {
                    let right_token = tokens[tok_n + 1].clone();

                    exprs.push(ast_operations::ResultExpression::new(
                        op_codes::EQUAL_CONDITION,
                        ast_operations::Argument::new(left_token.value.clone()),
                        ast_operations::Argument::new(right_token.value.clone()),
                    ));

                    tok_n += 2;
                }
                _ => panic!("UNHANDLED CONDITION"),
            }
        }

        exprs
    }

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
                        let mut ast_token =
                            ast_operations::FnCall::new(current_token.value.clone());

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

#[derive(Clone)]
struct VariableDef {
    name: String,
    val_type: op_codes::Val,
    value: Box<dyn primitive_values::PrimitiveValueBase>,
    expr_id: String,
}

#[derive(Clone)]
struct FunctionDef {
    name: String,
    body: Vec<Box<dyn AstBase>>,
    cb: fn(
        args: Vec<String>,
        args_vals: Vec<String>,
        body: Vec<Box<dyn AstBase>>,
        stack: &Mutex<Stack>,
        ast: &MutexGuard<ast_operations::Expression>,
    ) -> Result<ast_operations::BoxedValue, ()>,
    expr_id: String,
    arguments: Vec<String>,
}

#[derive(Clone)]
pub struct Stack {
    functions: Vec<FunctionDef>,
    variables: Vec<VariableDef>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            functions: Vec::new(),
            variables: Vec::new(),
        }
    }
    pub fn drop_ops_from_id(&mut self, id: String) {
        for (index, op_var) in self.variables.clone().iter().enumerate() {
            if op_var.expr_id == id {
                self.variables.remove(index);
            }
        }
    }
}

pub fn run_ast(
    ast: &Mutex<ast_operations::Expression>,
    stack: &Mutex<Stack>,
) -> Result<ast_operations::BoxedValue, ()> {
    let ast = ast.lock().unwrap();

    // Search variables in the stack by its name
    let get_var_reference = |var_name: String| -> Result<VariableDef, ()> {
        let stack = stack.lock().unwrap();

        for op_var in &stack.variables {
            if op_var.name == var_name {
                return Ok(op_var.clone());
            }
        }

        errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
        Err(())
    };

    // Search functions in the stack by its name
    let get_fn = |fn_name: String| -> Result<FunctionDef, ()> {
        let stack = stack.lock().unwrap();
        for op_fn in &stack.functions {
            if op_fn.name == fn_name {
                return Ok(op_fn.clone());
            }
        }

        errors::raise_error(errors::FUNCTION_NOT_FOUND, vec![fn_name.clone()]);
        Err(())
    };

    // Resolve values
    let resolve_val = |val_type: op_codes::Val, value: String| -> Result<String, ()> {
        let res: String = match val_type {
            // If the value is type String, Number or boolean then return it self
            op_codes::STRING => value,
            op_codes::BOOLEAN => value,
            op_codes::NUMBER => value,

            // If the value is a reference to a variable then returns the variable's current value
            op_codes::REFERENCE => {
                let ref_name = value.clone();
                let ref_value = get_var_reference(ref_name.clone());

                let mut val = String::from("");
                if ref_value.is_ok() {
                    let ref_value = ref_value.unwrap();
                    val = match ref_value.val_type {
                        op_codes::BOOLEAN => {
                            downcast_val::<primitive_values::Boolean>(ref_value.value.as_self())
                                .0
                                .to_string()
                        }
                        op_codes::STRING => {
                            downcast_val::<primitive_values::StringVal>(ref_value.value.as_self())
                                .0
                                .clone()
                        }
                        op_codes::NUMBER => {
                            downcast_val::<primitive_values::Number>(ref_value.value.as_self())
                                .0
                                .to_string()
                        }
                        _ => String::from(""),
                    }
                }
                val
            }
            _ => String::from(""),
        };

        Ok(res)
    };

    fn downcast_val<T: 'static>(val: &dyn Any) -> &T {
        val.downcast_ref::<T>().unwrap()
    }

    /*
     * For static values it will just return the input but for references it will resolve its value
     * and then return it
     */
    let resolve_def = |val_type: op_codes::Val,
                       ref_val: Box<dyn primitive_values::PrimitiveValueBase>|
     -> (op_codes::Val, Box<dyn primitive_values::PrimitiveValueBase>) {
        match val_type {
            op_codes::STRING => (val_type, ref_val),
            op_codes::BOOLEAN => (val_type, ref_val),
            op_codes::NUMBER => (val_type, ref_val),
            op_codes::REFERENCE => {
                let val = downcast_val::<primitive_values::Reference>(ref_val.as_self())
                    .0
                    .clone();

                let ref_def = get_var_reference(val);

                if ref_def.is_ok() {
                    let ref_assign = ref_def.unwrap();
                    (ref_assign.val_type, ref_assign.value)
                } else {
                    (val_type, ref_val)
                }
            }
            op_codes::FN_CALL => {
                let fn_call = downcast_val::<ast_operations::FnCall>(ref_val.as_self());
                let ref_fn = get_fn(fn_call.fn_name.clone());

                // If the calling function is found
                if ref_fn.is_ok() {
                    let mut arguments = Vec::new();

                    for argument in &fn_call.arguments {
                        // WIP
                        let val =
                            downcast_val::<primitive_values::StringVal>(argument.value.as_self());
                        let ref_value = resolve_val(argument.interface, val.0.clone());

                        if ref_value.is_ok() {
                            arguments.push(ref_value.unwrap().to_string());
                        }
                    }

                    let func = ref_fn.unwrap();

                    let return_val = (func.cb)(func.arguments, arguments, func.body, &stack, &ast);

                    // WIP

                    if return_val.is_ok() {
                        let boxed_val = return_val.unwrap();
                        (boxed_val.interface, boxed_val.value)
                    } else {
                        (0, Box::new(primitive_values::Number(0)))
                    }
                } else {
                    (0, Box::new(primitive_values::Number(0)))
                }
            }
            _ => (0, Box::new(primitive_values::Number(0))),
        }
    };

    // Modify a variable
    let modify_var = |var_name: String, value: Box<dyn primitive_values::PrimitiveValueBase>| {
        let mut stack = stack.lock().unwrap();

        for mut op_var in stack.variables.iter_mut() {
            if op_var.name == var_name {
                op_var.value = value.clone();
                return ();
            }
        }

        errors::raise_error(errors::VARIABLE_NOT_FOUND, vec![var_name.clone()]);
    };

    // Check if a conditional is true or not
    let eval_condition = |condition_code: op_codes::Val,
                          left_val: ast_operations::Argument,
                          right_val: ast_operations::Argument|
     -> bool {
        let left_val = resolve_val(left_val.val_type.clone(), left_val.value.clone());
        let right_val = resolve_val(right_val.val_type.clone(), right_val.value.clone());

        if left_val.is_ok() && right_val.is_ok() {
            match condition_code {
                op_codes::EQUAL_CONDITION => {
                    let left_val = left_val.unwrap();
                    let right_val = right_val.unwrap();

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

    fn stringify_arg(arg: (usize, Box<dyn PrimitiveValueBase>)) -> String {
        match arg.0 {
            op_codes::BOOLEAN => downcast_val::<primitive_values::Boolean>(arg.1.as_self())
                .0
                .to_string(),
            op_codes::STRING => downcast_val::<primitive_values::StringVal>(arg.1.as_self())
                .0
                .clone(),
            op_codes::NUMBER => downcast_val::<primitive_values::Number>(arg.1.as_self())
                .0
                .to_string(),
            _ => String::from(""),
        }
    }

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

                let (ref_type, ref_val) = resolve_def(ret_type, ret_val);

                return Ok(BoxedValue {
                    interface: ref_type,
                    value: ref_val.clone(),
                });
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

                let (ref_type, ref_val) = resolve_def(val_type, ref_val);

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
                    });
                }
            }

            /*
             * Handle variable assignments
             */
            op_codes::VAR_ASSIGN => {
                let variable = downcast_val::<ast_operations::VarAssignment>(op.as_self());

                modify_var(variable.var_name.clone(), variable.assignment.value.clone());
            }

            /*
             * Handle function calls
             */
            op_codes::FN_CALL => {
                let fn_call = downcast_val::<ast_operations::FnCall>(op.as_self());
                let ref_fn = get_fn(fn_call.fn_name.clone());

                // If the calling function is found
                if ref_fn.is_ok() {
                    let mut arguments = Vec::new();

                    for argument in &fn_call.arguments {
                        let arg_ref = resolve_def(argument.interface, argument.value.clone());

                        let arg_val = stringify_arg(arg_ref.clone());

                        arguments.push(arg_val.clone());
                    }

                    let func = ref_fn.unwrap();

                    let _ = (func.cb)(func.arguments, arguments, func.body, &stack, &ast);
                }
            }
            _ => {
                println!("IDK")
            }
        }
    }
    Err(())
}
