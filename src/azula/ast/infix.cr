require "./ast"
require "../token"

module Azula
    module AST

        class Infix < Expression

            @token : Token
            @left : Expression
            @operator : String
            @right : Expression

            def initialize(@token, @left, @operator, @right)   
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "(#{@left.to_string} #{@operator} #{@right.to_string})"
            end

            getter token
            getter left
            getter operator
            getter right

        end
    end
end