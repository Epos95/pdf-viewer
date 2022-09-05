FROM debian:buster-slim
RUN apt-get update && apt-get install -y pdftk imagemagick

FROM rust:1.63

COPY ./ ./
RUN cargo build

CMD ["convert"]
CMD ["./target/debug/pdf_axum_test"]
