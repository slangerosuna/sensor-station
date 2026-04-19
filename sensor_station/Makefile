WORKSPACE:= /sensor-station
IMAGE:= ghcr.io/iot4ag-hackathon-2025/sensor-station-challenge:latest

repo-init:
	python3 -m pip install pre-commit==3.4.0 && \
	pre-commit install

build-image:
	docker build . -t ${IMAGE} --target prod

clean-build:
	docker build . -t ${IMAGE} --no-cache --target prod

prod:
	docker run -d --rm \
	--net=host \
	${IMAGE}

proto:
	protoc --python_out=app/ protobuf/sensor.proto

clean:
	rm -rf app/protobuf
