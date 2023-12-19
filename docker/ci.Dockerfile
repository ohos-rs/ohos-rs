FROM --platform=linux/x86_64 openjdk:17-jdk-slim

# install rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
# create dir
RUN mkdir /harmony
WORKDIR /harmony

# copy command line to os
COPY ./commandline-tools-linux-2.0.0.2.zip /harmony
COPY ./component.txt /harmony

# unzip command line
RUN apt-get update && apt-get install -y unzip && unzip ./commandline-tools-linux-2.0.0.2.zip
RUN apt-get install -y locales locales-all curl

# nvm and node.js
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
RUN nvm

# set env
ENV SDK_MANAGER /harmony/command-line-tools/bin

ENV PATH="${SDK_MANAGER}:${PATH}"
ENV LANG zh_CN.UTF-8

RUN mkdir sdk
RUN locale
RUN sdkmgr install --component-file="/harmony/component.txt" --sdk-directory="/harmony/sdk" --accept-license