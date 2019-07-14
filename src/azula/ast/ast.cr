module Azula
    # AST contains classes for each node
    module AST

        abstract class Node
            abstract def token_literal : String
            abstract def to_string : String
        end

        abstract class Statement < Node
        end

        abstract class Expression < Node
        end

    end
end