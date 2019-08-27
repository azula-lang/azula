require "./spec_helper"

describe Azula::Compiler do

    it "compile code" do
        run("func main(): int { return 5; }").not_nil!.to_i.should eq 5
    end

    it "return nil" do
        typeof(run("func main(): void { return; }").not_nil!.to_pointer).should eq Pointer(Void)
    end

end