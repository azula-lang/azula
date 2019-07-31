require "./ast"
require "../token"

module Azula
    module AST

        # An infix expression is an expression that contains two other `Expression` with an operator between them.
        # Eg. 5 + 2
        class Infix < Expression

            @token : Token
            @left : Expression
            @operator : String
            @right : Expression

            def initialize(@token, @left, @operator, @right)   
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "(#{@left.to_string} #{@operator} #{@right.to_string})"
            end

            # Get the `Token` in this node.
            getter token
            # The `Expression` on the left side of the infix expression.
            getter left
            # The operator that will dictate the operation between the left and right.
            getter operator
            # The `Expression` on the right side of the infix expression.
            getter right

        end
    end
end