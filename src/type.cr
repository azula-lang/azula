module Azula
    # Abstract class the has every type in Azula as a subclass
    abstract class Type
        def is_pointer?
            false
        end

        def is_struct?
            false
        end

        abstract def to_s : String
        abstract def is(t : Azula::Type) : Bool
    end

    class IntegerType < Type
        getter size
        
        def initialize(@size : Int32 = 32)
        end

        def to_s : String
            return "int#{@size}"
        end

        def is(t : Type) : Bool
            if !t.is_a?(IntegerType)
                return false
            end
            return true
        end
    end

    class FloatType < Type
        getter size
        
        def initialize(@size : Int32 = 32)
        end

        def to_s : String
            return "float#{@size}"
        end

        def is(t : Type) : Bool
            if !t.is_a?(FloatType)
                return false
            end
            return true
        end
    end

    class BooleanType < Type
        def to_s : String
            return "bool"
        end

        def is(t : Type) : Bool
            if !t.is_a?(BooleanType)
                return false
            end
            return true
        end
    end

    class StringType < Type
        def to_s : String
            return "string"
        end

        def is(t : Type) : Bool
            if !t.is_a?(StringType)
                return false
            end
            return true
        end
    end

    class VoidType < Type
        def to_s : String
            return "void"
        end

        def is(t : Type) : Bool
            if !t.is_a?(VoidType)
                return false
            end
            return true
        end
    end

    class ArrayType < Type
        getter array_type
        def initialize(@array_type : Type?)
        end

        def to_s : String
            return "array(" + (@array_type.nil? ? "unknown" : @array_type.not_nil!.to_s) + ")"
        end

        def is(t : Type) : Bool
            if !t.is_a?(ArrayType)
                return false
            end
            if @array_type.nil?
                return true
            end

            array = t.as(ArrayType)
            if !array.array_type.not_nil!.is(@array_type.not_nil!)
                return false
            end 
            return true
        end
    end

    class FunctionType < Type
        getter arguments, returns, any_args
        def initialize(@arguments : Array(Type), @returns : Type)
            @any_args = false
        end

        def initialize(@returns : Type)
            @arguments = [] of Type
            @any_args = true
        end

        def to_s : String
            return "func(" + (@arguments.map { |s| s.to_s }.join(" ") + "): " + returns.to_s)
        end

        def is(t : Type) : Bool
            if !t.is_a?(FunctionType)
                return false
            end

            if !@any_args
                t.arguments.each_with_index do |arg, index|
                    if !arg.is(t.arguments[index])
                        return false
                    end
                end
            end

            return @returns.is(t.returns)
        end
    end

    class PointerType < Type
        getter nested_type
        def initialize(@nested_type : Type)
        end

        def to_s : String
            "@" + nested_type.to_s
        end

        def is(t : Type) : Bool
            if !t.is_a?(PointerType)
                return false
            end
            return @nested_type.is(t.as(PointerType).nested_type)
        end

        def is_pointer? : Bool
            true
        end
    end

    class InferType < Type
        def initialize
        end

        def to_s : String
            "var"
        end

        def is(t : Type) : Bool
            return true
        end

        def is_pointer? : Bool
            false
        end
    end
end