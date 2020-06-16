class Azula::Token
    getter type, literal, file, line_number, char_number
    def initialize(@type : TokenType, @literal : String, @file : String, @line_number : Int32, @char_number : Int32)
    end
end

enum Azula::TokenType
    # Illegal is used for any undefined tokens
    Illegal
    # Signifies the end of a file
    EOF

    # Identifier of type eg. int, string
    Type
    # Used to assign a variable name to value
    Assign
    # Used to indicate a return type
    Colon
    # Used to indicate end of a line
    Semicolon
    # Used for floats and accessing structs
    Dot
    # Used for variable/function names
    Identifier
    # Used to initialise a function
    Function
    # Used to return a value from a function
    Return
    # Used to separate terms
    Comma
    # Used for string literals
    StringLiteral
    # Used for number literals
    NumberLiteral
    # Used to create a struct
    Struct

    # Used to infer the type of a variable
    Var

    # True
    True
    # False
    False

    # +
    Plus
    # -
    Minus
    # *
    Asterisk
    # **
    Exponent
    # /
    Slash
    # %
    Modulo
    # &
    Ampersand
    # |
    Pipe
    # !
    Bang
    # ?
    Question
    # <<
    ShiftLeft
    # >>
    ShiftRight

    # Equality
    Eq
    # Inequality
    NotEq
    # Less than
    Lt
    # Greater than
    Gt
    # Less than or equal
    LtEq
    # Greater than or equal
    GtEq
    # Or
    Or
    # And
    And

    # Conditionals
    If
    Else

    # Continue
    Continue

    # Import statement
    Import
    # Module declaration
    Module

    # (
    LBracket
    # )
    RBracket
    # {
    LBrace
    # }
    RBrace
    # [
    LSquare
    # ]
    RSquare
end

# Reserved words in Azula
Azula::Keywords = {
    "int1" => TokenType::Type,
    "int8" => TokenType::Type,
    "int16" => TokenType::Type,
    "int" => TokenType::Type,
    "int32" => TokenType::Type,
    "int64" => TokenType::Type,

    "bool" => TokenType::Type,
    "string" => TokenType::Type,
    "float" => TokenType::Type,
    "float32" => TokenType::Type,
    "float64" => TokenType::Type,
    "void" => TokenType::Type,
    "error" => TokenType::Type,
    "array" => TokenType::Type,

    "var" => TokenType::Var,

    "func" => TokenType::Function,
    "return" => TokenType::Return,
    "continue" => TokenType::Continue,

    "struct" => TokenType::Struct,

    "import" => TokenType::Import,
    "module" => TokenType::Module,

    "true" => TokenType::True,
    "false" => TokenType::False,

    "if" => TokenType::If,
    "else" => TokenType::Else,
}