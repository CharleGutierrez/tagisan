# Sources

- [ACK CSI service-role authorization check](https://help.aliyun.com/en/ack/product-overview/product-change-announcement-on-adding-csi-related-service-role-authorization-check-when-creating-ack-managed-cluster) — account-level authorization requirement for CSI plugin and provisioner roles.

All sources were checked on 2026-08-11.

- [ACS supported regions](https://help.aliyun.com/zh/cs/product-overview/open-service-area) — confirms Hangzhou as `cn-hangzhou`.
- [Create an ACS cluster](https://help.aliyun.com/zh/cs/user-guide/create-an-acs-cluster) — console and OpenAPI requirements, network, components, and example request.
- [ACS cluster network planning](https://help.aliyun.com/zh/cs/user-guide/acs-cluster-network-planning) — VPC, VSwitch, Pod IP, Service CIDR, and multi-zone planning.
- [Delete an ACS cluster](https://help.aliyun.com/zh/cs/user-guide/deleting-a-cluster) — workload cleanup, deletion protection, and associated-resource selection.
- [CreateCluster metadata](https://api.aliyun.com/meta/v1/products/CS/versions/2015-12-15/apis/CreateCluster/api.json) — current request and response schema.
- [DescribeClusterDetail metadata](https://api.aliyun.com/meta/v1/products/CS/versions/2015-12-15/apis/DescribeClusterDetail/api.json) — cluster state and topology schema.
- [DescribeClusterResources metadata](https://api.aliyun.com/meta/v1/products/CS/versions/2015-12-15/apis/DescribeClusterResources/api.json) — associated-resource inventory.
- [DescribeClusterUserKubeconfig metadata](https://api.aliyun.com/meta/v1/products/CS/versions/2015-12-15/apis/DescribeClusterUserKubeconfig/api.json) — temporary KubeConfig parameters.
- [ModifyCluster metadata](https://api.aliyun.com/meta/v1/products/CS/versions/2015-12-15/apis/ModifyCluster/api.json) — deletion-protection update.
- [DeleteCluster metadata](https://api.aliyun.com/meta/v1/products/CS/versions/2015-12-15/apis/DeleteCluster/api.json) — associated-resource delete/retain behavior.
- [InstallClusterAddons metadata](https://api.aliyun.com/meta/v1/products/CS/versions/2015-12-15/apis/InstallClusterAddons/api.json) — addon version and configuration contract.
- [Alibaba Cloud CS Python SDK](https://pypi.org/project/alibabacloud-cs20151215/) — official SDK package used by the bundled script.
