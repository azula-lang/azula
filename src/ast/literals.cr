module Azula::AST

    # Literal integer value
    class IntegerLiteral < Node
        getter token, value
        def initialize(@token : Azula::Token, @value : Int64)
        end

        def to_s : String
            return @value.to_s
        end
    end

    # Literal float value
    class FloatLiteral < Node
        getter token, value
        def initialize(@token : Azula::Token, @value : Float64)
        end

        def to_s : String
            return @value.to_s
        end
    end

    # Literal boolean value
    class BooleanLiteral < Node
        getter token, value
        def initialize(@token : Azula::Token, @value : Bool)
        end

        def to_s : String
            return @value.to_s
        end
    end

    # Literal boolean value
    class StringLiteral < Node
        getter token, value
        def initialize(@token : Azula::Token, @value : String)
        end

        def to_s : String
            return @value
        end
    end

end