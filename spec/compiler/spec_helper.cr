require "spec"

require "../../src/azula/parser"
require "../../src/azula/compiler"
require "../../src/azula/lexer"

require "llvm"

def run(code : String)
    return compile_and_run code
end

def compile_and_run(code : String)
    # l = Azula::Lexer.new code
    # l.file = "tests"
    # p = Azula::Parser.new l
    # smt = p.parse_program


    code = "package \"tests\";" + code
    # Setup error manager
    Azula::ErrorManager.set_file code.split("\n")

    l = Azula::Lexer.new code
    l.file = "tests"
    p = Azula::Parser.new l
    smt = p.parse_program

    if !Azula::ErrorManager.can_compile
        Azula::ErrorManager.print_errors
        1.should eq 2
    end

    t = Azula::Types::Typechecker.new
    t.check smt

    c = Azula::Compiler::Compiler.new
    c.register_visitors
    c.compile smt

    c.create_executable "tests_temp"
    stdout = IO::Memory.new
    stderr = IO::Memory.new
    status = Process.run("./tests_temp", output: stdout, error: stderr)
    File.delete "tests_temp"
    output = stdout.to_s.gsub("\n", "")
    return output[0..output.size-2]
end

def wrap_main(input : String, return_type : String) String
    return "func main(): #{return_type} {#{input}}"
end