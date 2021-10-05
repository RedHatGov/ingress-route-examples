# OpenShift Ingress and Route Examples

This small collection of kubernetes manifests is designed to demonstrate the practical differences as the deployer of an application from using OpenShift Routes or Nginx as opinionated Ingress options, as well as what it looks like when using a more genericized Ingress implementation with either controller.

## Intended use

These deliberately simplified examples are designed to show the difference between behaviors when using different manifest definitions with different IngressControllers. They are not designed to express an opinion about any production implementations on your own clusters for your own applications - you should use the things that make the most sense for you. Hopefully we can demystify some of your choices here.

## Further Reading

Please see the [OpenShift Documentation](https://docs.openshift.com/container-platform/4.8/networking/configuring_ingress_cluster_traffic/overview-traffic.html) for more information on working with the provided IngressController for OpenShift, the OpenShift Router. If you have more questions about using external IngressControllers on OpenShift, see [this blog post](https://www.redhat.com/en/blog/using-nginx-ingress-controller-red-hat-openshift) as an example of using Nginx.
