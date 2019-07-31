module Azula
    # AST contains classes for each node
    module AST

        abstract class Node
            # The literal token representation of this node
            abstract def token_literal : String
            # The string representation of this node
            abstract def to_string : String
        end

        abstract class Statement < Node
        end

        abstract class Expression < Node
        end

    end
end