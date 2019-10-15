module Azula

    module Types
        enum Type
            INT
            INT8
            INT16
            FLOAT
            STRING
            CSTRING
            BOOL
            VOID
            ERROR
            POINTER
        end

        def Types.type_from_string(s : String) : Type?
            case s
            when "int"
                return Type::INT
            when "int8"
                return Type::INT8
            when "int16"
                return Type::INT16
            when "float"
                return Type::FLOAT
            when "string"
                return Type::STRING
            when "cstring"
                return Type::CSTRING
            when "bool"
                return Type::BOOL
            when "void"
                return Type::VOID
            when "error"
                return Type::ERROR
            end
        end
    end

end