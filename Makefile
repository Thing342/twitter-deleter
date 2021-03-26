CARGO:=cargo
DOCKER:=docker

PWD:=$(shell pwd)

APP_NAME:=twitter-deleter
$(APP_NAME):
	$(CARGO) build

#----

IMG_REPO:=docker-registry.wesj.app
IMG_OWNER:=wesjorg
IMG_TITLE:=$(APP_NAME)
IMG_VERSIONS:=0.1.0 0.1 0 latest
IMG_TAGS:=$(addprefix $(IMG_REPO)/$(IMG_OWNER)/$(IMG_TITLE):,$(IMG_VERSIONS))

IMG_BUILD_ARGS:=\
	RUSTC_VERSION=1.50 \
	APP_VERSION=0.1.0
container:
	$(DOCKER) build $(addprefix -t ,$(IMG_TAGS)) $(addprefix --build-arg ,$(IMG_BUILD_ARGS)) .

container-push:
	$(DOCKER) push -a $(IMG_REPO)/$(IMG_OWNER)/$(IMG_TITLE)

DOCKER_TEST_ENV:=\
	DRY_RUN=true\
	DAYS_TO_KEEP=2\
	SECRETS_DIR=/secrets

DOCKER_TEST_VOLUMES:=\
	$(PWD)/secrets:/secrets:ro

DOCKER_TEST_ARGS:= \
	$(addprefix -v ,$(DOCKER_TEST_VOLUMES)) \
	$(addprefix -e ,$(DOCKER_TEST_ENV)) \
	$(word 1, $(IMG_TAGS))

container-test: container
	$(DOCKER) run --rm $(DOCKER_TEST_ARGS)

container-test-shell: container
	$(DOCKER) run --rm -it --entrypoint bash $(DOCKER_TEST_ARGS)

#---

install-k8s: container-push
	$(MAKE) -C k8s install-prod

uninstall-k8s:
	$(MAKE) -C k8s uninstall-prod

#---

clean:
	$(CARGO) clean
	$(DOCKER) rmi $(IMG_TAGS)
	$(DOCKER) system prune
	$(MAKE) -c k8s clean