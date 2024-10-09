FROM rust:1.81 AS tools

RUN cargo install wasm-pack

RUN cargo install microserver

FROM tools AS build
EXPOSE 9000
COPY --from=tools /usr/local/cargo/bin/microserver /bin/microserver
COPY --from=tools /usr/local/cargo/bin/wasm-pack /bin/wasm-pack

COPY ./frontend/ ./frontend/
COPY ./protocol/ ./protocol/

RUN ls 
RUN ls frontend
RUN ls protocol

RUN /bin/wasm-pack build --target web --out-name wasm_client frontend/

FROM build as result
WORKDIR ./app
COPY --from=build /bin/microserver /bin/microserver
COPY --from=build /frontend/pkg/ ./pkg
COPY --from=build /frontend/index.html ./index.html
RUN ls

CMD ["/bin/microserver", "--port", "9000"]
