FROM rust:1.81 AS tools

RUN cargo install wasm-pack

RUN cargo install microserver

FROM tools as result
EXPOSE 9000
COPY --from=tools /usr/local/cargo/bin/microserver /bin/microserver
COPY --from=tools /usr/local/cargo/bin/wasm-pack /bin/wasm-pack

COPY ./ ./
RUN /bin/wasm-pack build --target web --out-name wasm_client


CMD ["/bin/microserver", "--port", "9000"]
