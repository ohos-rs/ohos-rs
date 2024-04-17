FROM --platform=linux/x86_64 debian:stable

ENV LANG=en_US.utf8 \
	RUSTUP_DIST_SERVER="https://rsproxy.cn" \
	RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup" \
	OHOS_NDK_HOME=/root/sdk \
	PATH="/root/.cargo/bin:${PATH}"

ADD ./config ~/.cargo/config

RUN mkdir ~/harmony && mkdir ~/sdk \
	&& cd ~/harmony \
	&& apt-get update \
	&& apt-get install -y --no-install-recommends locales \
	ca-certificates \
	unzip \
	curl \
	libssl-dev \
	gcc \
    gcc-multilib \
	&& rm -rf /var/lib/apt/lists/* \
	&& localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8 \
	&& curl -O https://repo.huaweicloud.com/openharmony/os/4.0-Release/ohos-sdk-windows_linux-public.tar.gz \
	&& mkdir ./ohos-sdk-windows_linux-public && tar -zxvf ./ohos-sdk-windows_linux-public.tar.gz -C ./ohos-sdk-windows_linux-public \
	&& cd ~/harmony/ohos-sdk-windows_linux-public/ohos-sdk/linux && unzip -u '*.zip' -d ~/sdk \
	&& cd ~ && rm -rf ~/harmony \
	&& curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && rustup default nightly && rustup component add rust-src
