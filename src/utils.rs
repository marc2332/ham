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
    pub const RETURN: Val = 17; // ==

    // This must be equal to the latest code
    const CODES_RANGE: Val = 17;

    // If the code is lower than 0 or greater than the defined range above it becomes invalid
    pub fn is_valid(op_code: Val) -> bool {
        if op_code > CODES_RANGE {
            false
        } else {
            true
        }
    }
}

pub mod primitive_values {
    use std::any::Any;

    pub trait PrimitiveValueBase: dyn_clone::DynClone {
        fn as_self(&self) -> &dyn Any;
    }

    dyn_clone::clone_trait_object!(PrimitiveValueBase);

    /*
     * Reference
     *
     * This happens when a variable instead of having an static value,
     * it references to another variable
     *
     */

    #[derive(Clone)]
    pub struct Reference(pub String);

    // Implement base methods for REFERENCE
    impl PrimitiveValueBase for Reference {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for REFERENCE
    pub trait ReferenceValueBase {
        fn new(val: String) -> Reference;
    }

    impl ReferenceValueBase for Reference {
        fn new(val: String) -> Reference {
            Reference(val)
        }
    }

    /*
     * String
     */

    #[derive(Clone)]
    pub struct StringVal(pub String);

    // Implement base methods for String
    impl PrimitiveValueBase for StringVal {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for String
    pub trait StringValueBase {
        fn new(val: String) -> StringVal;
        fn get_state(&self) -> String;
    }

    impl StringValueBase for StringVal {
        fn new(val: String) -> StringVal {
            StringVal(val)
        }

        fn get_state(&self) -> String {
            self.0.clone()
        }
    }

    /*
     * Number
     */

    #[derive(Clone)]
    pub struct Number(pub usize);

    // Implement base methods for Number
    impl PrimitiveValueBase for Number {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for Number
    pub trait NumberValueBase {
        fn new(val: usize) -> Number;
        fn get_state(&self) -> usize;
    }

    impl NumberValueBase for Number {
        fn new(val: usize) -> Number {
            Number(val)
        }

        fn get_state(&self) -> usize {
            self.0
        }
    }

    /*
     * Boolean
     */

    #[derive(Clone)]
    pub struct Boolean(pub bool);

    // Implement base methods for Boolean
    impl PrimitiveValueBase for Boolean {
        fn as_self(&self) -> &dyn Any {
            self
        }
    }

    // Custom methods for Boolean
    pub trait BooleanValueBase {
        fn new(val: bool) -> Boolean;
        fn get_state(&self) -> bool;
    }

    impl BooleanValueBase for Boolean {
        fn new(val: bool) -> Boolean {
            Boolean(val)
        }

        fn get_state(&self) -> bool {
            self.0
        }
    }
}
//get_tokens_in_group_of(starting_token, op_codes::OPEN_PARENT, op_codes::CLOSE_PARENT)
pub mod errors {

    pub type ErrorVal = usize;

    // Function wasn't found in the current scope
    pub const FUNCTION_NOT_FOUND: ErrorVal = 0;

    // Variable wasn't found in the current scope
    pub const VARIABLE_NOT_FOUND: ErrorVal = 1;

    // Unhandled value
    pub const UNHANDLED_VALUE: ErrorVal = 2;

    // Unhandled value type
    pub const UNHANDLED_VALUE_TYPE_CODE: ErrorVal = 3;

    pub fn raise_error(kind: ErrorVal, args: Vec<String>) {
        let msg = match kind {
            FUNCTION_NOT_FOUND => format!("Function <{}> was not found", args[0]),
            VARIABLE_NOT_FOUND => format!("Variable <{}> was not found", args[0]),
            UNHANDLED_VALUE => format!("Value <{}> is not handled", args[0]),
            UNHANDLED_VALUE_TYPE_CODE => {
                format!("Value type by code {} is not handled", args[0])
            }
            _ => String::from("Unhandled error"),
        };

        println!(" \n :: Error :: {}", msg);
    }
}
