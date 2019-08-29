require "../spec_helper"

describe Azula::Compiler do

    describe "std" do

        describe "print" do

            it "string" do
                input = wrap_main("string s = \"hi\"; print(s);", "void")
                compile_and_run(input).should eq "hi\n"
            end

            # it "int" do
            #     input = wrap_main("print(5);", "void")
            #     compile_and_run(input).should eq "5\n"
            # end

        end

    end

end