pub mod ast_operations {

    /* BASE */
    use crate::utils::{op_codes, primitive_values};
    use std::any::Any;


    pub trait AstBase: dyn_clone::DynClone {
        fn get_type(&self) -> op_codes::Val;
        fn as_self(&self) -> &dyn Any;
    }

    dyn_clone::clone_trait_object!(AstBase);

    /* FUNCTION ARGUMENT */
    #[derive(Clone)]
    pub struct Argument {
        pub val_type: op_codes::Val,
        pub value: String,
    }

    impl Argument {
        pub fn new(value: String) -> Argument {
            let val_type = match value.clone() {
                // Is String
                val if val.chars().nth(0).unwrap() == '"'
                    && val.chars().nth(val.len() - 1).unwrap() == '"' =>
                {
                    op_codes::STRING
                }
                // Is Number
                val if val.as_str().parse::<i32>().is_ok() => op_codes::NUMBER,
                _ => op_codes::REFERENCE,
            };

            Argument {
                val_type,
                value: value.clone(),
            }
        }
    }

    /* FUNCTION DEFINITION */
    pub trait FnDefinitionBase {
        fn get_def_name(&self) -> String;
        fn new(def_name: String, body: Vec<Box<dyn self::AstBase>>) -> Self;
    }

    #[derive(Clone)]
    pub struct FnDefinition {
        pub def_name: String,
        pub body: Vec<Box<dyn self::AstBase>>
    }

    impl FnDefinitionBase for FnDefinition {
        fn get_def_name(&self) -> String {
            return self.def_name.clone();
        }
        fn new(def_name: String, body: Vec<Box<dyn self::AstBase>>) -> FnDefinition {
            FnDefinition {
                def_name,
                body
            }
        }
    }

    impl AstBase for FnDefinition {
        fn get_type(&self) -> i32 {
            op_codes::FN_DEF
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* VARIABLE DEFINITION */
    pub trait VarDefinitionBase {
        fn get_def_name(&self) -> String;
        fn new(def_name: String, assignment: Assignment) -> Self;
    }

    #[derive(Clone)]
    pub struct VarDefinition {
        pub def_name: String,
        pub assignment: Assignment,
    }

    impl VarDefinitionBase for VarDefinition {
        fn get_def_name(&self) -> String {
            return self.def_name.clone();
        }
        fn new(def_name: String, assignment: Assignment) -> VarDefinition {
            VarDefinition {
                def_name,
                assignment,
            }
        }
    }

    impl AstBase for VarDefinition {
        fn get_type(&self) -> i32 {
            op_codes::VAR_DEF
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* VARIABLE ASSIGNMENT */
    pub trait VarAssignmentBase {
        fn get_def_name(&self) -> String;
        fn new(var_name: String, assignment: Assignment) -> Self;
    }

    #[derive(Clone)]
    pub struct VarAssignment {
        pub var_name: String,
        pub assignment: Assignment,
    }

    impl VarAssignmentBase for VarAssignment {
        fn get_def_name(&self) -> String {
            return self.var_name.clone();
        }
        fn new(var_name: String, assignment: Assignment) -> VarAssignment {
            VarAssignment {
                var_name,
                assignment,
            }
        }
    }

    impl AstBase for VarAssignment {
        fn get_type(&self) -> i32 {
            op_codes::VAR_ASSIGN
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* ASSIGNMENT */

    #[derive(Clone)]
    pub struct Assignment {
        pub interface: op_codes::Val,
        pub value: Box<dyn primitive_values::PrimitiveValueBase>,
    }

    impl AstBase for Assignment {
        fn get_type(&self) -> i32 {
            op_codes::LEFT_ASSIGN
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* EXPRESSION  */

    #[derive(Clone)]
    pub struct Expression {
        pub body: Vec<Box<dyn self::AstBase>>,
        pub token_type: op_codes::Val,
    }

    impl AstBase for Expression {
        fn get_type(&self) -> i32 {
            op_codes::EXPRESSION
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
        fn new() -> Expression {
            Expression {
                token_type: op_codes::EXPRESSION,
                body: Vec::new(),
            }
        }
        fn from_body(body: Vec<Box<dyn self::AstBase>>) -> Expression {
            Expression {
                token_type: op_codes::EXPRESSION,
                body,
            }
        }
    }

    /* FUNCTION CALL  */

    #[derive(Clone)]
    pub struct FnCall {
        pub token_type: op_codes::Val,
        pub fn_name: String,
        pub arguments: Vec<Argument>,
    }

    impl AstBase for FnCall {
        fn get_type(&self) -> i32 {
            op_codes::FN_CALL
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    pub trait FnCallBase {
        fn new(fn_name: String) -> Self;
    }

    impl FnCallBase for FnCall {
        fn new(fn_name: String) -> FnCall {
            FnCall {
                token_type: op_codes::FN_CALL,
                fn_name,
                arguments: Vec::new(),
            }
        }
    }
}
