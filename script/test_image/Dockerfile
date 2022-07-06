FROM rust:alpine
    RUN mkdir /core && mkdir /output && mkdir /tests && mkdir /program
    RUN apk add musl-dev

    COPY alsit-testing/ /core

    WORKDIR /core
    RUN cargo build --release

    CMD ["cargo", "run", "--release"]