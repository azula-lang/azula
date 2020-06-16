require "../ast/*"
require "../error"
require "./token"
require "../type"

# Add a prefix handler
macro register_prefix(token_type, method_name)
    @prefix_funcs[TokenType::{{token_type}}] = ->{ self.{{method_name}}.as(AST::Node?)}
end

# Add an infix handler
macro register_infix(token_type, method_name)
    @infix_funcs[TokenType::{{token_type}}] = ->(exp: AST::Node){ self.{{method_name}}(exp).as(AST::Node?)}
end

enum OperatorPrecedence
    LOWEST
    COMPARISON
    EQUALS
    LESS_GREATER
    SUM
    PRODUCT
    PREFIX
    CALL
    ACCESS
end

Precedences = {
    Azula::TokenType::Or => OperatorPrecedence::COMPARISON,
    Azula::TokenType::And => OperatorPrecedence::COMPARISON,
    Azula::TokenType::Eq => OperatorPrecedence::EQUALS,
    Azula::TokenType::NotEq => OperatorPrecedence::EQUALS,
    Azula::TokenType::Lt => OperatorPrecedence::LESS_GREATER,
    Azula::TokenType::LtEq => OperatorPrecedence::LESS_GREATER,
    Azula::TokenType::Gt => OperatorPrecedence::LESS_GREATER,
    Azula::TokenType::GtEq => OperatorPrecedence::LESS_GREATER,
    Azula::TokenType::Plus => OperatorPrecedence::SUM,
    Azula::TokenType::Minus => OperatorPrecedence::SUM,
    Azula::TokenType::ShiftLeft => OperatorPrecedence::PRODUCT,
    Azula::TokenType::ShiftRight => OperatorPrecedence::PRODUCT,
    Azula::TokenType::Slash => OperatorPrecedence::PRODUCT,
    Azula::TokenType::Asterisk => OperatorPrecedence::PRODUCT,
    Azula::TokenType::Exponent => OperatorPrecedence::PRODUCT,
    Azula::TokenType::Modulo => OperatorPrecedence::PRODUCT,
    Azula::TokenType::LBracket => OperatorPrecedence::CALL,
    Azula::TokenType::LBrace => OperatorPrecedence::CALL,
    Azula::TokenType::LSquare => OperatorPrecedence::CALL,
    Azula::TokenType::Dot => OperatorPrecedence::ACCESS,
    Azula::TokenType::Ampersand => OperatorPrecedence::ACCESS,
}

class Azula::Parser

    @current_token : Azula::Token
    @peek_token : Azula::Token
    @errors : Array(Azula::Error)
    @infix_funcs : Hash(TokenType, Proc(AST::Node, AST::Node?))
    @prefix_funcs : Hash(TokenType, Proc(AST::Node?))

    getter errors

    def initialize(@lexer : Azula::Lexer)
        @current_token = @lexer.next_token
        @peek_token = @lexer.next_token
        @errors = [] of Azula::Error

        @infix_funcs = {} of TokenType => Proc(AST::Node, AST::Node?)
        @prefix_funcs = {} of TokenType => Proc(AST::Node?)

        register_prefix NumberLiteral, parse_number_literal
        register_prefix True, parse_boolean_literal
        register_prefix False, parse_boolean_literal
        register_prefix StringLiteral, parse_string_literal
        register_prefix Function, parse_function
        register_prefix Identifier, parse_identifier
        register_prefix Return, parse_return
        register_prefix If, parse_if

        register_infix Plus, parse_infix_expression
        register_infix Minus, parse_infix_expression
        register_infix Asterisk, parse_infix_expression
        register_infix Slash, parse_infix_expression
        register_infix Modulo, parse_infix_expression
        register_infix Ampersand, parse_infix_expression
        register_infix Pipe, parse_infix_expression
        register_infix ShiftLeft, parse_infix_expression
        register_infix ShiftRight, parse_infix_expression
        register_infix Eq, parse_infix_expression
        register_infix NotEq, parse_infix_expression
        register_infix Lt, parse_infix_expression
        register_infix LtEq, parse_infix_expression
        register_infix Gt, parse_infix_expression
        register_infix GtEq, parse_infix_expression
        register_infix LBracket, parse_function_call
    end

    # Advance the lexer to the next token
    def next_token
        @current_token = @peek_token
        @peek_token = @lexer.next_token
    end

    # Parse an entire program
    def parse_program : AST::Program
        return AST::Program.new parse_block
    end

    def parse_block : AST::Block
        nodes = [] of AST::Node
        while @current_token.type != TokenType::EOF
            node = self.parse_statement
            # If parse_statement returns nil, something went wrong, assume error already returned
            if node.nil?
                return AST::Block.new(nodes)
            end
            
            nodes << node
            if @peek_token.type == TokenType::Semicolon
                self.next_token
            end
            self.next_token
            next
        end

        return AST::Block.new nodes
    end

    # Parse a single statement
    def parse_statement : AST::Node?
        case @current_token.type
        when TokenType::Type
            return parse_assign_statement
        when TokenType::LBracket
            self.next_token
            val = self.parse_statement
            self.next_token
            return val
        else
            exp = parse_expression
            if exp.nil?
                return
            end
            return exp
        end
    end

    # Parse an expression - something that has a return value
    def parse_expression(precedence : OperatorPrecedence = OperatorPrecedence::LOWEST, close : TokenType = TokenType::Semicolon) : AST::Node?
        prefix = @prefix_funcs.fetch @current_token.type, nil
        if prefix.nil?
            @errors << Azula::Error.new "no prefix function for #{@current_token.type}", Azula::ErrorType::Parsing, @current_token
            return
        end

        left = prefix.call

        while @peek_token.type != close && precedence < Precedences.fetch(@peek_token.type, OperatorPrecedence::LOWEST)
            infix = @infix_funcs.fetch @peek_token.type, nil
            if infix.nil?
                @errors << Azula::Error.new "no infix function for #{@peek_token.type}", Azula::ErrorType::Parsing, @peek_token
                return
            end

            self.next_token
            left = infix.call left.not_nil!
        end

        return left
    end

    # Parse an assign statement in the form (int x = 10)
    def parse_assign_statement : AST::Assign?
        first_token = @current_token

        identifier = parse_typed_identifier
        if identifier.nil?
            return
        end

        # Expect an assign token
        if @peek_token.type != TokenType::Assign
            @errors << Azula::Error.new "expected =, found #{@peek_token.type.to_s.downcase}", Azula::ErrorType::Parsing, @current_token
            return
        end

        self.next_token
        self.next_token

        # Parse the value
        value = self.parse_statement
        if value.nil?
            return
        end

        return AST::Assign.new first_token, identifier, value
    end

    # Parse a function definition
    def parse_function : AST::Function?
        func_token = @current_token

        if @peek_token.type == TokenType::Identifier
            self.next_token
            func_name = @current_token.literal
        else
            func_name = ""
        end

        arguments = [] of AST::Identifier
        # If next token is (, there is arguments
        if @peek_token.type == TokenType::LBracket
            self.next_token
            self.next_token
            while @current_token.type != TokenType::RBracket && @peek_token.type != TokenType::EOF
                ident = parse_typed_identifier
                if ident.nil?
                    return
                end

                arguments << ident
                self.next_token
                if @current_token.type == TokenType::Comma
                    self.next_token
                end
            end
        end
        
        func_return = AST::Identifier.new @current_token, "", VoidType.new

        if @peek_token.type == TokenType::Colon
            self.next_token
            self.next_token
            return_type = parse_type
            if return_type.nil?
                @errors << Azula::Error.new "unknown type #{@current_token.literal}", Azula::ErrorType::Parsing, @current_token
                return
            end
            func_return = AST::Identifier.new @current_token, "", return_type.not_nil!
        end

        if @peek_token.type != TokenType::LBrace
            @errors << Azula::Error.new "unexpected #{@peek_token.literal}, expected {", Azula::ErrorType::Parsing, @peek_token
            return
        end
        self.next_token
        self.next_token

        stmts = [] of AST::Node
        while @current_token.type != TokenType::RBrace
            stmt = parse_statement
            if stmt.nil?
                return
            end
            stmts << stmt
            if @peek_token.type == TokenType::Semicolon
                self.next_token
            end
            self.next_token
        end

        return AST::Function.new func_token, AST::Identifier.new(func_token, func_name, nil), arguments, func_return, AST::Block.new(stmts)
    end

    def parse_function_call(left : AST::Node) : AST::FunctionCall?
        self.next_token # skip (

        # If current token is ), there are no arguments
        if @current_token.type == TokenType::RBracket
            return AST::FunctionCall.new @current_token, left, [] of AST::Node
        end
        args = parse_expression_list
        if args.nil?
            return
        end

        return AST::FunctionCall.new @current_token, left, args
    end

    # Parse an identifier accompanied by a type
    def parse_typed_identifier : AST::Identifier?
        type = parse_type
        if type.nil?
            @errors << Azula::Error.new "unknown type #{@current_token.literal}", Azula::ErrorType::Parsing, @current_token
            return
        end

        self.next_token

        ident = @current_token.literal

        return AST::Identifier.new @current_token, ident, type
    end

    def parse_type_list : Array(Azula::Type)?
        types = [] of Azula::Type
        while @current_token.type == TokenType::Identifier || @current_token.type == TokenType::Type
            type = parse_type
            if type.nil?
                @errors << Azula::Error.new "unknown type #{@current_token.literal}", Azula::ErrorType::Parsing, @current_token
                return
            end
            types << type
            self.next_token
            if @current_token.type == TokenType::Comma
                next_token
            end
        end
        return types
    end

    # Parse a single identifier
    def parse_identifier : AST::Identifier?
        return AST::Identifier.new @current_token, @current_token.literal, nil
    end

    def parse_return : AST::Return?
        return_token = @current_token

        if @peek_token.type == TokenType::Semicolon
            self.next_token
            return AST::Return.new return_token, nil
        end

        next_token
        val = parse_statement
        if val.nil?
            return
        end

        return AST::Return.new return_token, val
    end

    def parse_if : AST::If?
        if_token = @current_token

        self.next_token

        if @current_token.type != TokenType::LBracket 
            @errors << Azula::Error.new "unexpected #{@current_token.literal}, expected {", Azula::ErrorType::Parsing, @current_token
            return
        end
        
        self.next_token

        expression = parse_expression close: TokenType::RBracket
        if expression.nil?
            return
        end

        self.next_token
        self.next_token

        if @current_token.type != TokenType::LBrace 
            @errors << Azula::Error.new "unexpected #{@current_token.literal}, expected {", Azula::ErrorType::Parsing, @current_token
            return
        end

        self.next_token

        stmts = [] of AST::Node
        while @current_token.type != TokenType::RBrace
            stmt = parse_statement
            if stmt.nil?
                return
            end
            stmts << stmt
            if @peek_token.type == TokenType::Semicolon
                self.next_token
            end
            self.next_token
        end

        if @peek_token.type != TokenType::Else
            return AST::If.new if_token, expression, AST::Block.new(stmts), nil, [] of AST::If
        end

        self.next_token # else
        self.next_token # {
        if @current_token.type != TokenType::LBrace 
            @errors << Azula::Error.new "unexpected #{@current_token.literal}, expected {", Azula::ErrorType::Parsing, @current_token
            return
        end

        self.next_token

        else_stmts = [] of AST::Node
        while @current_token.type != TokenType::RBrace
            stmt = parse_statement
            if stmt.nil?
                return
            end
            else_stmts << stmt
            if @peek_token.type == TokenType::Semicolon
                self.next_token
            end
            self.next_token
        end

        return AST::If.new if_token, expression, AST::Block.new(stmts), AST::Block.new(else_stmts), [] of AST::If
    end

    # Parse a number literal, either int or float
    def parse_number_literal : AST::Node?
        if @peek_token.type == TokenType::Dot
            first = @current_token
            self.next_token
            self.next_token
            second = @current_token
            return AST::FloatLiteral.new first, "#{first.literal}.#{second.literal}".to_f64
        end
        return AST::IntegerLiteral.new @current_token, @current_token.literal.to_i64
    end

    def parse_string_literal : AST::Node?
        val = @current_token.literal[0..@current_token.literal.size-1]
        val = val.gsub("\\n", "\n")
        return AST::StringLiteral.new @current_token, val
    end

    # Parse a boolean literal, either true or false
    def parse_boolean_literal : AST::BooleanLiteral?
        return AST::BooleanLiteral.new @current_token, @current_token.type == TokenType::True
    end

    # Parse an infix expression, eg. 5 + 2 / 3 = (5 + (2 / 3))
    def parse_infix_expression(left : AST::Node) : AST::Node?
        tok = @current_token
        operator = @current_token
        
        precedence = Precedences.fetch tok.type, OperatorPrecedence::LOWEST
        self.next_token
        right = parse_expression precedence
        if right.nil?
            return
        end

        return AST::Infix.new tok, left, operator, right.not_nil!
    end

    # Parse a type definition and return the Azula type
    def parse_type : Azula::Type?
        case @current_token.literal
        when "int8"
            return Azula::IntegerType.new 8
        when "int16"
            return Azula::IntegerType.new 16
        when "int32"
            return Azula::IntegerType.new 32
        when "int"
            return Azula::IntegerType.new 32
        when "int64"
            return Azula::IntegerType.new 64
        when "float32"
            return Azula::FloatType.new 32
        when "float"
            return Azula::FloatType.new 32
        when "float64"
            return Azula::FloatType.new 64
        when "bool"
            return Azula::BooleanType.new
        when "string"
            return Azula::StringType.new
        when "array"
            array_type : Azula::Type? = nil
            # If left bracket, type is given, no need to infer
            if @peek_token.type == TokenType::LBracket
                self.next_token # array
                self.next_token # (
                array_type = parse_type
                if @peek_token.type != TokenType::RBracket
                    @errors << Azula::Error.new "bracket not closed", Azula::ErrorType::Parsing, @current_token
                    return
                end
                self.next_token # )
            end
            return Azula::ArrayType.new array_type
        when "func"
            args = [] of Azula::Type
            # If left bracket, type is given, no need to infer
            if @peek_token.type != TokenType::LBracket
                @errors << Azula::Error.new "missing function args", Azula::ErrorType::Parsing, @current_token
                return
            end
            self.next_token # function
            self.next_token # (
            args = parse_type_list
            if args.nil?
                return
            end
            if @current_token.type != TokenType::RBracket
                @errors << Azula::Error.new "bracket not closed", Azula::ErrorType::Parsing, @current_token
                return
            end
            self.next_token # )
            if @current_token.type != TokenType::Colon
                return Azula::FunctionType.new args, Azula::VoidType.new
            end
            self.next_token # :
            ret_type = parse_type
            if ret_type.nil?
                @errors << Azula::Error.new "unknown type #{@current_token.literal}", Azula::ErrorType::Parsing, @current_token
                return
            end
            return Azula::FunctionType.new args, ret_type
        else
            return
        end
    end

    def parse_expression_list : Array(AST::Node)?
        stmts = [] of AST::Node
        statement = parse_statement
        if statement.nil?
            return
        end
        stmts << statement
        self.next_token
        while @current_token.type == TokenType::Comma
            self.next_token
            statement = parse_statement
            if statement.nil?
                return
            end

            stmts << statement
            self.next_token
        end
        return stmts
    end

end