require "./visitor"
require "../../ast/*"
require "../compiler"
require "llvm"

module Azula
    module Compiler
        module Visitors

            # Visit a While statement
            @[CompilerVisitor(node: AST::While)]
            class While < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::While)
                    if node.nil?
                        return
                    end

                    old_builder = compiler.builder
                    loop_cond_builder = compiler.builder
                    cond : LLVM::Value? = nil

                    after_block = compiler.current_func.not_nil!.basic_blocks.append "after" do |builder|
                        compiler.builder = builder
                    end

                    loop_cond = compiler.current_func.not_nil!.basic_blocks.append "loop-cond" do |builder|
                        old = compiler.builder
                        compiler.builder = builder
                        cond = compiler.compile node.iterator
                        loop_cond_builder = builder
                        compiler.builder = old
                    end

                    old_loop_cond = compiler.current_loop_cond
                    compiler.current_loop_cond = loop_cond

                    loop_block = compiler.current_func.not_nil!.basic_blocks.append "loop" do |builder|
                        old = compiler.builder
                        compiler.builder = builder
                        compiler.compile node.body
                        compiler.builder.br loop_cond
                        compiler.builder = old
                    end

                    if !cond.nil?
                        loop_cond_builder.cond cond, loop_block, after_block
                    end

                    old_builder.br loop_cond

                    compiler.current_loop_cond = old_loop_cond

                    return
                end

            end
        end
    end
end