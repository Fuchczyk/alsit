FROM rust:alpine
    # Create structure and install deps
    RUN mkdir /server && mkdir /exercises
    RUN apk add musl-dev docker jq

    # Copy server files
    COPY . /server/

    # Compile server
    WORKDIR /server
    RUN sh script/build/compile.sh

    # Run compiled server
    CMD ["sh", "alsit.sh", "indocker_run"]