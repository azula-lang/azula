require "./parser/lexer"
require "./parser/parser"
require "./error"
require "./compiler/typechecker"
require "./compiler/compiler"

class Azula::Program

    getter errors

    @root_node : AST::Node?
    @compiler : Azula::Compiler

    def initialize(@file : String, @code : String)
        @lexer = Azula::Lexer.new @code
        @parser = Azula::Parser.new @lexer
        @errors = [] of Azula::Error
        @root_node = nil
        @compiler = Azula::Compiler.new
    end

    def parse
        @root_node = @parser.parse_program
        @errors = @errors + @parser.errors
    end

    def typecheck : Bool
        if @root_node.nil?
            return false
        end
        typechecker = Azula::Typechecker.new
        typechecker.check(@root_node.not_nil!)
        @errors = @errors + typechecker.errors
        return typechecker.errors.size == 0
    end

    def compile
        if @root_node.nil?
            return
        end

        @compiler.compile @root_node.not_nil!
        @errors = @errors + @compiler.errors
        if @errors.size == 0
            @compiler.main_module.print_to_file "test.ll"
        end
    end

    def optimise
        fun_pass_manager = @compiler.main_module.new_function_pass_manager
        pass_manager_builder.populate fun_pass_manager
        fun_pass_manager.run @compiler.main_module
        module_pass_manager.run @compiler.main_module
    end

    def create_executable(file : String, static : Bool)
        LLVM.init_x86

        if !Dir.exists? ".build"
            Dir.mkdir ".build"
        end

        target = LLVM::Target.from_triple(LLVM.default_target_triple)
        machine = target.create_target_machine LLVM.default_target_triple
        machine.emit_obj_to_file @compiler.main_module, ".build/temp.o"

        system "clang -o #{file} .build/temp.o #{static ? "-static" : ""}"

        File.delete ".build/temp.o"
        Dir.rmdir ".build"
    end

    def write_llvm(file : String)
        @compiler.main_module.print_to_file file
    end

    @module_pass_manager : LLVM::ModulePassManager?

    private def module_pass_manager
        @module_pass_manager ||= begin
            mod_pass_manager = LLVM::ModulePassManager.new
            pass_manager_builder.populate mod_pass_manager
            mod_pass_manager
        end
    end

    @pass_manager_builder : LLVM::PassManagerBuilder?

    private def pass_manager_builder
        @pass_manager_builder ||= begin
            registry = LLVM::PassRegistry.instance
            registry.initialize_all

            builder = LLVM::PassManagerBuilder.new
            builder.opt_level = 3
            builder.size_level = 0
            builder.use_inliner_with_threshold = 275
            builder
        end
    end

end