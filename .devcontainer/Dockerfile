FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive && apt-get -y install coinor-libcbc-dev clang tmux

COPY --from=denoland/deno:bin-2.3.5 /deno /usr/local/bin/deno

