FROM rust:1.81 AS build

COPY ./signaling-server/ ./server/
COPY ./protocol/ ./protocol/

RUN cargo build --release --manifest-path server/Cargo.toml

FROM build AS result
WORKDIR ./app

COPY --from=build /server/taget/release/signaling-server signaling-server

CMD ["./signaling-server"]
