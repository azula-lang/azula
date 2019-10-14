require "./spec_helper"

describe Azula::Compiler do

    describe "literals" do

        it "int" do
            input = wrap_main("return 5;", "int")
            run(input).not_nil!.to_i.should eq 5
        end

        it "float" do
            input = wrap_main("return 5.234;", "float")
            # Floats are being awkward
            run(input).not_nil!.to_f64.to_f32.should eq 5.234.to_f32
        end

        it "string" do
            input = wrap_main("string s = \"hi\"; println(s);", "void")
            compile_and_run(input).should eq "hi"
        end

        it "boolean" do
            input = wrap_main("return true;", "bool")
            run(input).not_nil!.to_b.should be_true

            input = wrap_main("return false;", "bool")
            run(input).not_nil!.to_b.should be_false
        end

    end

end