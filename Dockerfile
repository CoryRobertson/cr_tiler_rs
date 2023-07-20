FROM rust:1.70 as builder
COPY . .
RUN cargo build --package cr_tile_game_service --release

FROM debian:bookworm
COPY --from=builder /target/release/cr_tile_game_service ./cr_tile_game_service
EXPOSE 8114
VOLUME ["/data"]
CMD ["./cr_tile_game_service"]