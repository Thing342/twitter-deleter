ARG RUSTC_VERSION="1.50"
ARG RUSTC_IMAGE="rust"
ARG APP_VERSION="latest"

ARG DEPLOY_IMAGE="debian:bullseye-slim"

FROM $RUSTC_IMAGE:$RUSTC_VERSION as build

WORKDIR /work
ADD Cargo.* ./
ADD src ./src
RUN cargo build --release

# Main image
FROM $DEPLOY_IMAGE

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get upgrade && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=build /work/target/release/twitter-deleter /usr/bin/twitter-deleter
COPY LICENSE /LICENSE
COPY dependency-licenses.txt /rust-dependency-licenses.txt

LABEL maintainer=wes@wesj.org
LABEL version=$APP_VERSION
LABEL license=agplv3
LABEL org.wesj.build_image=$RUSTC_IMAGE:$RUSTC_VERSION
LABEL org.wesj.depoy_image=$DEPLOY_IMAGE

CMD ["/usr/bin/twitter-deleter"]