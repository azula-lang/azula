require "./ast"
require "../token"

module Azula
    module AST
        # Block statement is for a group of statements eg inside if
        class Block < Statement

            @token : Token
            @statements : Array(Statement)

            def initialize(@token, @statements)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                output = ""
                @statements.each do |statement|
                    output += statement.to_string + "\n"
                end
                return output
            end

            getter token
            getter statements

        end

    end
end