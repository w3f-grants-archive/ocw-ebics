# Contains commands to launch the chain and the demo app
build:
	@echo "Building..."
	@cargo build --release
run-tests:
	@echo "Running tests..."
	@cargo test -- --nocapture
launch-chain:
	@echo "Launching chain..."
	@cargo run --release -- --dev --tmp
launch-demo-app:
	@echo "Launching demo app..."
	@cargo run --release -- --dev --tmp & yarn --cwd ebics-demo start
