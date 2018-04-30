FROM rust:1.25 AS builder

RUN mkdir -p /home/rust
WORKDIR /home/rust
COPY . .
RUN cargo build --release

RUN curl https://s3-eu-west-1.amazonaws.com/fishbrain-codebuild-assets/chamber-v2.0.0-linux-amd64 -o /usr/bin/chamber && chmod +x /usr/bin/chamber

FROM debian:9.4 as runner

RUN apt-get update \
  && DEBIAN_FRONTEND=noninteractive apt-get install -y openssh-server ca-certificates libssl1.1 \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/bin/chamber /usr/bin

COPY docker/sshd_config /etc/ssh/sshd_config
COPY docker/motd /etc/motd
COPY docker/instructions /usr/bin/instructions
RUN chmod 555 /usr/bin/instructions

COPY --from=builder /home/rust/target/release/ssh-auth-github /usr/bin
RUN adduser --disabled-password --shell /usr/bin/instructions --gecos Tunnel,,,, tunnel

COPY --chown=tunnel:tunnel docker/homedir /home/tunnel
RUN chmod 700 /home/tunnel/.ssh

RUN mkdir -p /run/sshd

EXPOSE 22

CMD ["/bin/sh", "-c", "/usr/bin/chamber export $CHAMBER_SERVICE -f /etc/ssh-auth-github.json && /usr/sbin/sshd -D -e"]
