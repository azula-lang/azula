module Azula::AST

    # Identifier is a name that represents a variable/function etc, can be paired with a type.
    class Identifier < Node
        getter token, value, type
        setter type
        def initialize(@token : Azula::Token, @value : String, @type : Azula::Type?)
        end

        def to_s : String
            return "#{@type.nil? ? "" : @type.not_nil!.to_s + " "}" + @value.to_s
        end
    end

end