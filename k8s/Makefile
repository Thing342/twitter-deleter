KUSTOMIZE:=kustomize
KUBECTL:=kubectl -n applications
KUBESEAL:=kubeseal -n applications

K8S_YAMLS := cronjob.yml pvc.yml secret-sealed.yml kustomization.yml

secret-sealed.yml: secret.yml
	$(KUBESEAL) -o=yaml < $< > $@

kustomized.yml: $(K8S_YAMLS)
	$(KUSTOMIZE) build > $@

all: kustomized.yml

install: kustomized.yml
	$(KUBECTL) apply -f $<

uninstall: kustomized.yml
	$(KUBECTL) delete -f $<

clean:
	rm kustomized.yml secret-sealed.yml || true