module Azula
    module Compiler
        class Compiler

            # Add the builtin functions
            def add_builtins
                strlen = @main_module.functions.add("strlen", [@context.int8.pointer], @context.int32, true)
                add_builtin_func("strlen", strlen)
                scanf = @main_module.functions.add("scanf", [@context.int8.pointer], @context.void, true)
                add_builtin_func("scanf", scanf)
                add_print_funcs
                input = @main_module.functions.add("input", [@string_type.pointer], @context.int8.pointer) do |func|
                    entry = func.basic_blocks.append "entry" do |builder|
                        store = builder.alloca @context.int8.array(10)
                        ptr = builder.gep store, @context.int32.const_int(0), @context.int32.const_int(0)
                        builder.call scanf, [builder.global_string_pointer("%s"), ptr]
                        builder.ret store
                    end
                end
                string_cstring = @main_module.functions.add("cstring_conv", [@string_type.pointer], @context.int8.pointer, true) do |func|
                    entry = func.basic_blocks.append "entry" do | builder |
                        v = builder.gep func.params[0], @context.int32.const_int(0), @context.int32.const_int(0)
                        val = builder.load v
                        builder.ret val
                    end
                end
                add_builtin_func("cstring_conv", string_cstring)

                # cstring_string = @main_module.functions.add("string_conv", [@context.int8.pointer], @string_type.pointer, true) do |func|
                #     entry = func.basic_blocks.append "entry" do | builder |
                #         old_builder = @builder
                #         @builder = builder
                #         alloca = @builder.alloca @context.int8.pointer
                #         param = func.params[0]
                #         val = @builder.call strlen, [param]
                #         # val = @context.int32.const_int(5)
                #         @builder.store param, alloca
                #         str = @context.const_struct [
                #             alloca,
                #             val,
                #             val,
                #         ]
                #         alloca = @builder.alloca @string_type
                #         @builder.store str, alloca
                #         @builder.ret alloca
                #         @builder = old_builder
                #     end
                # end
                # add_builtin_func("string_conv", cstring_string)

            end

            # Register builtin function
            def add_builtin_func(name : String, func : LLVM::Function)
                @builtin_funcs[name] = func
            end

            # Register the various print functions
            def add_print_funcs
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
                print_float = @main_module.functions.add("__print_float", [@context.double], @context.void, true) do |func|
                    entry = func.basic_blocks.append "entry" do | builder |
                        builder.call @builtin_printfunc, [builder.global_string_pointer("%f"), func.params[0]]
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
                print_funcs[context.int8] = print_int

                add_builtin_func("__print_float", print_float)
                print_funcs[context.double] = print_float

                add_builtin_func("__print_bool", print_bool)
                print_funcs[context.int1] = print_bool
            end

        end
    end
end