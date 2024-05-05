
run-as-sidecar:
	cargo build --target x86_64-unknown-linux-musl --release
	cd testservice && cargo build --target x86_64-unknown-linux-musl --release
	docker compose up --build

