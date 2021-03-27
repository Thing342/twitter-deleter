CARGO:=cargo
DOCKER:=docker

PWD:=$(shell pwd)

GIT_SHA:=$(shell git rev-parse --short HEAD)

LICENSE:=$(shell cat LICENSE)
print-license: LICENSE
	cat $^

check-license: Cargo.lock LICENSE
	$(CARGO) lichking check

dependency-licenses.txt: Cargo.lock LICENSE
	$(CARGO) lichking bundle --file $@ || true

#----

APP_NAME:=twitter-deleter
$(APP_NAME): print-license check-license dependency-licenses.txt
	$(CARGO) build

#----

IMG_REPO:=docker-registry.wesj.app
IMG_OWNER:=wesjorg
IMG_TITLE:=$(APP_NAME)
IMG_VERSIONS:=1.0.0.0 1.0.0 1.0 1 latest $(GIT_SHA)
IMG_TAGS:=$(addprefix $(IMG_REPO)/$(IMG_OWNER)/$(IMG_TITLE):,$(IMG_VERSIONS))

IMG_BUILD_ARGS:=\
	RUSTC_VERSION=1.50 \
	APP_VERSION=1.0.0.0
container: print-license check-license dependency-licenses.txt
	$(DOCKER) build $(addprefix -t ,$(IMG_TAGS)) $(addprefix --build-arg ,$(IMG_BUILD_ARGS)) .

container-push: container
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