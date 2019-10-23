module Azula

    module Types

        class Type

            @main_type : (TypeEnum | String)
            @secondary_type  : Type?

            getter main_type
            getter secondary_type
            setter secondary_type

            def initialize(@main_type, @secondary_type = nil)
                
            end

            def self.type_from_string(s : String) : Type
                case s
                when "int"
                    return Type.new TypeEnum::INT
                when "int8"
                    return Type.new TypeEnum::INT8
                when "int16"
                    return Type.new TypeEnum::INT16
                when "int64"
                    return Type.new TypeEnum::INT64
                when "float"
                    return Type.new TypeEnum::FLOAT
                when "string"
                    return Type.new TypeEnum::STRING
                when "cstring"
                    return Type.new TypeEnum::CSTRING
                when "bool"
                    return Type.new TypeEnum::BOOL
                when "void"
                    return Type.new TypeEnum::VOID
                when "error"
                    return Type.new TypeEnum::ERROR
                when "array"
                    return Type.new TypeEnum::ARRAY
                end

                return Type.new s
            end

            def is_int : Bool
                return @main_type == TypeEnum::INT || @main_type == TypeEnum::INT8 || @main_type == TypeEnum::INT16 || @main_type == TypeEnum::INT64
            end

        end

        enum TypeEnum
            INT
            INT8
            INT16
            INT64
            FLOAT
            STRING
            CSTRING
            BOOL
            VOID
            ERROR
            POINTER
            ARRAY
        end

    end

end