require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a If node 
            @[CompilerVisitor(node: AST::If)]
            class If < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::If)
                    if node.nil?
                        return
                    end

                    # Save old builder
                    old_builder = compiler.builder

                    # Compile the condition
                    condition = compiler.compile node.condition

                    # Create the after block that will run after the If/Else
                    after_block = compiler.current_func.not_nil!.basic_blocks.append "after" do | builder |
                        compiler.builder = builder
                    end

                    # Compile the If Block
                    if_block = compiler.current_func.not_nil!.basic_blocks.append "if" do | builder |
                        old = compiler.builder
                        compiler.builder = builder
                        compiler.compile node.consequence
                        if !compiler.has_return
                            compiler.builder.br after_block
                        end
                        compiler.builder = old
                    end

                    # Compile the Else Block
                    if !node.alternative.nil?
                        old = compiler.builder
                        else_func = compiler.current_func.not_nil!.basic_blocks.append "else" do | builder |
                            compiler.builder = builder
                            compiler.compile node.alternative.not_nil!
                            if !compiler.has_return
                                builder.br after_block
                            end
                        end
                        old_builder.cond condition.not_nil!.to_unsafe, if_block, else_func
                        compiler.builder = old
                        return
                    end

                    old_builder.cond condition.not_nil!.to_unsafe, if_block, after_block
                    end

            end
        end
    end
end