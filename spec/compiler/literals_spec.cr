require "./spec_helper"

describe Azula::Compiler do

    describe "literals" do

        it "int" do
            input = wrap_main("println(5);", "void")
            run(input).not_nil!.to_i.should eq 5
        end

        it "float" do
            input = wrap_main("println(5.234);", "void")
            # Floats are being awkward
            run(input).not_nil!.should eq "5.234000"
        end

        it "string" do
            input = wrap_main("string s = \"hi\"; println(s);", "void")
            compile_and_run(input).should eq "hi"
        end

        it "boolean" do
            input = wrap_main("println(true);", "void")
            run(input).not_nil!.should eq "true"

            input = wrap_main("println(false);", "void")
            run(input).not_nil!.should eq "false"
        end

    end

end