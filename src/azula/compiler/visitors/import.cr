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

                    if !Dir.exists?(".build")
                        Dir.mkdir ".build"
                    end
                    
                    node.imports.each do |import|
                        if Dir.exists?("/usr/lib/azula/sources/#{import}/")
                            Dir.glob("/usr/lib/azula/sources/#{import}/*.azl").each do |s|
                                add_import import, "#{s}", compiler
                            end
                        elsif Dir.exists?("./#{import}/")
                            Dir.glob("#{compiler.project_top}/#{import}/*.azl").each do |s|
                                add_import import, "#{s}", compiler
                            end
                        else
                            ErrorManager.add_error Error.new "unknown import \"#{import}\"", node.token.file, node.token.linenumber, node.token.charnumber
                        end
                    end
                    return nil
                end

                def add_import(name : String, import : String, compiler : Compiler)
                    content = File.read import
                    l = Azula::Lexer.new content
                    l.file = import
                    p = Azula::Parser.new l
                    smt = p.parse_program
    
                    c = Azula::Compiler::Compiler.new
                    c.register_visitors
                    c.project_top = compiler.project_top
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
    
                    c.create_object_file "#{compiler.project_top}/.build/#{name}"
                    compiler.imports << name
                end
            end

        end
    end
end