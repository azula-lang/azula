require "./spec_helper"

describe Azula::Compiler do

    describe "while" do

        it "basic" do
            input = "
            func main(): int {
                int i = 0;
                while(i < 10) {
                    i = i + 1;
                }
                return i;
            }
            "
            run(input).not_nil!.to_i.should eq 10
        end

    end

end