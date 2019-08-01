require "./ast"

module Azula
    module AST
        # Program statement contains all the statements in a program.
        class Program < Statement

            @statements : Array(Statement)

            def initialize(@statements)
            end

            # The literal token representation of this node.
            def token_literal : String
                return not @statements.empty? ? @statements[0].token_literal : ""
            end

            # The literal string representation of this node.
            def to_string : String
                output = ""
                @statements.each do |statement|
                    output += statement.to_string + "\n"
                end
                return output[0..output.size-2]
            end

            # The statements in this Program.
            getter statements

        end
    end
end