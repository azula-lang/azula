require "./ast"
require "../token"

module Azula
    module AST

        # Access expression is used to access something in an identifier.
        class Access < Expression

            @token : Token
            @left_exp : Expression
            @access_field : Expression

            def initialize(@token, @left_exp, @access_field)   
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "(#{left_exp.to_string}.#{access_field.to_string})"
            end

            # Get the `Token` in this node.
            getter token
            # The `Struct` that the field will be accessed in.
            getter left_exp
            # The field to be accessed inside the `Struct`.
            getter access_field

        end
    end
end