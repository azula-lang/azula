require "./ast"
require "./identifier"
require "../token"

module Azula
    module AST

        class Assign < Statement

            @token : Token
            @ident : (TypedIdentifier | Identifier)
            @value : Expression

            def initialize(@token, @ident, @value)   
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "#{@ident.to_string} = #{@value.to_string}"
            end

            getter token
            getter ident
            getter value

        end
    end
end