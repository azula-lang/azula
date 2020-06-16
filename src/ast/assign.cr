require "./ast"

module Azula::AST

    # Value assign node eg. int x = 5, var y = 10
    class Assign < Node
        getter token, identifier, value
        def initialize(@token : Azula::Token, @identifier : AST::Identifier, @value : AST::Node)
        end

        def to_s : String
            return @identifier.to_s + " = " + @value.to_s
        end
    end

end