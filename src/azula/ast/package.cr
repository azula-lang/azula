require "./ast"
require "../token"

module Azula
    module AST

        class Package < Statement

            @token : Token
            @name : String

            def initialize(@token, @name)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{self.token_literal} #{name}"
            end

            # Get the `Token` in this node.
            getter token
            # The `Identifier` used to reference this struct.
            getter name

        end
    end
end