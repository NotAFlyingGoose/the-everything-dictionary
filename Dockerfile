FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_ENV=production
ENV ROCKET_PORT=80

WORKDIR /app
COPY . .

RUN cargo install --path .

CMD ["the-everything-dictionary"]