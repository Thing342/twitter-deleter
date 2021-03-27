# Twitter Deleter
Author: Wes Jordan (wes@wesj.org)

A bot that automatically deletes older Twitter posts so you can live In The Moment.

Deleted tweets are written to stdout as JSON documents so they can be preserved in your private collection.

## Building
To build the binary:
```shell
make twitter-deleter
```
To build (and push) the container:
```shell
make container-push
```
**Note: you should adjust the `IMG_REPO` variable in the Makefile to push to your repository instead of mine.**

## Installing
`twitter-deleter` is a simple Rust binary that runs once and deletes all posts older than a certain cutoff.
Install it by building the binary for your platform (`cargo build --release`) and setting up a cronjob for it to run automatically.

`twitter-deleter` can be configured using the following environment variables:
* `DRY_RUN` - shows the Tweets it _would_ delete without affecting anything. 
    **This is by default set to `true` to promote discretion.**
* `DAYS_TO_KEEP` - number of days of Tweets to keep when it runs.
    **Default: `30`**
* `SECRETS_DIR` - Folder where your Twitter secrets are stored.
  **Default: `./secrets`**
  - `twitter-deleter` expects the directory to contain the following files:
      - access_token
      - access_token_secret 
      - consumer_secret     
      - consumer_token       
      - username
    
`twitter-deleter` prints the tweets it deletes to stdout.
So, you should probably configure cron to redirect its stdout to a log file:
```shell
DRY_RUN=false DAYS_TO_KEEP=7 SECRETS_DIR=/secrets/twitter-creds twitter-deleter >> /logs/deleted-tweets.log
```

### As a Kubernetes CronJob
This software can be packaged in a container (and is licensed with the AGPL) so you can deploy it as a CronJob in Kubernetes.
The easiest way to do this is to copy `k8s/example/kustomization.yml` and use Kustomize to generate personalized YAML definitions for your cluster, e.g:
```shell
kustomize build . | kubectl apply -f -
```
This project's kustomization adds a PersistentVolumeClaim for storage of logged tweets between runs.
The CronJob definition expects this PVC to be called `logs-data` if you wish to add your own.

I have this entire process automated for my home cluster using
```shell
make install-k8s
```
which builds and pushes the container, builds kustomizations and applies them to the cluster automatically.
Feel free to modify the Makefile and Kustomizations to make this work on your own cluster.