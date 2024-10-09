FROM rust:1.81 AS tools

RUN cargo install wasm-pack

RUN cargo install microserver

FROM tools as result
EXPOSE 9000

COPY ./ ./
RUN wasm build --target web --out-name wasm_client


CMD ["microserver", "--port", "9000"]
