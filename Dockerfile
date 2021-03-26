ARG RUSTC_VERSION="1.50"
ARG RUSTC_IMAGE="rust"
ARG APP_VERSION="0.1.0"

FROM $RUSTC_IMAGE:$RUSTC_VERSION as build

WORKDIR /work
ADD Cargo.* ./
ADD src ./src
RUN cargo build --release

# Main image
FROM debian:bullseye-slim

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get upgrade && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=build /work/target/release/twitter-deleter /usr/bin/twitter-deleter
CMD ["/usr/bin/twitter-deleter"]