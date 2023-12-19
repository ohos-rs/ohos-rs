FROM --platform=linux/x86_64 ubuntu:22.04

SHELL ["/bin/bash", "-c"]

# Java
RUN apt-get update && apt-get install openjdk-17-jdk curl -y

# nvm and node.js
RUN curl https://get.volta.sh | bash
ENV VOLTA_HOME="${HOME}/.volta"
ENV PATH="${VOLTA_HOME}/bin:${PATH}"
RUN volta install node@16

# create dir
RUN mkdir /harmony
WORKDIR /harmony

# copy command line to os
COPY ./commandline-tools-linux-2.0.0.2.zip /harmony
COPY ./component.txt /harmony

# unzip command line
RUN apt-get update && apt-get install -y unzip && unzip ./commandline-tools-linux-2.0.0.2.zip

# lang
RUN apt-get install -y language-pack-zh-hans
ENV LANG zh_CN.UTF-8
RUN locale

# install rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# set env
ENV SDK_MANAGER=/harmony/command-line-tools/bin
ENV PATH="${SDK_MANAGER}:${PATH}"

# install sdk
RUN mkdir sdk
RUN node -v
RUN npm -v
RUN sdkmgr install --component-file="/harmony/component.txt" --sdk-directory="/harmony/sdk" --accept-license