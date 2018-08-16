
WORKDIR := $(shell pwd)
DATE    := $(shell date +%s)
VERSION := v1.0.12

test:
	cargo test

run:
	cargo run --release -- run-server $(WORKDIR)/srtm90/processed

build-images:
	# Build the builder image, all tools needed to compile server
	docker build . -t milesg/elevation-api-builder:latest --file ./Dockerfile-Builder

	# Run it on current code base, will deposit the executable into $(pwd)/target/release
	docker run --rm -v $(WORKDIR):/build/ milesg/elevation-api-builder:latest

	# Build server, basic installs and copying over the executable
	docker build . -t milesg/elevation-api-server:$(VERSION) --file ./Dockerfile-Server --build-arg DONTCACHE=$(DATE)
	echo "Built server, run with : 'docker run --rm -d -p 8000:8000 -v <netcdf data dir>:/data/ milesg/elevation-api-server:$(VERSION)'"

start-docker-server:
	docker run --rm -d -p 8000:8000 -v $(WORKDIR)/srtm90/processed:/data/ milesg/elevation-api-server:$(VERSION)

stop-docker-server:
	docker rm $(shell docker ps -aq) -f

deploy:
	@echo "$(DOCKER_PASSWORD)" | docker login -u "milesg" --password-stdin
	docker push milesg/elevation-api-server:$(VERSION)

sync-90m_files:
	rsync -e "ssh -i "$(PEM-KEY)"" -az --progress --rsync-path="sudo rsync" \
	 $(WORKDIR)/90m_files/processed/ ec2-user@$(REMOTE-HOST):/home/ec2-user/efs/srtm90

build-deploy:
	cd beanstalk && zip -r deploy-$(VERSION).zip . && cd ..