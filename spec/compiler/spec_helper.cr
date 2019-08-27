require "spec"

require "../../src/azula/parser"
require "../../src/azula/compiler"
require "../../src/azula/lexer"

require "llvm"

def run(code : String) LLVM::GenericValue?
    l = Azula::Lexer.new code
    l.file = "tests"
    p = Azula::Parser.new l
    smt = p.parse_program

    if !p.errors.empty?
        p.errors.each do |error|
            puts error
        end
        return
    end

    c = Azula::Compiler::Compiler.new
    c.register_visitors
    c.compile smt

    LLVM.init_x86
    jit = LLVM::JITCompiler.new c.main_module
    result = jit.run_function(c.main_module.functions["main"], c.context)

    return result
end

def wrap_main(input : String, return_type : String) String
    return "func main(): #{return_type} {#{input}}"
end