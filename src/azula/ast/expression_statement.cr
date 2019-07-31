require "./ast"
require "../token"

module Azula
    module AST

        # Expression statements are for when an expression is not assigned or inside a statement.
        # An example of an expression statement would be a function that returns a value, but it is not assigned to anything.
        class ExpressionStatement < Statement

            @token : Token
            @expression : Expression

            def initialize(@token, @expression)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "#{@expression.to_string}"
            end

            # Get the `Token` in this node.
            getter token
            # The `Expression` in this expression statement.
            getter expression

        end
    end
end