FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_ENV=production

CMD ROCKET_PORT=$PORT ./target/release/rocket