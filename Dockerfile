FROM rust:1.76

RUN apt-get update && apt-get install -y curl
ENV NODE_VERSION=16.19.1
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"

COPY ./ ./

WORKDIR ./webuntis-node-test
RUN npm install

WORKDIR .. 

RUN cargo build --release
CMD ["./target/release/autosign2"]
