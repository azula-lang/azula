module Azula::AST

    # Program node is the root node for a program
    class Program < Node
        getter block
        def initialize(@block : AST::Block)
        end

        def to_s : String
            return block.to_s
        end
    end

end