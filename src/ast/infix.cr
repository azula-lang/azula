require "./ast"

module Azula::AST

    # Value assign node eg. int x = 5, var y = 10
    class Infix < Node
        getter token, left, operator, right
        def initialize(@token : Azula::Token, @left : AST::Node, @operator : Azula::Token, @right : AST::Node)
        end

        def to_s : String
            return "(#{@left.to_s} #{@operator.literal} #{@right.to_s})"
        end
    end

end