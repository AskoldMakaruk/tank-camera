FROM rust:1.81 AS tools

RUN cargo install wasm-pack

RUN cargo install microserver

RUN rustup target add wasm32-unknown-unknown

FROM tools AS build
COPY --from=tools /usr/local/cargo/bin/microserver /bin/microserver
COPY --from=tools /usr/local/cargo/bin/wasm-pack /bin/wasm-pack

COPY ./frontend/ ./frontend/
COPY ./protocol/ ./protocol/

RUN /bin/wasm-pack build --target web --out-name wasm_client frontend/

FROM build AS result
EXPOSE 9000
WORKDIR /app
COPY --from=build /bin/microserver /bin/microserver
COPY --from=build /frontend/pkg/ ./pkg
COPY --from=build /frontend/index.html ./index.html

CMD ["/bin/microserver", "--port", "9000"]
