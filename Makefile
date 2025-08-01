BINARY=target/release/passwordless-auth
WORKER=target/release/email-worker

.PHONY: all build fmt lint test docker docker-up clean

all: build

build:
	cargo build --release
	cargo build --release --bin email-worker

fmt:
	cargo fmt

lint:
	cargo clippy --all-features -- -D warnings

test:
	# run unit & integration tests
	cargo test

docker-build:
	docker build -t passwordless-auth:latest .

docker-up:
	docker compose up --build

clean:
	cargo clean
	rm -f auth.db

