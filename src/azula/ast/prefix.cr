require "./ast"
require "../token"

module Azula
    module AST

        class Prefix < Expression

            @token : Token
            @operator : String
            @right : Expression

            def initialize(@token, @operator, @right)   
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "(#{@operator}#{@right.to_string})"
            end

            getter token
            getter operator
            getter right

        end
    end
end