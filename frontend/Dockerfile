FROM rust:1.81

COPY ./ ./

RUN cargo install wasm-pack

RUN cargo install microserver

RUN wasm build --target web --out-name wasm_client

EXPOSE 9000

CMD ["microserver", "--port", "9000"]


# todo зменшити розмір контейнера прибрати cargo make
# або зробити через docker compose 

