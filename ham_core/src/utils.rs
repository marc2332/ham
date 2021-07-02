use serde::Serialize;

#[derive(Clone)]
pub enum Directions {
    LeftToRight,
    RightToLeft,
}

#[derive(Clone, Debug, Serialize, PartialEq, Copy)]
pub enum Ops {
    Invalid,
    Reference,
    VarDef,
    LeftAssign,
    Expression,
    FnCall,
    OpenParent,
    CloseParent,
    Boolean,
    Number,
    String,
    VarAssign,
    FnDef,
    OpenBlock,
    CloseBlock,
    IfConditional,
    ResExpression,
    EqualCondition,
    Return,
    PropAccess,
    CommaDelimiter,
    WhileDef,
    NotEqualCondition,
    Pointer,
    Import,
    Module,
    FromModule,
    Break,
}

pub mod errors {

    use colored::*;

    pub enum CODES {
        // Function wasn't found in the current scope
        FunctionNotFound,

        // Variable wasn't found in the current scope
        VariableNotFound,

        // Not used returned value
        ReturnedValueNotUsed,

        // Pointer points to an invalid reference
        BrokenPointer,

        // Module is not found (ex, file's path is not correct)
        ModuleNotFound,

        // Got a keyword instead of another one
        UnexpectedKeyword,
    }

    pub fn raise_error(kind: CODES, args: Vec<String>) {
        let msg = match kind {
            CODES::FunctionNotFound => format!("Function '{}' was not found", args[0]),
            CODES::VariableNotFound => format!("Variable '{}' was not found", args[0].blue()),
            CODES::ReturnedValueNotUsed => {
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
            CODES::BrokenPointer => {
                format!(
                    "Pointer points to variable by id '{}' which does no longer exist.",
                    args[0].blue()
                )
            }
            CODES::ModuleNotFound => {
                format!("There is no module in path '{}'", args[0].blue())
            }
            CODES::UnexpectedKeyword => {
                format!("Unexpected keyword '{}'", args[0].blue())
            }
        };

        println!("{}: {}", "Error".red(), msg);
    }
}
