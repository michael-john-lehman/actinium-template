FROM rust:1.90.0-alpine AS build

RUN apk add --no-cache musl-dev

WORKDIR /dev-server

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src

RUN touch src/main.rs

# Build the real application
RUN cargo build --release

FROM debian:bullseye-slim AS runtime

# Server User
RUN useradd -ms /bin/bash server-user

COPY --from=build /dev-server/target/release/backend /server

USER server-user

EXPOSE 8000

ENTRYPOINT [ "/server" ]