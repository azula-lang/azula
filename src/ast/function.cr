require "./ast"

module Azula::AST

    # Function node
    class Function < Node
        getter token, identifier, arguments, returns, body
        def initialize(@token : Azula::Token, @identifier : AST::Identifier, @arguments : Array(AST::Identifier), @returns : AST::Identifier, @body : AST::Block)
        end

        def to_s : String
            return "func " +  @identifier.to_s + "(#{arguments.map {|s| s.to_s }.join(", ")}): " + @returns.to_s + "{\n" + @body.to_s + "\n}"
        end
    end

    # Function call node
    class FunctionCall < Node
        getter token, function, arguments
        def initialize(@token : Azula::Token, @function : AST::Node, @arguments : Array(AST::Node))
        end

        def to_s : String
            return @function.to_s + "(#{arguments.map {|s| s.to_s }.join(", ")})"
        end
    end

end