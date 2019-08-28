require "./spec_helper"

describe Azula::Compiler do

    describe "if" do

        it "basic" do
            input = wrap_main("if(true) { return 5; } else { return 10; }", "int")
            run(input).not_nil!.to_i.should eq 5
        end

        it "boolean" do
            input = wrap_main("bool x = false; if(x) { return 10; } else { return 4; }", "int")
            run(input).not_nil!.to_i.should eq 4
        end

        it "nested" do
            input = wrap_main("
            if(true) {
                if(false) { 
                    return 14;
                } else { 
                    return 20;
                } 
            } else { 
                return 10; 
            }
            ", "int")
            run(input).not_nil!.to_i.should eq 20
        end

    end

end