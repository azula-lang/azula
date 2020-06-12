FROM crystallang/crystal:0.34.0
RUN apt-get update && apt-get install -y llvm-8
COPY . .
RUN crystal build src/azula.cr --release --static

FROM ubuntu:18.04
RUN apt-get update && apt-get install -y clang
COPY --from=0 azula /usr/bin/azula
ENTRYPOINT [ "azula" ]