# Setup
FROM --platform=x86_64 rust:slim AS setup 
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo install cargo-chef 
WORKDIR /src

FROM setup as prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Cache
FROM setup as cook
COPY --from=prepare /src/recipe.json recipe.json
RUN cargo chef cook --target x86_64-unknown-linux-musl --release --recipe-path recipe.json

# Build
FROM cook as build
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

# Runtimes
FROM scratch AS runtime-auth
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/cotonou-auth /
CMD ["/cotonou-auth"]

FROM scratch AS runtime-notif
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/cotonou-notif /
CMD ["/cotonou-notif"]

FROM scratch AS runtime-mms
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/cotonou-matchmaking-service /
CMD ["/cotonou-matchmaking-service"]