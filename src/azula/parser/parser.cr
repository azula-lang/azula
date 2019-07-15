require "../ast/*"
require "../token"
require "../lexer"
require "../types/*"

macro register_prefix(token_type, method_name)
    @prefix_funcs[TokenType::{{token_type}}] = ->{ self.{{method_name}}.as(AST::Expression?)}
end

macro register_infix(token_type, method_name)
    @infix_funcs[TokenType::{{token_type}}] = ->(exp: AST::Expression){ self.{{method_name}}(exp).as(AST::Expression?)}
end

macro expect_peek_return(token_type)
    if !self.expect_peek TokenType::{{token_type}}
        return
    end
end

macro nil_return(var)
    if {{var}}.nil?
        return
    end
end

macro type_or_literal
    type = Types::Type.parse? @current_token.literal
    if type.nil?
        types << @current_token.literal
    else
        types << type
    end
end

module Azula

    enum OperatorPrecedence
        LOWEST,
        COMPARISON,
        EQUALS,
        LESS_GREATER,
        SUM,
        PRODUCT,
        PREFIX,
        CALL,
        ACCESS,
    end

    Precedences = {
        TokenType::OR => OperatorPrecedence::COMPARISON,
        TokenType::AND => OperatorPrecedence::COMPARISON,
        TokenType::EQ => OperatorPrecedence::EQUALS,
        TokenType::NOT_EQ => OperatorPrecedence::EQUALS,
        TokenType::LT => OperatorPrecedence::LESS_GREATER,
        TokenType::LT_EQ => OperatorPrecedence::LESS_GREATER,
        TokenType::GT => OperatorPrecedence::LESS_GREATER,
        TokenType::GT_EQ => OperatorPrecedence::LESS_GREATER,
        TokenType::PLUS => OperatorPrecedence::SUM,
        TokenType::MINUS => OperatorPrecedence::SUM,
        TokenType::SLASH => OperatorPrecedence::PRODUCT,
        TokenType::ASTERISK => OperatorPrecedence::PRODUCT,
        TokenType::EXPONENT => OperatorPrecedence::PRODUCT,
        TokenType::MODULO => OperatorPrecedence::PRODUCT,
        TokenType::LBRACKET => OperatorPrecedence::CALL,
        TokenType::LBRACE => OperatorPrecedence::CALL,
        TokenType::DOT => OperatorPrecedence::ACCESS,
    }

    class Parser

        @lexer : Lexer
        @errors : Array(String)
        @current_token : Token
        @peek_token : Token
        @infix_funcs : Hash(TokenType, Proc(AST::Expression, AST::Expression?))
        @prefix_funcs : Hash(TokenType, Proc(AST::Expression?))

        getter errors

        def initialize(@lexer)
            @errors = [] of String
            @current_token = @lexer.next_token
            @peek_token = @lexer.next_token

            @infix_funcs = {} of TokenType => Proc(AST::Expression, AST::Expression?)
            @prefix_funcs = {} of TokenType => Proc(AST::Expression?)

            register_prefix NUMBER, parse_number_literal
            register_prefix STRING, parse_string_literal
            register_prefix TRUE, parse_boolean_literal
            register_prefix FALSE, parse_boolean_literal
            register_prefix LBRACKET, parse_grouped_expression
            register_prefix IDENTIFIER, parse_identifier

            register_infix PLUS, parse_infix_expression
            register_infix MINUS, parse_infix_expression
            register_infix SLASH, parse_infix_expression
            register_infix ASTERISK, parse_infix_expression
            register_infix EXPONENT, parse_infix_expression
            register_infix MODULO, parse_infix_expression
            register_infix EQ, parse_infix_expression
            register_infix NOT_EQ, parse_infix_expression
            register_infix LT, parse_infix_expression
            register_infix LT_EQ, parse_infix_expression
            register_infix GT, parse_infix_expression
            register_infix GT_EQ, parse_infix_expression
            register_infix OR, parse_infix_expression
            register_infix AND, parse_infix_expression
            register_infix LBRACKET, parse_function_call_expression
            register_infix LBRACE, parse_struct_initialising
            register_infix DOT, parse_struct_access
        end

        def next_token
            @current_token = @peek_token
            @peek_token = @lexer.next_token
        end

        def parse_program : AST::Program
            statements = [] of AST::Statement
            while @current_token.type != TokenType::EOF
                stmt = self.parse_statement
                if !stmt.nil?
                    statements << stmt
                else
                    return AST::Program.new statements
                end
                self.next_token
            end
            return AST::Program.new statements
        end

        def parse_statement : AST::Statement?
            case @current_token.type
            when TokenType::TYPE
                return self.parse_assign_statement
            when TokenType::RETURN
                return self.parse_return_statement
            when TokenType::FUNCTION
                return self.parse_function_statement
            when TokenType::STRUCT
                return self.parse_struct
            when TokenType::COLON
                self.next_token
                return self.parse_assign_statement
            when TokenType::IF
                return self.parse_if_statement true
            when TokenType::WHILE
                return self.parse_while_loop
            end
            case @peek_token.type
            when TokenType::COMMA
                return self.parse_assign_statement
            else
                return self.parse_expression_statement
            end
        end

        def parse_block_statement : AST::Block?
            tok = @current_token
            stmts = [] of AST::Statement
            self.next_token

            while @current_token.type != TokenType::RBRACE && @current_token.type != TokenType::EOF
                if @current_token.type == TokenType::EOF
                    self.add_error "body has no close"
                    return
                end
                stmt = self.parse_statement
                if !stmt.nil?
                    stmts << stmt
                else
                    return
                end
                self.next_token
            end

            if @current_token.type != TokenType::RBRACE
                self.add_error "expected {, got EOF"
                return
            end

            return AST::Block.new tok, stmts
        end

        def expect_peek(t : TokenType) : Bool
            if @peek_token.type == t
                self.next_token
                return true
            end
            self.add_error "expected next token to be #{t}, got #{@peek_token.type} instead"
            return false
        end

        def parse_expression(precedence : OperatorPrecedence = OperatorPrecedence::LOWEST, close : TokenType = TokenType::SEMICOLON) : AST::Expression?
            prefix = @prefix_funcs.fetch @current_token.type, nil
            if prefix.nil?
                self.add_error "no prefix function for #{@current_token.type}"
                return
            end

            left = prefix.call

            while @peek_token.type != close && precedence < self.token_precedence(@peek_token.type)
                infix = @infix_funcs.fetch @peek_token.type, nil
                nil_return infix

                self.next_token
                nil_return left
                left = infix.call left.not_nil!
            end

            return left
        end

        def parse_assign_statement : AST::Assign?
            t = @current_token
            idents = [] of (AST::TypedIdentifier | AST::Identifier)
            ident = parse_typed_identifier
            if ident.nil?
                ident = AST::Identifier.new @current_token, @current_token.literal
            end

            idents << ident

            while @peek_token.type == TokenType::COMMA
                self.next_token
                self.next_token
                ident = parse_typed_identifier
                if ident.nil?
                    return
                end

                idents << ident
            end

            expect_peek_return ASSIGN

            values = self.parse_expression_list TokenType::SEMICOLON
            nil_return values

            return AST::Assign.new t, idents, values.not_nil!
        end

        def parse_typed_identifier : AST::TypedIdentifier?
            assign_token = @current_token
            type = Types::Type.parse? @current_token.literal
            if type.nil?
                type = @current_token.literal
            end

            expect_peek_return IDENTIFIER

            return AST::TypedIdentifier.new @current_token, @current_token.literal, type
        end

        def parse_string_literal : AST::StringLiteral
            return AST::StringLiteral.new @current_token, @current_token.literal
        end

        def parse_number_literal : (AST::IntegerLiteral? | AST::FloatLiteral?)
            if @peek_token.type == TokenType::DOT
                first = @current_token
                next_token
                next_token
                second = @current_token
                val = "#{first.literal}.#{second.literal}".to_f
                if !val.nil?
                    return AST::FloatLiteral.new first, val
                end
                self.add_error "could not parse float"
                return
            end
            val = @current_token.literal.to_i
            if val.nil?
                self.add_error "could not parse integer"
            end
            return AST::IntegerLiteral.new @current_token, val
        end

        def parse_boolean_literal : AST::BooleanLiteral
            return AST::BooleanLiteral.new @current_token, @current_token.type == TokenType::TRUE
        end

        def parse_return_statement : AST::Return?
            tok = @current_token

            values = self.parse_expression_list TokenType::SEMICOLON

            return AST::Return.new tok, values
        end

        def parse_infix_expression(left : AST::Expression) AST::Expression?
            tok = @current_token
            operator = @current_token.literal
            
            precedence = token_precedence tok.type
            self.next_token
            right = parse_expression precedence
            nil_return right

            return AST::Infix.new tok, left, operator, right.not_nil!
        end

        def parse_expression_list(last : TokenType) Array(AST::Expression)
            exps = [] of AST::Expression

            if @peek_token.type == last
                self.next_token
                return exps
            end

            self.next_token
            exp = self.parse_expression
            if !exp.nil?
                exps << exp
            end

            while @peek_token.type == TokenType::COMMA
                self.next_token
                self.next_token
                exp = self.parse_expression
                if !exp.nil?
                    exps << exp
                end
            end

            if !expect_peek last
            end

            return exps
        end

        def parse_identifier : AST::Identifier?
            return AST::Identifier.new @current_token, @current_token.literal
        end

        def parse_function_statement : AST::Function?
            tok = @current_token
            self.next_token
            name = AST::Identifier.new @current_token, @current_token.literal

            expect_peek_return LBRACKET

            params = self.parse_function_parameters
            nil_return params

            expect_peek_return COLON

            self.next_token

            return_types = self.parse_function_return_types
            nil_return return_types

            expect_peek_return LBRACE

            body = self.parse_block_statement
            nil_return body

            return AST::Function.new tok, name, params, return_types, body.not_nil!
        end

        def parse_function_parameters : Array(AST::TypedIdentifier)?
            idents = [] of AST::TypedIdentifier

            if @peek_token.type == TokenType::RBRACKET
                self.next_token
                return idents
            end

            self.next_token

            ident = self.parse_typed_identifier
            nil_return ident
            idents << ident

            while @peek_token.type == TokenType::COMMA
                self.next_token
                self.next_token
                ident = self.parse_typed_identifier
                nil_return ident
                idents << ident
            end

            expect_peek_return RBRACKET

            return idents
        end

        def parse_function_return_types : Array(Types::Type | String)?
            types = [] of (Types::Type | String)
            if @current_token.type == TokenType::LBRACKET
                self.next_token
                type_or_literal
                while @peek_token.type == TokenType::COMMA
                    self.next_token
                    self.next_token
                    type_or_literal
                end

                expect_peek_return RBRACKET
            else
                type_or_literal
            end
            return types
        end

        def parse_grouped_expression : AST::Expression?
            self.next_token
            exp = self.parse_expression
            expect_peek_return RBRACKET

            return exp
        end

        def parse_function_call_expression(function : AST::Expression) AST::Expression?
            function = function.as?(AST::Identifier)
            nil_return function
            tok = @current_token
            args = self.parse_expression_list TokenType::RBRACKET
            nil_return args

            return AST::FunctionCall.new tok, function.as(AST::Identifier), args
        end

        def parse_expression_statement : AST::ExpressionStatement?
            tok = @current_token
            exp = self.parse_expression
            nil_return exp
            expect_peek_return SEMICOLON

            return AST::ExpressionStatement.new tok, exp.not_nil!
        end

        def parse_struct : AST::Struct?
            tok = @current_token
            self.next_token
            name = AST::Identifier.new @current_token, @current_token.literal

            fields = [] of AST::TypedIdentifier
            expect_peek_return LBRACE

            while @peek_token.type != TokenType::RBRACE
                self.next_token
                exp = self.parse_typed_identifier
                nil_return exp
                fields << exp.not_nil!
                if @peek_token.type != TokenType::COMMA
                    break
                end
                self.next_token
            end

            expect_peek_return RBRACE

            self.next_token

            return AST::Struct.new tok, name, fields
        end

        def parse_struct_initialising(struct_ident : AST::Expression) : AST::Expression?
            struct_ident = struct_ident.as?(AST::Identifier)
            nil_return struct_ident
            tok = @current_token
            args = self.parse_expression_list TokenType::RBRACE
            nil_return args

            return AST::StructInitialise.new tok, struct_ident, args
        end

        def parse_if_statement(top_level : Bool) : AST::If?
            tok = @current_token
            expect_peek_return LBRACKET
            self.next_token
            exp = self.parse_expression OperatorPrecedence::LOWEST, TokenType::RBRACKET
            if exp.nil?
                self.add_error "invalid condition"
            end
            nil_return exp
            expect_peek_return RBRACKET
            expect_peek_return LBRACE
            cond = self.parse_block_statement
            nil_return cond

            if !top_level
                return AST::If.new tok, exp.not_nil!, cond.not_nil!, [] of AST::If, nil
            end

            alts = [] of AST::If

            while @peek_token.type == TokenType::ELSEIF
                self.next_token
                alt = self.parse_if_statement false
                nil_return alt
                alts << alt
            end

            alternative : AST::Block?

            if @peek_token.type == TokenType::ELSE
                self.next_token
                expect_peek_return LBRACE
                alternative = self.parse_block_statement
                nil_return alternative
            end

            return AST::If.new tok, exp.not_nil!, cond.not_nil!, alts, alternative
        end

        def parse_while_loop : AST::While?
            tok = @current_token
            expect_peek_return LBRACKET
            self.next_token
            exp = self.parse_expression OperatorPrecedence::LOWEST, TokenType::RBRACKET
            nil_return exp

            expect_peek_return RBRACKET
            expect_peek_return LBRACE
            body = self.parse_block_statement
            nil_return body

            return AST::While.new tok, exp, body
        end

        def parse_struct_access(s : AST::Expression) : AST::StructAccess
            tok = @current_token
            self.next_token
            field = AST::Identifier.new @current_token, @current_token.literal

            return AST::StructAccess.new tok, s, field
        end

        def token_precedence(type : TokenType) OperatorPrecedence
            pre = Precedences.fetch type, nil
            if pre.nil?
                return OperatorPrecedence::LOWEST
            end
            return pre
        end

        def add_error(error : String)
            @errors << "#{error} (file #{@current_token.file}, line #{@current_token.linenumber}, char #{@current_token.charnumber})"
        end

    end
end