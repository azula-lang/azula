require "./visitor"
require "../../ast/*"
require "../compiler"
require "../../types"
require "llvm"

module Azula
    module Compiler
        module Visitors

            # Visit a Array and return the Value.
            @[CompilerVisitor(node: AST::Import)]
            class Import < Visitor
                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Import)
                    if node.nil?
                        return
                    end
                    
                    node.imports.each do |import|
                        add_import "src/azula/std/#{import}/#{import}.azl", compiler
                    end
                    return nil
                end

                def add_import(import : String, compiler : Compiler)
                    content = File.read import
                    l = Azula::Lexer.new content
                    l.file = import
                    p = Azula::Parser.new l
                    smt = p.parse_program
    
                    c = Azula::Compiler::Compiler.new
                    c.register_visitors
                    c.compile smt
    
                    c.functions.each do |f|
                        compiler.main_module.functions.add(f.name, f.args, f.return_type)
                    end

                    c.structs.each_key do |s|
                        compiler.structs[s] = c.structs[s]
                    end

                    c.struct_fields.each_key do |s|
                        compiler.struct_fields[s] = c.struct_fields[s]
                    end
    
                    c.create_object_file "#{import}"
                    compiler.imports << import
                end
            end

        end
    end
end