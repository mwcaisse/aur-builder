OWNER=mwcaisse
GIT_REPO=application-images
IMAGE_NAME=arch-aur-builder
VERSION=1.2
TAG=registry.gitlab.com/$(OWNER)/$(GIT_REPO)/$(IMAGE_NAME):$(VERSION)

all: push

build:
	docker build --no-cache -t $(TAG) .

push: build
	docker push $(TAG)
