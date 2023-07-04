FROM rust:1.69

EXPOSE 3000

WORKDIR /

COPY . .

RUN cargo build --release -j 3 --bin pdf-viewer

CMD ["./target/release/pdf-viewer", "-p", "3000", "-s", "/state_dir/state.json"]
