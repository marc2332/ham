pub mod op_codes {

    #[derive(Clone)]
    pub enum Directions {
        LeftToRight,
        RightToLeft,
    }

    pub type Val = usize;

    /*
     * Each code identifies a different type of token / operation
     */

    pub const REFERENCE: Val = 0;
    pub const VAR_DEF: Val = 1;
    pub const LEFT_ASSIGN: Val = 2;
    pub const EXPRESSION: Val = 3;
    pub const FN_CALL: Val = 4;
    pub const OPEN_PARENT: Val = 5;
    pub const CLOSE_PARENT: Val = 6;
    pub const BOOLEAN: Val = 7;
    pub const NUMBER: Val = 8;
    pub const STRING: Val = 9;
    pub const VAR_ASSIGN: Val = 10;
    pub const FN_DEF: Val = 11;
    pub const OPEN_BLOCK: Val = 12;
    pub const CLOSE_BLOCK: Val = 13;
    pub const IF_CONDITIONAL: Val = 14;
    pub const RES_EXPRESSION: Val = 15;
    pub const EQUAL_CONDITION: Val = 16; // ==
    pub const RETURN: Val = 17; // return
    pub const PROP_ACCESS: Val = 18; // .
    pub const COMMA_DELIMITER: Val = 19; //,
    pub const WHILE_DEF: Val = 20;
    pub const NOT_EQUAL_CONDITION: Val = 21; // !=
    pub const POINTER: Val = 22;
}

pub mod primitive_values {
    use erased_serde::serialize_trait_object;
    use serde::Serialize;
    use std::any::Any;

    pub trait PrimitiveValueBase:
        dyn_clone::DynClone + erased_serde::Serialize + std::fmt::Debug
    {
        fn as_self(&self) -> &dyn Any;
    }

    dyn_clone::clone_trait_object!(PrimitiveValueBase);
    serialize_trait_object!(PrimitiveValueBase);

    /*
     * Pointer
     */

    #[derive(Clone, Debug, Serialize)]
    pub struct Pointer(pub u64);

    // Implement base methods for Pointer
    impl PrimitiveValueBase for Pointer {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for String
    pub trait PointerBase {
        fn get_state(&self) -> u64;
    }

    impl PointerBase for Pointer {
        fn get_state(&self) -> u64 {
            self.0
        }
    }

    /*
     * String
     */

    #[derive(Clone, Debug, Serialize)]
    pub struct StringVal(pub String);

    // Implement base methods for String
    impl PrimitiveValueBase for StringVal {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for String
    pub trait StringValueBase {
        fn new(val: String) -> Self;
        fn get_state(&self) -> String;
    }

    impl StringValueBase for StringVal {
        fn new(val: String) -> Self {
            Self(val)
        }

        fn get_state(&self) -> String {
            self.0.clone()
        }
    }

    /*
     * Number
     */

    #[derive(Clone, Debug, Serialize)]
    pub struct Number(pub usize);

    // Implement base methods for Number
    impl PrimitiveValueBase for Number {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for Number
    pub trait NumberValueBase {
        fn new(val: usize) -> Self;
        fn get_state(&self) -> usize;
    }

    impl NumberValueBase for Number {
        fn new(val: usize) -> Self {
            Number(val)
        }

        fn get_state(&self) -> usize {
            self.0
        }
    }

    /*
     * Boolean
     */

    #[derive(Clone, Debug, Serialize)]
    pub struct Boolean(pub bool);

    // Implement base methods for Boolean
    impl PrimitiveValueBase for Boolean {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for Boolean
    pub trait BooleanValueBase {
        fn new(val: bool) -> Self;
        fn get_state(&self) -> bool;
    }

    impl BooleanValueBase for Boolean {
        fn new(val: bool) -> Self {
            Self(val)
        }

        fn get_state(&self) -> bool {
            self.0
        }
    }
}

pub mod errors {

    use colored::*;

    pub type ErrorVal = usize;

    // Function wasn't found in the current scope
    pub const FUNCTION_NOT_FOUND: ErrorVal = 0;

    // Variable wasn't found in the current scope
    pub const VARIABLE_NOT_FOUND: ErrorVal = 1;

    // Not used returned value
    pub const RETURNED_VALUE_NOT_USED: ErrorVal = 2;

    // Pointer points to an invalid reference
    pub const BROKEN_POINTER: ErrorVal = 3;

    pub fn raise_error(kind: ErrorVal, args: Vec<String>) {
        let msg = match kind {
            FUNCTION_NOT_FOUND => format!("Function '{}' was not found", args[0]),
            VARIABLE_NOT_FOUND => format!("Variable '{}' was not found", args[0].blue()),
            RETURNED_VALUE_NOT_USED => {
                format!(
                    "Returned value '{}' by function '{}' is not used\n
    let value = {}({});
    ¯¯¯¯¯¯¯¯¯
    ↑ Help: Assign the return value to a variable. ",
                    args[0].blue(),
                    args[1].blue(),
                    args[1].blue(),
                    args[2]
                )
            }
            BROKEN_POINTER => {
                format!(
                    "Pointer points to variable by id '{}' which does no longer exist.",
                    args[0].blue()
                )
            }
            _ => String::from("Unhandled error"),
        };

        println!("{}: {}", "Error".red(), msg);
    }
}
