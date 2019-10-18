require "./ast"
require "../token"
require "../types"

module Azula
    module AST
        # Identifier is a word/phrase used as an alias to a variable or function.
        class Identifier < Expression

            @token : Token
            @ident : String

            def initialize(@token, @ident)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return @ident
            end

            # Get the `Token` in this node.
            getter token
            # The identifier in string form.
            getter ident

        end

        # TypedIdentifier a word/phrase, paired with a `Type`, used to identify a variable.
        class TypedIdentifier < Statement

            @token : Token
            @ident : String
            @type : Types::Type

            def initialize(@token, @ident, @type)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "#{@type.main_type.to_s.downcase} #{@ident}"
            end

            # The literal string representation of this node.
            getter token
            # The identifier in string form.
            getter ident
            # The type of this identifier.
            getter type

        end
    end
end