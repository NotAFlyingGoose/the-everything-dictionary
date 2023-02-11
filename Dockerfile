FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_ENV=production

WORKDIR /app
COPY . .

RUN cargo install --path .

CMD ROCKET_PORT=$PORT /usr/local/cargo/bin/the-everything-dictionary