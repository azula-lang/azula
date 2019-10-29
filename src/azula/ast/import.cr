require "./ast"
require "../token"
require "./identifier"

module Azula
    module AST

        class Import < Statement

            @token : Token
            @imports : Array(String)

            def initialize(@token, @imports)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{self.token_literal} {\n"
                @imports.each do |import|
                    s += import + "\n"
                end
                s += "}"
            end

            # Get the `Token` in this node.
            getter token
            # The `Identifier` used to reference this struct.
            getter imports

        end
    end
end