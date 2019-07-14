module Azula

    module Types
        enum Type
            INT
            FLOAT
            STRING
            BOOL
            ERROR
        end

        def Types.type_from_string(s : String) : Type?
            case s
            when "int"
                return Type::INT
            when "float"
                return Type::FLOAT
            when "string"
                return Type::STRING
            when "bool"
                return Type::BOOL
            when "error"
                return Type::ERROR
            end
        end
    end

end