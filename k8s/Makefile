KUSTOMIZE:=kustomize
KUBECTL:=kubectl -n applications
KUBESEAL:=kubeseal -n applications

K8S_YAMLS:=sealed-secret.yml kustomization.yml

%/sealed-secret.yml: %/secret.yml
	$(KUBESEAL) -o=yaml < $< > $@

%/kustomized.yml: $(addprefix %/,$(K8S_YAMLS))
	$(KUSTOMIZE) build $(dir $@) > $@

all: test/kustomized.yml prod/kustomized.yml

install-%: %/kustomized.yml
	$(KUBECTL) apply -f $<

uninstall-%: %/kustomized.yml
	$(KUBECTL) delete -f $<

clean-%: %/
	rm $<kustomized.yml $<sealed-secret.yml || true

clean: clean-test clean-prod