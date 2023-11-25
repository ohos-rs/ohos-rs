FROM openjdk:17-jdk-slim

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

# set env
ENV SDK_MANAGER /harmony/command-line-tools/bin

ENV PATH="${SDK_MANAGER}:${PATH}"

RUN mkdir sdk
RUN sdkmgr install --component-file="/harmony/component.txt" --sdk-directory="/harmony/sdk"