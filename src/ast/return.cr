module Azula::AST

    # Return allows you to return a value from a function
    class Return < Node
        getter token, value
        def initialize(@token : Azula::Token, @value : AST::Node?)
        end

        def to_s : String
            return "return " + (@value.nil? ? "" : @value.to_s)
        end
    end

end