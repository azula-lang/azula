require "../types"

module Azula
    module Compiler

        class LLVMVar
            getter pointer : LLVM::Value
            getter type : Types::Type

            def initialize(@pointer, @type)
            end
        end

    end
end