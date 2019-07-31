require "./ast"
require "../token"

module Azula
    module AST
        # Block statement is for a group of statements.
        class Block < Statement

            @token : Token
            @statements : Array(Statement)

            def initialize(@token, @statements)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                output = ""
                @statements.each do |statement|
                    output += statement.to_string + "\n"
                end
                return output
            end

            # Get the `Token` in this node.
            getter token
            # Get the list of `Statement` in this block.
            getter statements

        end

    end
end