require "./ast"
require "../token"
require "./block"

module Azula
    module AST

        class While < Statement

            @token : Token
            @iterator : Expression
            @body : Block

            def initialize(@token, @iterator, @body)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "#{self.token_literal}(#{@iterator.to_string}) {\n#{@body.to_string}}"
            end

            getter token
            getter iterator
            getter body

        end
    end
end