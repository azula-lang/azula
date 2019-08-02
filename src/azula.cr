require "./azula/lexer/"
require "./azula/parser/"
require "./azula/types/"
require "./azula/compiler/"

# Azula is a strongly-typed compiled language, using an LLVM backend
module Azula
end

VERSION = "0.3.0"
PROMPT = ">> "

puts "Azula " + VERSION

#print PROMPT
if ARGV.size != 2
    puts "Incorrect number of arguments."
    exit
end
todo = ARGV[0]
file = ARGV[1]
content = File.read file
l = Azula::Lexer.new content
l.file = file
p = Azula::Parser.new l
smt = p.parse_program

if !p.errors.empty?
    p.errors.each do |error|
        puts error
    end
end

# t = Azula::Types::Typechecker.new
# puts t.check smt

# if !t.errors.empty?
#   t.errors.each do |error|
#     puts error
#   end
# end

c = Azula::Compiler.new smt
c.compile

outfile = file.split("/")[file.split("/").size-1].sub(".azl", "")

if todo == "build"
    target = LLVM::Target.from_triple(LLVM.default_target_triple)
    machine = target.create_target_machine LLVM.default_target_triple
    machine.emit_obj_to_file c.main_module, "#{outfile}.o"

    system "clang -o #{outfile} -lstdc++ #{outfile}.o"

    File.delete "#{outfile}.o"

    puts "Compiled to ./#{outfile}"
elsif todo == "run"
    target = LLVM::Target.from_triple(LLVM.default_target_triple)
    machine = target.create_target_machine LLVM.default_target_triple
    machine.emit_obj_to_file c.main_module, "#{outfile}.o"

    system "clang -o #{outfile} -lstdc++ #{outfile}.o"

    File.delete "#{outfile}.o"

    system "./#{outfile}"
elsif todo == "llir"
    c.write_to_file "#{outfile}.ll"
    puts "Wrote LLIR to #{outfile}.ll"
end

