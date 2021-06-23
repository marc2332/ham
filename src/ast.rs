pub mod ast_operations {

    /* BASE */
    use crate::utils::primitive_values::PrimitiveValueBase;
    use crate::utils::{op_codes, primitive_values};

    use erased_serde::serialize_trait_object;
    use serde::Serialize;

    use std::any::Any;
    use uuid::Uuid;

    pub trait AstBase: dyn_clone::DynClone + erased_serde::Serialize + std::fmt::Debug {
        fn get_type(&self) -> op_codes::Val;
        fn as_self(&self) -> &dyn Any;
    }

    dyn_clone::clone_trait_object!(AstBase);
    serialize_trait_object!(AstBase);

    /* RETURN STATEMENT */

    #[derive(Clone, Debug, Serialize)]
    pub struct ReturnStatement {
        pub value: BoxedValue,
    }

    impl AstBase for ReturnStatement {
        fn get_type(&self) -> usize {
            op_codes::RETURN
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* RESULT EXPRESSION */
    pub trait ResultExpressionBase {
        fn new(relation: op_codes::Val, left: BoxedValue, right: BoxedValue) -> Self;
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct ResultExpression {
        pub left: BoxedValue,
        pub relation: op_codes::Val,
        pub right: BoxedValue,
    }

    impl ResultExpressionBase for ResultExpression {
        fn new(relation: op_codes::Val, left: BoxedValue, right: BoxedValue) -> ResultExpression {
            ResultExpression {
                left,
                relation,
                right,
            }
        }
    }

    impl AstBase for ResultExpression {
        fn get_type(&self) -> usize {
            op_codes::RES_EXPRESSION
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
        fn new(
            conditions: Vec<ResultExpression>,
            body: Vec<Box<dyn self::AstBase>>,
        ) -> IfConditional {
            IfConditional { conditions, body }
        }
    }

    impl AstBase for IfConditional {
        fn get_type(&self) -> usize {
            op_codes::IF_CONDITIONAL
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* FUNCTION DEFINITION */
    pub trait FnDefinitionBase {
        fn get_def_name(&self) -> String;
        fn new(def_name: String, body: Vec<Box<dyn self::AstBase>>, arguments: Vec<String>)
            -> Self;
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct FnDefinition {
        pub def_name: String,
        pub body: Vec<Box<dyn self::AstBase>>,
        pub arguments: Vec<String>,
    }

    impl FnDefinitionBase for FnDefinition {
        fn get_def_name(&self) -> String {
            return self.def_name.clone();
        }
        fn new(
            def_name: String,
            body: Vec<Box<dyn self::AstBase>>,
            arguments: Vec<String>,
        ) -> FnDefinition {
            FnDefinition {
                def_name,
                body,
                arguments,
            }
        }
    }

    impl AstBase for FnDefinition {
        fn get_type(&self) -> usize {
            op_codes::FN_DEF
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
            return self.def_name.clone();
        }
        fn new(def_name: String, assignment: BoxedValue) -> VarDefinition {
            VarDefinition {
                def_name,
                assignment,
            }
        }
    }

    impl AstBase for VarDefinition {
        fn get_type(&self) -> usize {
            op_codes::VAR_DEF
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
            return self.var_name.clone();
        }
        fn new(var_name: String, assignment: BoxedValue) -> VarAssignment {
            VarAssignment {
                var_name,
                assignment,
            }
        }
    }

    impl AstBase for VarAssignment {
        fn get_type(&self) -> usize {
            op_codes::VAR_ASSIGN
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    /* BOXED VALUE */

    #[derive(Clone, Debug, Serialize)]
    pub struct BoxedValue {
        pub interface: op_codes::Val,
        pub value: Box<dyn primitive_values::PrimitiveValueBase>,
    }

    impl AstBase for BoxedValue {
        fn get_type(&self) -> usize {
            op_codes::LEFT_ASSIGN
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
        pub token_type: op_codes::Val,
        pub expr_id: String,
    }

    impl AstBase for Expression {
        fn get_type(&self) -> usize {
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
                expr_id: Uuid::new_v4().to_string(),
            }
        }
        fn from_body(body: Vec<Box<dyn self::AstBase>>) -> Expression {
            Expression {
                token_type: op_codes::EXPRESSION,
                body,
                expr_id: Uuid::new_v4().to_string(),
            }
        }
    }

    /* FUNCTION CALL  */

    #[derive(Clone, Serialize, Debug)]
    pub struct FnCall {
        pub token_type: op_codes::Val,
        pub fn_name: String,
        pub arguments: Vec<BoxedValue>,
        /*
         * TODO: Use Option
         */
        pub reference_to: String,
    }

    impl AstBase for FnCall {
        fn get_type(&self) -> usize {
            op_codes::FN_CALL
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    pub trait FnCallBase {
        fn new(fn_name: String, reference_to: String) -> Self;
    }

    impl FnCallBase for FnCall {
        fn new(fn_name: String, reference_to: String) -> FnCall {
            FnCall {
                token_type: op_codes::FN_CALL,
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
        fn get_type(&self) -> usize {
            op_codes::WHILE_DEF
        }
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    pub trait WhileBase {
        fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> Self;
    }

    impl WhileBase for While {
        fn new(conditions: Vec<ResultExpression>, body: Vec<Box<dyn self::AstBase>>) -> While {
            While { conditions, body }
        }
    }
}
