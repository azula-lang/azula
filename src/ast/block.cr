module Azula::AST

    # Block node contains a list of a nodes
    class Block < Node
        getter nodes
        def initialize(@nodes : Array(Node))
        end

        def to_s : String
            return nodes.map { |node| node.to_s }.join("\n")
        end
    end

end