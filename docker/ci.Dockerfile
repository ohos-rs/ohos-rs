FROM --platform=linux/amd64 southorange/ohos-base

RUN cargo install ohrs \
    && cd /root \
    && ohrs init hello \
    && cd hello \
    && ohrs build
