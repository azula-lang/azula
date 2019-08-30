require "./spec_helper"

describe Azula::Compiler do

    describe "structs" do

        it "basic" do
            input = "
            struct API {
                string version,
            }

            func new_api(string version): API {
                return API{version};
            }

            func print_api(API api): void {
                print(api.version);
            }

            func main(): void {
                API api = new_api(\"0.10.0\");

                print_api(api);
            }
            "
            compile_and_run(input).should eq "0.10.0"
        end

    end

end