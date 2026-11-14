FROM node:24-trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends asciinema ca-certificates curl git python3 zip unzip procps && rm -rf /var/lib/apt/lists/*
COPY sigma /usr/local/bin/sigma
RUN useradd --uid 1001 --create-home demo
USER demo
ENV PATH="/home/demo/.local/bin:${PATH}" NPM_CONFIG_PREFIX="/home/demo/.local" TERM="xterm-256color" COLORTERM="truecolor"
WORKDIR /home/demo/work
CMD ["sleep", "infinity"]
