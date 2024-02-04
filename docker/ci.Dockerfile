FROM --platform=linux/amd64 docker.io/southorange/ohos-base

RUN cargo install ohrs \
    && ohrs build
