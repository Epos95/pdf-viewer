FROM rust:1.69

WORKDIR /pdf-viewer

COPY . .

RUN cargo build --release -j 2 --bin pdf-viewer

CMD ["./target/release/pdf-viewer"]
