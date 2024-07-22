FROM --platform=linux/amd64 debian:stable

ARG CI_NDK_VERSION=4.1
ENV NDK_VERSION=${CI_NDK_VERSION}

ENV LANG=en_US.utf8 \
	RUSTUP_DIST_SERVER="https://rsproxy.cn" \
	RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup" \
	OHOS_NDK_HOME=/root/sdk \
	RUSTUP_HOME=/usr/local/rustup \
	CARGO_HOME=/usr/local/cargo \
	PATH=/usr/local/cargo/bin:$PATH 

COPY ./docker/config ~/.cargo/config

# rust version should >= 1.78.0
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	locales \
	pkg-config \
	ca-certificates \
	unzip \
	curl \
	openssl \
	libssl-dev \
	git \
	gcc \
    gcc-multilib \
	&& mkdir ~/harmony \
	&& mkdir ~/sdk \
	&& cd ~/harmony \
	&& localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8 \
	&& curl -O https://repo.huaweicloud.com/openharmony/os/${NDK_VERSION}-Release/ohos-sdk-windows_linux-public.tar.gz \
	&& mkdir ./ohos-sdk-windows_linux-public && tar -zxvf ./ohos-sdk-windows_linux-public.tar.gz -C ./ohos-sdk-windows_linux-public \
	&& cd ~/harmony/ohos-sdk-windows_linux-public/ohos-sdk/linux && unzip -u '*.zip' -d ~/sdk \
	&& cd ~ && rm -rf ~/harmony \
	&& curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && rustup target add aarch64-unknown-linux-ohos \
	&& rustup target add armv7-unknown-linux-ohos \
	&& rustup target add x86_64-unknown-linux-ohos
