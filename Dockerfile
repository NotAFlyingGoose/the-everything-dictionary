FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_ENV=production
ENV ROCKET_PORT=$PORT

WORKDIR /app
COPY . .

RUN cargo install --path .

CMD ["the-everything-dictionary"]