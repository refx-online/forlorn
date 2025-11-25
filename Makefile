run:
	docker run \
		--network=host \
		--env-file=.env \
		-v refxpy_data:/srv/root/.data \
		-it forlorn:latest

run-bg:
	docker run \
		--network=host \
		--env-file=.env \
		-v refxpy_data:/srv/root/.data \
		-d forlorn:latest

build:
	DOCKER_BUILDKIT=1 docker build -t forlorn:latest .

fmt:
	cargo +nightly fmt --all -- --emit=files
