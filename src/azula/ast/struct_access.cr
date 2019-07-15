require "./ast"
require "../token"

module Azula
    module AST

        class StructAccess < Expression

            @token : Token
            @struct_exp : Expression
            @field : Expression

            def initialize(@token, @struct_exp, @field)   
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "(#{struct_exp.to_string}.#{field.to_string})"
            end

            getter token
            getter struct_exp
            getter field

        end
    end
end