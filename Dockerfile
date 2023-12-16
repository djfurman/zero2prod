# Use the latest Rust stable release as builder
FROM rust:1.74.1 as builder

# Switch working directory to `/opt/app` (same as `cd /opt/app`)
# The `/opt/app` folder will be created by the container in case it does not already exist
WORKDIR /opt/app

# Install the required system dependencies for our linking configuration
RUN apt-get update -y \
    && apt-get install lld clang -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates

# Copy all files from the working environment to the image
COPY . .

# Force SQLX to Build Offline
ENV SQLX_OFFLINE true

# Build the binary
RUN cargo build --release

# Use a multistage build to create an execution envionrment
FROM debian:bookworm-slim as runtime

# Switch to Linux standard /opt directory for application instsall
WORKDIR /opt/app

# Update OpenSSL and the certificate store for TLS verifications
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt-lists/*

ADD https://truststore.pki.rds.amazonaws.com/global/global-bundle.pem /opt/app/global-bundle.pem
COPY --from=builder /opt/app/target/release/zero2prod zero2prod
COPY configuration configuration

# Set Docker to the Production Environment
ENV APP_ENVIRONMENT production

# Run this when `docker run` is executed to launch the binary
ENTRYPOINT [ "./zero2prod" ]
