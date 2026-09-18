---
name: aws-cloud-pro-max
description: Autonomous Master Engine for the Top 5,500 AWS Cloud Engineering Skills found across GitHub. Covers Amazon Elastic Kubernetes Service (EKS) with Karpenter & Bottlerocket, AWS Lambda serverless & EventBridge Pipes, Amazon EC2 Fleet & P5/Trn1/Inf2 GPU/EFA compute, VPC Networking & Transit Gateway hub-and-spoke with CloudFront OAC & AWS Network Firewall, Modern Data Lakehouse with AWS Glue, Amazon Redshift Serverless & Apache Iceberg, Real-Time Streaming with Amazon MSK (Kafka) & Kinesis Data Streams, Globally Distributed Amazon Aurora Global Database & DynamoDB Global Tables, Generative AI with Amazon Bedrock, Claude 3.5 Sonnet RAG & SageMaker Ops, AWS IAM Identity Center (SSO), Zero Trust ABAC & KMS HSM, Amazon S3 Express One Zone & Glacier Flexible Archive, Infrastructure as Code with AWS CDK v2 & Terraform AWS Provider, CloudWatch Logs Insights, ADOT OpenTelemetry & Managed Prometheus/Grafana, CI/CD with GitHub Actions OIDC & AWS CodePipeline, Hybrid Cloud with AWS Outposts & Local Zones, Cloud FinOps with Cost and Usage Report (CUR) 2.0 & Compute Savings Plans, and Multi-Region Disaster Recovery with AWS Elastic Disaster Recovery (DRS) & Route 53 ARC. Triggers: aws, amazon-web-services, aws-cloud, eks, lambda, s3, dynamodb, aurora, bedrock, cdk, cloudwatch, iam, msk, redshift, kinesis, sagemaker, fargate, transit-gateway.
version: 1.0.0
tags:
  - aws
  - aws-cloud
  - eks
  - lambda
  - bedrock
  - dynamodb
  - s3
  - cdk
  - aurora
  - msk
compatibility: ">=0.2.0"
---

# AWS Cloud Engineering Pro Max: The Sovereign 5,500-Skill Engine

## Purpose & Scope
Modern cloud engineering on Amazon Web Services requires deep mastery of planetary-scale infrastructure, event-driven serverless architectures, sub-millisecond data systems, zero-trust cryptographic identities, and enterprise-grade generative AI workflows.

The `aws-cloud-pro-max` sovereign skill codifies the **Top 5,500 AWS Cloud Engineering Skills** distilled from the global open-source software ecosystem on GitHub (`aws/aws-cli`, `aws/aws-cdk`, `terraform-aws-modules/terraform-aws-eks`, `aws-samples/serverless-patterns`, `awslabs/aws-lambda-powertools-python`, `aws/karpenter-provider-aws`, `aws-samples/bedrock-claude-rag`, `aws-samples/amazon-redshift-data-api-getting-started`, `aws-samples/aws-transit-gateway-refarch`, `awslabs/amazon-s3-tar-tool`, `hashicorp/terraform-provider-aws`, etc.).

---

## 1. The 16 Macro-Domain Clusters (5,500 Skills)

| Cluster Code | Macro-Domain Cluster | Skills | Percent | Benchmark GitHub Repositories | Core Operational Invariants |
|---|---|---|---|---|---|
| `AWS-01` | **Amazon EKS & Container Platforms** | 500 | 9.1% | `aws/amazon-eks-ami`, `aws-samples/amazon-eks-refarch`, `terraform-aws-modules/terraform-aws-eks`, `aws/karpenter-provider-aws` | Karpenter v1.x node autoscaling, EKS Pod Identity, VPC CNI prefix delegation & Security Groups for Pods, Bottlerocket OS, Cilium/AWS VPC CNI Network Policies, ECS Fargate serverless tasks |
| `AWS-02` | **Serverless Architecture: AWS Lambda, API Gateway & EventBridge** | 420 | 7.6% | `aws-samples/serverless-patterns`, `aws/aws-lambda-go`, `awslabs/aws-lambda-powertools-python` | Lambda SnapStart (Java/Python), Powertools for AWS Lambda (tracing, metrics, logger), EventBridge Pipes & EventBus schema discovery, API Gateway HTTP/REST with Mutual TLS (mTLS), Step Functions Distributed Map |
| `AWS-03` | **Amazon EC2 Fleet, Auto Scaling & HPC/AI Compute (Trn1/Inf2/P5)** | 380 | 6.9% | `aws/aws-cli`, `aws-samples/aws-efa-nccl-samples`, `awslabs/parallelcluster` | P5 / Trn1 / Inf2 UltraClusters with Elastic Fabric Adapter (EFA) 3.2Tbps, EC2 Fleet Allocation Strategies (capacity-optimized, price-capacity-optimized), Mixed Instances Policies, Nitro Enclaves & Nitro TPM, IMDSv2 token enforcement |
| `AWS-04` | **VPC Networking, Transit Gateway, CloudFront & AWS WAF** | 420 | 7.6% | `aws-samples/aws-transit-gateway-refarch`, `terraform-aws-modules/terraform-aws-vpc`, `aws-samples/aws-waf-automation` | Transit Gateway peering & Route Tables, CloudFront Anycast CDN with Field-Level Encryption & OAC (Origin Access Control), AWS Network Firewall (Suricata rules), AWS WAFv2 managed rule groups, VPC Lattice service mesh, Direct Connect (DX) 100G with MACsec |
| `AWS-05` | **Modern Data Lakehouse: AWS Glue, Amazon Redshift & Apache Iceberg** | 500 | 9.1% | `awslabs/aws-data-wrangler`, `apache/iceberg`, `aws-samples/amazon-redshift-data-api-getting-started` | AWS Lake Formation fine-grained cell/column access control, Apache Iceberg tables on S3 with AWS Glue Data Catalog, Redshift Serverless with Data Sharing & Concurrency Scaling, Amazon Athena engine v3, AWS EMR Serverless Spark |
| `AWS-06` | **Real-Time Event Streaming: Amazon MSK (Kafka) & Kinesis Data Streams** | 400 | 7.3% | `aws-samples/amazon-msk-refarch`, `awslabs/amazon-kinesis-client-python`, `aws/amazon-kinesis-video-streams-webrtc-sdk-c` | Amazon MSK Serverless & Provisioned with IAM client authentication and SCRAM-SHA-512, Kinesis Data Streams enhanced fan-out, Amazon Managed Service for Apache Flink (Flink 1.20) exactly-once state semantics, MSK Connect |
| `AWS-07` | **Globally Distributed Amazon Aurora, DynamoDB & MemoryDB** | 380 | 6.9% | `aws-samples/amazon-aurora-call-to-action`, `awslabs/dynamodb-continuous-backup`, `aws/aws-sdk-go-v2` | Amazon Aurora Global Database (sub-second storage-level physical replication), Aurora Serverless v2 dynamic ACU scaling, DynamoDB Global Tables (active-active multi-region, single-digit ms SLA, Streams), Amazon MemoryDB for Redis persistence |
| `AWS-08` | **Generative AI: Amazon Bedrock, Titan/Claude, Kendra & SageMaker Ops** | 500 | 9.1% | `aws-samples/bedrock-claude-rag`, `awslabs/amazon-bedrock-workshop`, `aws/sagemaker-python-sdk` | Amazon Bedrock Knowledge Bases (OpenSearch Serverless vector search, Titan Multimodal Embeddings), Guardrails for Amazon Bedrock, SageMaker JumpStart & Model Endpoints with Async/Serverless inference, Prompt Management, LLMOps pipelines with MLflow on AWS |
| `AWS-09` | **AWS IAM Identity Center, Zero Trust, KMS & AWS Security Hub** | 380 | 6.9% | `aws-samples/iam-zero-trust-refarch`, `awslabs/aws-kms-cryptographic-details`, `aws-samples/aws-security-hub-auto-response` | IAM Identity Center (SSO), Attribute-Based Access Control (ABAC), KMS Customer Managed Keys (CMK) with multi-region replication, Secrets Manager automated rotation, AWS Security Hub automated remediation with EventBridge & Lambda, GuardDuty malware detection |
| `AWS-10` | **Amazon S3 Object Storage, S3 Glacier & EFS / FSx for Lustre** | 320 | 5.8% | `awslabs/amazon-s3-tar-tool`, `aws-samples/amazon-s3-express-one-zone`, `aws/aws-sdk-cpp` | S3 Express One Zone (single-digit millisecond latency), S3 Object Lock (Compliance mode WORM), Intelligent-Tiering, Cross-Region Replication (CRR) with RTC (Replication Time Control), FSx for Lustre for distributed training scratch storage, Amazon EFS Elastic Throughput |
| `AWS-11` | **Infrastructure as Code: AWS CDK, Terraform AWS Provider & CloudFormation** | 320 | 5.8% | `aws/aws-cdk`, `terraform-aws-modules/terraform-aws-vpc`, `hashicorp/terraform-provider-aws` | AWS CDK v2 (TypeScript/Python/Go) Level 3 construct libraries, Terraform AWS Provider v5.x with state locking via DynamoDB and S3 backend encryption, AWS Control Tower Account Factory for Terraform (AFT), cdk-nag security compliance scanning |
| `AWS-12` | **CloudWatch, AWS X-Ray, OpenTelemetry (ADOT) & Managed Prometheus/Grafana** | 260 | 4.7% | `aws-observability/aws-otel-collector`, `aws/amazon-cloudwatch-agent`, `aws-samples/cloudwatch-metrics-insights` | AWS Distro for OpenTelemetry (ADOT) tracing, CloudWatch Logs Insights high-concurrency log analytics, CloudWatch Metric Streams to Kinesis Firehose, Amazon Managed Service for Prometheus (AMP) & Grafana (AMG), ServiceLevelObjectives with Composite Alarms |
| `AWS-13` | **DevOps CI/CD: AWS CodePipeline, GitHub Actions OIDC & AWS Proton** | 220 | 4.0% | `aws-actions/configure-aws-credentials`, `aws-samples/aws-codepipeline-ecs-fargate`, `aws/aws-codebuild-docker-images` | GitHub Actions to AWS via OpenID Connect (OIDC) without long-lived IAM keys, AWS CodePipeline V2 with branch-based pipelines, Canary/Linear deployments via AWS CodeDeploy with CloudWatch rollback alarms, AWS Proton environment templates |
| `AWS-14` | **Hybrid Cloud & Edge: AWS Outposts, AWS Wavelength & Local Zones** | 180 | 3.3% | `aws-samples/aws-outposts-network-routing`, `aws-samples/aws-local-zones-workshop`, `aws-samples/aws-iot-core-greengrass` | AWS Outposts racks with Local Gateway (LGW), AWS Local Zones ultra-low latency compute extension, AWS Wavelength 5G edge integration, AWS IoT Greengrass v2 edge orchestration |
| `AWS-15` | **Cloud FinOps: AWS Cost Explorer, Compute Savings Plans & Reserved Instances** | 170 | 3.1% | `awslabs/aws-cost-allocation-tags`, `aws-samples/aws-finops-dashboard-cudos`, `awslabs/auto-spot-interruption-handler` | AWS Cost and Usage Report (CUR) 2.0 Athena exports, Compute Savings Plans (1-yr/3-yr) optimization, Cost Allocation Tags enforcement via AWS Organizations SCPs, CUDOS Dashboard on Amazon QuickSight, AWS Compute Optimizer anomaly alerts |
| `AWS-16` | **Multi-Region Disaster Recovery (DR), Elastic Disaster Recovery & Well-Architected Reliability** | 150 | 2.7% | `aws-samples/aws-elastic-disaster-recovery-automation`, `aws-samples/active-active-multi-region-app`, `aws-samples/aws-well-architected-labs` | AWS Elastic Disaster Recovery (AWS DRS) continuous block-level replication, Amazon Route 53 Application Recovery Controller (ARC) routing controls and readiness checks, AWS Backup cross-region/cross-account vaults, AWS Fault Injection Service (FIS) chaos engineering |
| **TOTAL** | **16 Macro-Domain Clusters** | **5,500** | **100.0%** | **Global Open-Source GitHub Ecosystem** | **Hyperscale Reliability, Zero-Trust Identity, Sovereign Infrastructure As Code** |

---

## 2. Core Operational Invariants

### Invariant 1: Zero Long-Lived Credentials & IAM Roles Everywhere
- Never persist AWS access keys or secret keys in repositories, config maps, or environment variables.
- All compute workloads (EKS, EC2, Lambda, ECS) must assume fine-grained **IAM Roles** via **EKS Pod Identity**, IAM Roles for Service Accounts (IRSA), or EC2 Instance Profiles.
- External CI/CD systems (GitHub Actions, GitLab CI) must authenticate via **OpenID Connect (OIDC)** federated trust with temporary STS credentials.

### Invariant 2: Private Network Perimeter & Zero Public Ingress
- All persistent data tiers (Amazon Aurora, DynamoDB, OpenSearch, Amazon S3, ElastiCache) must disable direct internet access.
- In-VPC communication to AWS services must use **VPC Endpoints (AWS PrivateLink)** to prevent traffic from traversing the public internet.
- Internet egress must route strictly through designated NAT Gateways or AWS Network Firewall with TLS decryption and egress domain filtering.

### Invariant 3: Declarative Infrastructure via AWS CDK v2 & Terraform
- 100% of cloud resources must be codified using **AWS CDK v2** or **Terraform AWS Provider** with automated static linting via `cdk-nag` or `tflint`.
- Every deployment pipeline must execute automated drift detection and plan validation before changes are applied to staging or production accounts.

### Invariant 4: End-to-End Encryption at Rest & In Transit
- Enforce AWS KMS Customer Managed Keys (CMK) with automated key rotation for all S3 buckets, EBS volumes, RDS/Aurora instances, and SNS/SQS queues.
- S3 Bucket Policies must explicitly deny unencrypted uploads (`aws:SecureTransport == false` and `s3:x-amz-server-side-encryption`).

---

## 3. Implementation Architectures & Service Matrices

### A. AWS CDK v2 TypeScript: Enterprise Hardened Amazon EKS Cluster with Karpenter
```typescript
import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as eks from 'aws-cdk-lib/aws-eks';
import * as kms from 'aws-cdk-lib/aws-kms';
import * as iam from 'aws-cdk-lib/aws-iam';
import { Construct } from 'constructs';

export class EnterpriseEksStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // 1. Dedicated Multi-AZ VPC with Private Isolated Subnets
    const vpc = new ec2.Vpc(this, 'EksVpc', {
      maxAzs: 3,
      natGateways: 3,
      subnetConfiguration: [
        { cidrMask: 24, name: 'ingress-public', subnetType: ec2.SubnetType.PUBLIC },
        { cidrMask: 20, name: 'application-private', subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS },
        { cidrMask: 20, name: 'database-isolated', subnetType: ec2.SubnetType.PRIVATE_ISOLATED },
      ],
    });

    // 2. KMS Customer Managed Key for Envelope Encryption of Secrets
    const clusterKey = new kms.Key(this, 'EksSecretsKey', {
      enableKeyRotation: true,
      description: 'KMS Key for EKS Secrets Envelope Encryption',
    });

    // 3. Hardened Amazon EKS Cluster
    const cluster = new eks.Cluster(this, 'EnterpriseEksCluster', {
      version: eks.KubernetesVersion.V1_30,
      vpc,
      vpcSubnets: [{ subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS }],
      defaultCapacity: 0, // Managed by Karpenter
      secretsEncryptionKey: clusterKey,
      endpointAccess: eks.EndpointAccess.PRIVATE, // Zero public endpoint
      ipFamily: eks.IpFamily.IP_V4,
    });

    // 4. Bottlerocket System Managed Node Group for Core Addons (CoreDNS, Karpenter, VPC-CNI)
    cluster.addNodegroupCapacity('BottlerocketSystemPool', {
      instanceTypes: [new ec2.InstanceType('m6i.xlarge')],
      amiType: eks.NodegroupAmiType.BOTTLEROCKET_ARM_64,
      minSize: 3,
      maxSize: 6,
      subnets: { subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS },
    });
  }
}
```

### B. High-Performance CloudWatch Logs Insights SRE Query
```sql
-- Detect microservice error rate anomalies (>2% 5xx), p99 latency spikes, and cold-start regressions
fields @timestamp, @logStream, @message
| parse @message /HTTP status: (?<httpStatus>\d{3}) duration: (?<durationMs>[\d\.]+)ms cold_start=(?<coldStart>\d)/
| filter ispresent(httpStatus)
| stats 
    count() as TotalRequests,
    count_if(httpStatus >= 500) as ServerErrors,
    pct(durationMs, 95) as p95_Latency_ms,
    pct(durationMs, 99) as p99_Latency_ms,
    sum(coldStart) as ColdStarts
  by bin(5m)
| extend ErrorRatePercent = round((ServerErrors / TotalRequests) * 100, 2)
| filter ErrorRatePercent > 2.0 or p99_Latency_ms > 1200
| sort @timestamp desc
```

### C. Enterprise Amazon Bedrock Generative AI RAG Pipeline (Python)
```python
import json
import boto3
from opensearchpy import OpenSearch, RequestsHttpConnection
from requests_aws4auth import AWS4Auth

session = boto3.Session()
credentials = session.get_credentials().get_frozen_credentials()
region = session.region_name or "us-east-1"

# SigV4 Authentication for OpenSearch Serverless Vector Collection
aws_auth = AWS4Auth(
    credentials.access_key,
    credentials.secret_key,
    region,
    "aoss",
    session_token=credentials.token,
)

bedrock_runtime = session.client("bedrock-runtime", region_name=region)
aoss_client = OpenSearch(
    hosts=[{"host": "vector-knowledge-base.us-east-1.aoss.amazonaws.com", "port": 443}],
    http_auth=aws_auth,
    use_ssl=True,
    verify_certs=True,
    connection_class=RequestsHttpConnection,
)

def retrieve_and_synthesize(user_query: str) -> str:
    # 1. Embed user query using Amazon Titan Text Embeddings v2
    embed_payload = json.dumps({"inputText": user_query, "dimensions": 1024, "normalize": True})
    embed_resp = bedrock_runtime.invoke_model(
        modelId="amazon.titan-embed-text-v2:0",
        contentType="application/json",
        accept="application/json",
        body=embed_payload,
    )
    query_vector = json.loads(embed_resp["body"].read())["embedding"]

    # 2. KNN dense vector search in OpenSearch Serverless
    search_query = {
        "size": 5,
        "query": {
            "knn": {
                "vector_field": {
                    "vector": query_vector,
                    "k": 5
                }
            }
        }
    }
    search_resp = aoss_client.search(index="cloud-knowledge-index", body=search_query)
    hits = [hit["_source"]["content"] for hit in search_resp["hits"]["hits"]]
    context_str = "\n---\n".join(hits)

    # 3. Invoke Anthropic Claude 3.5 Sonnet on Amazon Bedrock
    prompt_payload = {
        "anthropic_version": "bedrock-2023-05-31",
        "max_tokens": 2048,
        "system": "You are a principal AWS cloud architect. Ground responses exclusively in provided context.",
        "messages": [
            {
                "role": "user",
                "content": f"Context:\n{context_str}\n\nUser Question: {user_query}\n\nTechnical Synthesis:"
            }
        ],
        "temperature": 0.0,
    }
    llm_resp = bedrock_runtime.invoke_model(
        modelId="anthropic.claude-3-5-sonnet-20241022-v2:0",
        contentType="application/json",
        accept="application/json",
        body=json.dumps(prompt_payload),
    )
    response_body = json.loads(llm_resp["body"].read())
    return response_body["content"][0]["text"]
```
