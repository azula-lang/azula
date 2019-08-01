require "./ast"
require "../token"

module Azula
    module AST

        # A prefix expression is when an operator is put in front of an expression.
        # Eg. !true, -5
        class Prefix < Expression

            @token : Token
            @operator : String
            @right : Expression

            def initialize(@token, @operator, @right)   
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "(#{@operator}#{@right.to_string})"
            end

            # Get the `Token` in this node.
            getter token
            # The prefix operator placed in front of expression.
            getter operator
            # The expression to be changed by the operator.
            getter right

        end
    end
end