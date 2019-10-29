require "./spec_helper"

describe Azula::Compiler do

    describe "if" do

        it "basic" do
            input = wrap_main("if(true) { println(5); } else { println(10); }", "void")
            run(input).not_nil!.to_i.should eq 5
        end

        it "boolean" do
            input = wrap_main("bool x = false; if(x) { println(10); } else { println(4); }", "void")
            run(input).not_nil!.to_i.should eq 4
        end

        it "nested" do
            input = wrap_main("
            if(true) {
                if(false) { 
                    println(14);
                } else { 
                    println(20);
                } 
            } else { 
                println(10); 
            }
            ", "void")
            run(input).not_nil!.to_i.should eq 20
        end

    end

end