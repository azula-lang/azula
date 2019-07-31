require "./ast"
require "../token"

module Azula
    module AST

        class ExpressionStatement < Statement

            @token : Token
            @expression : Expression

            def initialize(@token, @expression)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "#{@expression.to_string}"
            end

            getter token
            getter expression

        end
    end
end