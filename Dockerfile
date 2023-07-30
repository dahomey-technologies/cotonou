# Setup
FROM clux/muslrust:stable AS setup 
RUN cargo install cargo-chef 
WORKDIR /src

FROM setup as prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Cache
FROM setup as cook
COPY --from=prepare /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build
FROM cook as build
COPY . .
RUN cargo build --release

# Runtimes
FROM scratch AS runtime-auth
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/cotonou-auth /
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
CMD ["/cotonou-auth"]

FROM scratch AS runtime-notif
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/cotonou-notif /
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
CMD ["/cotonou-notif"]

FROM scratch AS runtime-mms
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/cotonou-matchmaking-service /
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
CMD ["/cotonou-matchmaking-service"]