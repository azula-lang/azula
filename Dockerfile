FROM ubuntu:18.04

RUN apt-get update && apt-get install -y gnupg2 curl
RUN curl -sL "https://keybase.io/crystal/pgp_keys.asc" | apt-key add -
RUN echo "deb https://dist.crystal-lang.org/apt crystal main" | tee /etc/apt/sources.list.d/crystal.list
RUN apt-get update && apt-get install -y crystal clang-8
RUN ln -s /usr/bin/clang-8 /usr/local/bin/clang
RUN ln -s /usr/bin/llvm-config-8 /usr/local/bin/llvm-config
WORKDIR /azula
COPY . .
RUN ./install.sh
WORKDIR /azula/wd
ENTRYPOINT ["azula"]