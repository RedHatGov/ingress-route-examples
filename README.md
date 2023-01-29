# OpenShift Ingress and Route Examples

This small collection of kubernetes manifests is designed to demonstrate the practical differences as the deployer of an application from using OpenShift Routes or Nginx as opinionated Ingress options, as well as what it looks like when using a more genericized Ingress implementation with either controller.

## Intended use

These deliberately simplified examples are designed to show the difference between behaviors when using different manifest definitions with different IngressControllers. They are not designed to express an opinion about any production implementations on your own clusters for your own applications - you should use the things that make the most sense for you. Hopefully we can demystify some of your choices here.

Note that you will need to be a cluster-admin to install several of the things in this demo, because we have to modify Security Context Constraints in order to install an upstream ingress controller.

## Walkthrough

### Deploying our demo application

Deploy a simple application with a Deployment and Service (to load balance across the Deployment replicas):

```sh
oc apply -f https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/00-demo-application.yml
```

`namespace/helloworld created`\
`deployment.apps/hello-world created`\
`service/hello-world created`

Validate that you can reach the application with a simple port-forward hitting the Service:

```sh
oc port-forward service/hello-world -n helloworld 8000:8000 &
```

`[1] 1094138`\
`Forwarding from 127.0.0.1:8000 -> 8000`\
`Forwarding from [::1]:8000 -> 8000`

```sh
curl http://localhost:8000
```

`Handling connection for 8000`\
`Hello, world, from hello-world-56cd647dcf-rznfm!`

```sh
curl http://localhost:8000/hello/Red%20Hatter
```

`Handling connection for 8000`\
`Hello, Red Hatter, from hello-world-56cd647dcf-rznfm!`

Clean up your port forward to get ready for the application of a proper Ingress mechanism:

```sh
kill %1
```

`[1]+  Terminated              oc port-forward service/hello-world -n helloworld 8000:8000`

### Deploying an OpenShift Route to access our application

Deploy an OpenShift Route to your service with Edge TLS encryption (and use the default certificate from your OpenShift Router) using the following commands:

```sh
oc apply -f https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/01-route.yml
```

`route.route.openshift.io/hello-world created`

Validate that you can reach the application through the HTTPS Route that you’ve created:

```sh
curl https://$(oc get route -n helloworld hello-world -ojsonpath='{.status.ingress[0].host}')
```

`Hello, world, from hello-world-56cd647dcf-s4srg!`

Note that, in this case, I’ve got an HTTPS certificate deployed in my OpenShift Router that is trusted by most clients (from LetsEncrypt). If you don’t, you will have to add -k to your options for curl to accept untrusted certificates.

### Deploying an upstream Ingress controller to OpenShift

Now that we’ve got that working, let’s deploy an Nginx Ingress. My cluster happens to be on AWS, so I’ll use the standard ingress-nginx deployment designed for Kubernetes running on AWS. The standard deployment for Nginx on Kubernetes doesn’t take the default security posture of OpenShift into account, so it’s not allowed to run with the level of permissions expected. There is a fully supported Nginx operator for OpenShift that handles all of this configuration for you, but we’re sticking as close to the upstream Nginx Ingress deployment as possible here to demonstrate the portability. We can use the procedure from OpenShift documentation to add the capabilities and UID constraints required for Nginx Ingress in a targeted way (rather than simply opening the namespace up to allow anything) via a simple manifest. Let’s apply that now:

```sh
oc apply -f https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/02-nginx-ingress-scc.yml
```

`namespace/ingress-nginx created`\
`securitycontextconstraints.security.openshift.io/nginx created`\
`role.rbac.authorization.k8s.io/ingress-nginx-scc created`\
`rolebinding.rbac.authorization.k8s.io/ingress-nginx-scc created`

Then we’ll apply the stock upstream Nginx deployment:

```sh
oc apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.5.1/deploy/static/provider/aws/deploy.yaml
```

`namespace/ingress-nginx unchanged`\
`serviceaccount/ingress-nginx created`\
`serviceaccount/ingress-nginx-admission created`\
`role.rbac.authorization.k8s.io/ingress-nginx created`\
`role.rbac.authorization.k8s.io/ingress-nginx-admission created`\
`clusterrole.rbac.authorization.k8s.io/ingress-nginx created`\
`clusterrole.rbac.authorization.k8s.io/ingress-nginx-admission created`\
`rolebinding.rbac.authorization.k8s.io/ingress-nginx created`\
`rolebinding.rbac.authorization.k8s.io/ingress-nginx-admission created`\
`clusterrolebinding.rbac.authorization.k8s.io/ingress-nginx created`\
`clusterrolebinding.rbac.authorization.k8s.io/ingress-nginx-admission created`\
`configmap/ingress-nginx-controller created`\
`service/ingress-nginx-controller created`\
`service/ingress-nginx-controller-admission created`\
`deployment.apps/ingress-nginx-controller created`\
`job.batch/ingress-nginx-admission-create created`\
`job.batch/ingress-nginx-admission-patch created`\
`ingressclass.networking.k8s.io/nginx created`\
`validatingwebhookconfiguration.admissionregistration.k8s.io/ingress-nginx-admission created`

We should be able to watch our Deployment come online and show Ready at this point:

```sh
oc get deploy -n ingress-nginx -w
```

`NAME                       READY   UP-TO-DATE   AVAILABLE   AGE`\
`ingress-nginx-controller   0/1     1            0           10s`\
`ingress-nginx-controller   1/1     1            1           30s`

You can `Ctrl + C` to cancel the watch once the controller shows ready.

### Leveraging the upstream Ingress controller

Now that we’ve got Nginx set up on the cluster, there are a few paths we could take to configure access to the Nginx Ingress. Because I’m on AWS, I could configure a Route 53 CNAME record in one of my Hosted Zones to point to the NLB address. Or, if I didn’t have access to the DNS provider but didn't need the DNS to work publicly, I could look up the IP address of the NLB and set /etc/hosts entries to point to the NLB on my local machine. This is because Nginx will use either the SNI header in TLS packets or the HTTP Host reference in the HTTP headers to make decisions about where to send incoming traffic.

To keep things simple here, we’re going to just use the DNS name of the NLB in our Ingress specification for this service. To get the NLB DNS name for the Nginx Ingress Controller, and apply the Ingress resource with the updated hostname, run the following:

```sh
ingress_nlb=$(oc get service -n ingress-nginx ingress-nginx-controller -ojsonpath='{.status.loadBalancer.ingress[0].hostname}')
curl -s https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/03-nginx-ingress.yml | sed 's/hello\.nginx\.example\.com/'"${ingress_nlb}/" | oc apply -f -
```

`ingress.networking.k8s.io/hello-world created`

To verify that the Ingress worked, let’s curl it:

```sh
curl http://${ingress_nlb}
```

`Hello, world, from hello-world-56cd647dcf-s8g9w!`

If you get some error from `curl` about being unable to resolve the host, it's because the NLB was provisioned so recently that the DNS records haven't propogated just yet. Keep trying until your old DNS cache expires and a new lookup succeeds (which may take a few minutes). It might be helpful to run something like the following:

```sh
while ! curl -s --connect-timeout 5 http://${ingress_nlb}; do sleep 5; done
```

In the end, once the cloud finishes moving around to meet your requests, you'll get the right output.

In the previous example, we demonstrated a simple application and showed traffic routing to it via an OpenShift Route. Here we demonstrated the installation of an extra Kubernetes Ingress controller, in this case Nginx, and made use of that controller to direct traffic into the same demo application. The original OpenShift Route could safely be deleted in this case, leaving the Nginx Ingress path available to route traffic, thus demonstrating the flexibility and choice that comes into play by leveraging both Routes and Ingress objects to achieve the same result.

### Using unspecified Ingress resources with the OpenShift Router

One other capability, and one that really speaks to the portability of using Ingress with OpenShift, is that we can create an Ingress without a specified ingressClassName field set in the spec. OpenShift sets the OpenShift IngressController, aka the OpenShift Router, as the default IngressClass and would therefore handle this Ingress resource by creating a Route and then handling it like normal.

Let’s take a look at what happens when we apply a more generic Ingress resource, with the host name aligning with our OpenShift router instead of the Nginx Ingress.

```sh
router_nlb=$(oc get service router-default -n openshift-ingress -ojsonpath='{.status.loadBalancer.ingress[0].hostname}')
curl -s https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/04-agnostic-ingress.yml | sed 's/hello\.agnostic\.example\.com/'"${router_nlb}/" | oc apply -f -
```

`ingress.networking.k8s.io/agnostic created`

Something interesting happens when we create this Ingress. As mentioned above, instead of Nginx recognizing it and handling configuration, it gets assigned to our default IngressClass. You can see Nginx rejecting it and the result of the OpenShift Router receiving it by running the following:

```sh
oc logs deploy/ingress-nginx-controller -n ingress-nginx | tail -2
```

`I1005 20:36:41.802965       7 main.go:101] "successfully validated configuration, accepting" ingress="agnostic/helloworld"`\
`I1005 20:36:41.811462       7 store.go:361] "Ignoring ingress because of error while validating ingress class" ingress="helloworld/agnostic" error="ingress does not contain a valid IngressClass"`

```sh
oc get route -n helloworld
```

`NAME             HOST/PORT                                                                PATH   SERVICES      PORT    TERMINATION   WILDCARD`\
`agnostic-52wv8   a578d2c79f3e5432aaf2d980bb333e6f-284865141.us-east-2.elb.amazonaws.com   /      hello-world   <all>                 None`\
`hello-world      hello-world-helloworld.apps.openshift.jharmison.net                             hello-world   8000    edge          None`

And finally, as before, we can validate it by hitting the host from our Ingress (or our Route!):

```sh
curl http://$router_nlb
```

`Hello, world, from hello-world-56cd647dcf-s4srg!`

```sh
curl http://$(oc get ingress -n helloworld agnostic -ojsonpath='{.spec.rules[0].host}')/hello/Red%20Hatters
```

`Hello, Red Hatters, from hello-world-56cd647dcf-s8g9w!`

### Cleanup

We can remove these similarly to how we deployed them all.

```sh
oc delete -f https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/04-agnostic-ingress.yml
oc delete -f https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/03-nginx-ingress.yml
oc delete -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.5.1/deploy/static/provider/aws/deploy.yaml --wait=false
oc delete -f https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/02-nginx-ingress-scc.yml --wait=false
oc delete -f https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/01-route.yml
oc delete -f https://raw.githubusercontent.com/RedHatGov/ingress-route-examples/main/00-demo-application.yml
```

`ingress.networking.k8s.io "agnostic" deleted`\
`ingress.networking.k8s.io "hello-world" deleted`\
`namespace "ingress-nginx" deleted`\
`serviceaccount "ingress-nginx" deleted`\
`serviceaccount "ingress-nginx-admission" deleted`\
`role.rbac.authorization.k8s.io "ingress-nginx" deleted`\
`role.rbac.authorization.k8s.io "ingress-nginx-admission" deleted`\
`clusterrole.rbac.authorization.k8s.io "ingress-nginx" deleted`\
`clusterrole.rbac.authorization.k8s.io "ingress-nginx-admission" deleted`\
`rolebinding.rbac.authorization.k8s.io "ingress-nginx" deleted`\
`rolebinding.rbac.authorization.k8s.io "ingress-nginx-admission" deleted`\
`clusterrolebinding.rbac.authorization.k8s.io "ingress-nginx" deleted`\
`clusterrolebinding.rbac.authorization.k8s.io "ingress-nginx-admission" deleted`\
`configmap "ingress-nginx-controller" deleted`\
`service "ingress-nginx-controller" deleted`\
`service "ingress-nginx-controller-admission" deleted`\
`deployment.apps "ingress-nginx-controller" deleted`\
`job.batch "ingress-nginx-admission-create" deleted`\
`job.batch "ingress-nginx-admission-patch" deleted`\
`ingressclass.networking.k8s.io "nginx" deleted`\
`validatingwebhookconfiguration.admissionregistration.k8s.io "ingress-nginx-admission" deleted`\
`namespace "ingress-nginx" deleted`\
`securitycontextconstraints.security.openshift.io "nginx" deleted`\
`role.rbac.authorization.k8s.io "ingress-nginx-scc" deleted`\
`rolebinding.rbac.authorization.k8s.io "ingress-nginx-scc" deleted`\
`route.route.openshift.io "hello-world" deleted`\
`namespace "helloworld" deleted`\
`deployment.apps "hello-world" deleted`\
`service "hello-world" deleted`

## Further Reading

Please see the [OpenShift Documentation](https://docs.openshift.com/container-platform/4.9/networking/configuring_ingress_cluster_traffic/overview-traffic.html) for more information on working with the provided IngressController for OpenShift, the OpenShift Router. If you have more questions about using external IngressControllers on OpenShift, see [this blog post](https://www.redhat.com/en/blog/using-nginx-ingress-controller-red-hat-openshift) as an example of using Nginx.
