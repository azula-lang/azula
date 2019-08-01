require "./ast"
require "../token"
require "./block"

module Azula
    module AST

        # While loop repeats until iterator evaluates to false.
        class While < Statement

            @token : Token
            @iterator : Expression
            @body : Block

            def initialize(@token, @iterator, @body)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "#{self.token_literal}(#{@iterator.to_string}) {\n#{@body.to_string}}"
            end

            # Get the `Token` in this node.
            getter token
            # The condition for this while loop. While loop will keep running until this evaluates to false.
            getter iterator
            # The `Block` that will be run continuously until the iterator evaluates to false.
            getter body

        end
    end
end