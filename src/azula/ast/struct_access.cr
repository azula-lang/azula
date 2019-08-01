require "./ast"
require "../token"

module Azula
    module AST

        # Struct access expression is used to get a value from a struct instance.
        class StructAccess < Expression

            @token : Token
            @struct_exp : Expression
            @field : Expression

            def initialize(@token, @struct_exp, @field)   
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "(#{struct_exp.to_string}.#{field.to_string})"
            end

            # Get the `Token` in this node.
            getter token
            # The `Struct` that the field will be accessed in.
            getter struct_exp
            # The field to be accessed inside the `Struct`.
            getter field

        end
    end
end