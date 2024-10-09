FROM rust:1.81 AS tools

RUN cargo install wasm-pack

RUN cargo install microserver

FROM tools AS result
EXPOSE 9000
COPY --from=tools /usr/local/cargo/bin/microserver /bin/microserver
COPY --from=tools /usr/local/cargo/bin/wasm-pack /bin/wasm-pack

COPY ./ ./
RUN /bin/wasm-pack build --target web --out-name wasm_client ./frontend/Cargo.toml


CMD ["/bin/microserver", "--port", "9000"]
