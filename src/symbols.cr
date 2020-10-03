class Azula::SymbolTable

    @symbols : Hash(String, Azula::Type)

    def initialize
        @symbols = {} of String=>Azula::Type
    end

    def initialize(@symbols)
    end

    def add(label : String, type : Azula::Type)
        @symbols[label] = type
    end

    def exists?(label : String) : Bool
        return @symbols.has_key?(label)
    end

    def get(label : String) : Azula::Type?
        return @symbols.fetch label, nil
    end

    def remove(label : String)
        if exists?(label)
            @symbols.delete label
        end
    end

    def copy : SymbolTable
        return SymbolTable.new @symbols
    end

end