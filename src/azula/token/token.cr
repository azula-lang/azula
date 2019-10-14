module Azula

    enum TokenType
        # Illegal is used for any undefined tokens
        ILLEGAL
        # Signifies the end of a file
        EOF

        # Identifier of type eg. int, string
        TYPE
        # Used to assign a variable name to value
        ASSIGN
        # Used to indicate a return type
        COLON
        # Used to indicate end of a line
        SEMICOLON
        # Used for floats and accessing structs
        DOT
        # Used for variable/function names
        IDENTIFIER
        # Used to initialise a function
        FUNCTION
        # Used to return a value from a function
        RETURN
        # Used to separate terms
        COMMA
        # Used for string literals
        STRING
        # Used for number literals
        NUMBER
        # Used to cast one type to another
        AS
        # Used to create a struct
        STRUCT
        # Used to declare an external function
        EXTERN

        # True
        TRUE
        # False
        FALSE
        # Null
        NULL

        # +
        PLUS
        # -
        MINUS
        # *
        ASTERISK
        # **
        EXPONENT
        # /
        SLASH
        # %
        MODULO
        # Ampersand
        AMPERSAND

        # Equality
        EQ
        # Inequality
        NOT_EQ
        # Less than
        LT
        # Greater than
        GT
        # Less than or equal
        LT_EQ
        # Greater than or equal
        GT_EQ

        # Logical OR
        OR
        # Logical AND
        AND
        # Logical NOT
        NOT

        # Conditionals
        IF
        ELSEIF
        ELSE

        # Switch statement
        SWITCH
        # Default statement in switch
        DEFAULT
        # Continue
        CONTINUE

        # While loop
        WHILE

        # (
        LBRACKET
        # )
        RBRACKET
        # {
        LBRACE
        # }
        RBRACE
        # [
        LSQUARE
        # ]
        RSQUARE

    end

    # Token represents a syntax token used to generate the AST
    class Token

        def initialize(@type : TokenType, @literal : String, @file : String, @linenumber : Int32, @charnumber : Int32)
        end

        # Get the string representation of this Token
        def to_string : String
            return "Token #{@type} (#{@literal}) in #{@file} line #{@linenumber}, character #{@charnumber}"
        end

        # Get the `TokenType` of this Token
        getter type
        # Get the literal string of this Token
        getter literal
        # Get which file this Token is in
        getter file
        # Get the line number this Token is on
        getter linenumber
        # Get the character position of this token in the line
        getter charnumber

    end

    # Keyword string to its TokenType
    Keywords = {
        "int" => TokenType::TYPE,
        "int8" => TokenType::TYPE,
        "int16" => TokenType::TYPE,
        "bool" => TokenType::TYPE,
        "string" => TokenType::TYPE,
        "float" => TokenType::TYPE,
        "error" => TokenType::TYPE,
        "void" => TokenType::TYPE,

        "func" => TokenType::FUNCTION,
        "return" => TokenType::RETURN,
        "continue" => TokenType::CONTINUE,

        "extern" => TokenType::EXTERN,
        
        "as" => TokenType::AS,

        "struct" => TokenType::STRUCT,

        "true" => TokenType::TRUE,
        "false" => TokenType::FALSE,
        "null" => TokenType::NULL,
        "or" => TokenType::OR,
        "and" => TokenType::AND,

        "if" => TokenType::IF,
        "elseif" => TokenType::ELSEIF,
        "else" => TokenType::ELSE,

        "switch" => TokenType::SWITCH,
        "default" => TokenType::DEFAULT,

        "while" => TokenType::WHILE,
    }
    
end