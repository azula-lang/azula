module Azula
    module Compiler
        class Compiler

            # Add the builtin functions
            def add_builtins
                print_string = @main_module.functions.add("__print_s", [@string_type.pointer], @context.void, true) do |func|
                    entry = func.basic_blocks.append "entry" do | builder |
                        v = builder.gep func.params[0], @context.int32.const_int(0), @context.int32.const_int(0)
                        val = builder.load v
                        builder.call @builtin_printfunc, val
                        builder.ret
                    end
                end
                print_int = @main_module.functions.add("__print_int", [@context.int32], @context.void, true) do |func|
                    entry = func.basic_blocks.append "entry" do | builder |
                        builder.call @builtin_printfunc, [builder.global_string_pointer("%d"), func.params[0]]
                        builder.ret
                    end
                end
                print_bool = @main_module.functions.add("__print_bool", [@context.int1], @context.void, true) do |func|
                    entry = func.basic_blocks.append "entry" do | builder |
                        # Compile the condition
                        condition = func.params[0]

                        # Compile the If Block
                        if_block = func.basic_blocks.append "if" do | b |
                            b.call @builtin_printfunc, [b.global_string_pointer("true")]
                            b.ret
                        end

                        # Compile the Else Block
                        else_func = func.basic_blocks.append "else" do | b |
                            b.call @builtin_printfunc, [builder.global_string_pointer("false")]
                            b.ret
                        end

                        builder.cond condition.not_nil!.to_unsafe, if_block, else_func
                    end
                end

                add_builtin_func("__print_s", print_string)
                print_funcs[@string_type.pointer] = print_string
                print_funcs[@string_type] = print_string

                add_builtin_func("__print_int", print_int)
                print_funcs[context.int32] = print_int

                add_builtin_func("__print_bool", print_bool)
                print_funcs[context.int1] = print_bool
            end

            # Register builtin function
            def add_builtin_func(name : String, func : LLVM::Function)
                @builtin_funcs[name] = func
            end

        end
    end
end