# FROM rust:1.78-alpine as builder
# RUN apk add --no-cache musl-dev sqlite-static openssl-dev openssl-libs-static pkgconf git libpq-dev

# WORKDIR /wd

# COPY ./Cargo.toml ./Cargo.lock ./

# # trick to skip rebuilding deps.. so bad?!
# RUN mkdir src/
# RUN echo 'fn main() {}' > src/main.rs
# RUN cargo build --release
# RUN rm -rf src
# RUN rm -rf target
# COPY ./src ./src
# RUN cargo build --release

FROM scratch

# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
# COPY --from=builder /wd/target/release/testservice .

COPY ./target/x86_64-unknown-linux-musl/release .
CMD ["/testservice"]
