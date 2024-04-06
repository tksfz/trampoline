
Trampoline is a scalable, Kube-native task orchestrator written in Rust.

Trampoline has two main components:
- the __Dispatcher__ forwards messages from a message queue to a worker
- the __Controller__ provisions and assigns Dispatchers to topics

Aside from Kubernetes, Trampoline depends only on having a message queue available. For now, Trampoline is tightly integrated with [Apache Pulsar](https://pulsar.apache.org/) as its message queue.

# Dev Environment

Use [Nix](https://nixos.wiki/wiki/Development_environment_with_nix-shell) and the included shell.nix file to create a dev environment that includes
cargo, minikube, VS Code, and other tools:

```shell
trampoline$ export NIXPKGS_ALLOW_UNFREE=1
trampoline$ nix-shell
nix-shell$ # within this env you can run vs code, minikube, etc.
```

# Run the Example

The `examples/email-pipeline-worker` directory contains an example, dummy implementation of a worker that implements a batch email pipeline. To run
the example, you'll run three components:

- Pulsar in Minikube
- The example worker that implements an email pipeline
- A Dispatcher

## Run Pulsar in Minikube

Follow [these instructions](https://pulsar.apache.org/docs/next/getting-started-helm/) to run Pulsar on Minikube, summarized here:

```shell
$ minikube start --memory=8192 --cpus=4 --kubernetes-version=1.23.1

# Install Pulsar
$ helm repo add apache https://pulsar.apache.org/charts && helm repo update
$ helm install --values examples/values-minikube.yaml --set initialize=true --namespace pulsar pulsar-mini apache/pulsar

# Watch the following command to see when Pulsar is ready:
$ kubectl get pods -n pulsar

# Make Pulsar accessible through its proxy service
$ minikube service pulsar-mini-proxy -n pulsar
```

Take note of the service URL printed when running `minikube service` above

## Build and Run the email-pipeline Example Worker

```shell
trampoline$ cd examples/email-pipeline-worker
trampoline/examples/email-pipeline-worker$ cargo run
```

## Configure the Dispatcher

In the `trampoline/dispatcher` directory create a config file `dispatcher.toml` with
the following contents:

```toml
[mq]
url = "pulsar://<the service url from `minikube service`>"

[[workers]]
task_selector = { type = "email-pipeline" }
endpoint = "http://localhost:3000"
```

For `mq.url` use the Pulsar service URL printed from the `minikube service` command, bound to the `pulsar/6650` port. `minikube service` shows this with the http protocol, but you'll actually use `pulsar://`.


## Build and Run the Dispatcher

```
trampoline/dispatcher$ cargo run
```

## Send Task Messages to the Queue

The Pulsar deployment includes a toolkit pod that contains the basic Pulsar tools:

```
$ kubectl exec -it -n pulsar pulsar-mini-toolset-0 -- /bin/bash
pulsar@pulsar-mini-toolset-0:/pulsar$ bin/pulsar-client produce test  -m '{"type": "email-pipeline", "value": {}}' -s '\n'
```

This triggers a run of the dummy batch email pipeline. You can observe the run by
watching the logs for both the dispatcher and the pipeline worker.