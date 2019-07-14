require "./ast"
require "../token"
require "../types"

module Azula
    module AST
        # Identifier is used for references
        class Identifier < Expression

            @token : Token
            @ident : String

            def initialize(@token, @ident)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return @ident
            end

            getter token
            getter ident

        end

        # TypedIdentifier is used when initialising a variable with a type
        class TypedIdentifier < Statement

            @token : Token
            @ident : String
            @type : (Types::Type | String)

            def initialize(@token, @ident, @type)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "#{@type.to_s.downcase} #{@ident}"
            end

            getter token
            getter ident
            getter type

        end
    end
end