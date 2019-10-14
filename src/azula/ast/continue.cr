require "./ast"
require "../token"

module Azula
    module AST

        # Continue statement is used to skip a loop.
        class Continue < Statement

            @token : Token

            def initialize(@token)   
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{self.token_literal}"
            end

            # Get the `Token` in this node.
            getter token

        end
    end
end