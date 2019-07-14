require "./ast"

module Azula
    module AST
        # Program is a container for all statements
        class Program < Statement

            @statements : Array(Statement)

            def initialize(@statements)
            end

            def token_literal : String
                return not @statements.empty? ? @statements[0].token_literal : ""
            end

            def to_string : String
                output = ""
                @statements.each do |statement|
                    output += statement.to_string + "\n"
                end
                return output
            end

        end
    end
end