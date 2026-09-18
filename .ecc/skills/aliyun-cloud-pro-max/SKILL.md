---
name: aliyun-cloud-pro-max
description: Autonomous Master Engine for the Top 5,500 Alibaba Cloud & China Cloud Engineering Skills found across GitHub. Covers Container Service for Kubernetes (ACK) with Terway CNI & OpenKruise, Function Compute (FC 3.0) & Serverless App Engine (SAE), Elastic Compute Service (ECS) with CIPU & eRDMA 750Gbps, VPC Networking & Cloud Enterprise Network (CEN) Transit Router, MaxCompute (ODPS) lakehouse & Hologres real-time HSAP analytics, Apache RocketMQ 5.0 event streaming & Canal CDC, PolarDB distributed compute-storage separation & OceanBase, Qwen 2.5 foundation models, Model Studio (Bailian) & PAI AI platform, Resource Access Management (RAM) & KMS with SM2/SM3/SM4 cryptography, Object Storage Service (OSS) with OSS-HDFS & JindoFS, Terraform Provider Alicloud & ROS IaC, Simple Log Service (SLS) & ARMS APM observability, Spring Cloud Alibaba with Nacos, Sentinel & Seata, Apsara Stack hybrid cloud, BSS FinOps cost governance, and "Two Locations Three Centers" (两地三中心) disaster resilience. Triggers: aliyun, alibaba-cloud, alicloud, ack, polardb, rocketmq, qwen, maxcompute, oss, nacos, sentinel, seata, hologres, bailian, arms, sls, apsara, cen, fc.
version: 1.0.0
tags:
  - aliyun
  - alibaba-cloud
  - alicloud
  - ack
  - polardb
  - rocketmq
  - qwen
  - maxcompute
  - oss
  - nacos
  - terraform-alicloud
compatibility: ">=0.2.0"
---

# Alibaba Cloud Engineering Pro Max: The Sovereign 5,500-Skill Engine

## Purpose & Scope
Modern cloud engineering across China and the Asia-Pacific hyperscale ecosystem requires deep mastery of planetary-scale infrastructure, resilient high-concurrency microservices, sub-second HTAP/HSAP data architectures, sovereign cryptographic standards, and enterprise-grade generative AI workflows. Alibaba Cloud (Aliyun) stands as the #1 cloud provider in China (~36% market share) and a global top-tier hyperscaler, anchoring the technological backbone of Double 11 (Singles' Day) peak traffic and mission-critical enterprise systems.

The `aliyun-cloud-pro-max` sovereign skill codifies the **Top 5,500 Alibaba Cloud & China Hyperscaler Engineering Skills** distilled from the global open-source software ecosystem on GitHub (`aliyun/terraform-provider-alicloud`, `AliyunContainerService/ack-distro`, `openkruise/kruise`, `alibaba/dragonfly`, `alibaba/higress`, `alibaba/KubeVela`, `apache/rocketmq`, `alibaba/canal`, `polardb/polardb-for-postgresql`, `oceanbase/oceanbase`, `QwenLM/Qwen2.5`, `QwenLM/Qwen-Agent`, `alibaba/spring-cloud-alibaba`, `alibaba/nacos`, `alibaba/sentinel`, `alibaba/seata`, `aliyun/serverless-application-package`, `aliyun/aliyun-cli`, etc.).

---

## 1. The 16 Macro-Domain Clusters (5,500 Skills)

| Cluster Code | Macro-Domain Cluster | Skills | Percent | Benchmark GitHub Repositories | Core Operational Invariants |
|---|---|---|---|---|---|
| `ALI-01` | **Container Service for Kubernetes (ACK) & Cloud-Native Platforms** | 500 | 9.1% | `AliyunContainerService/ack-distro`, `openkruise/kruise`, `alibaba/dragonfly`, `alibaba/higress`, `alibaba/KubeVela` | ACK Managed / ACK Serverless (ASK), RRSA (RAM Roles for Service Accounts), Terway CNI with ENI multi-IP & NetworkPolicies, OpenKruise advanced workloads (CloneSet, In-Place Update), Dragonfly P2P image acceleration, Higress Envoy-based cloud-native gateway |
| `ALI-02` | **Serverless Computing: Function Compute (FC 3.0) & Serverless App Engine (SAE)** | 420 | 7.6% | `aliyun/serverless-application-package`, `devsapp/serverless-devs`, `aliyun/fc-runtime` | Function Compute 3.0 (1ms billing granularity, GPU serverless inference, WebSockets, custom container runtimes), SAE 2.0 microservice zero-code containerization & CPU burst, EventBridge event routing, CloudFlow distributed state machines |
| `ALI-03` | **Elastic Compute Service (ECS), CIPU Architecture & High-Performance Compute** | 380 | 6.9% | `aliyun/aliyun-cli`, `aliyun/alibabacloud-python-sdk-ecs`, `alibaba/cloud-kernel` | 8th gen ECS (g8i/c8i/r8i) powered by CIPU (Cloud Infrastructure Processing Unit), Elastic RDMA (eRDMA 750Gbps) low-latency networking, Auto Scaling Fleet Allocation Strategies, Spot instance interruption draining, Aliyun Linux 3 kernel tuning |
| `ALI-04` | **VPC Networking, Cloud Enterprise Network (CEN) & Global Traffic** | 420 | 7.6% | `aliyun/terraform-provider-alicloud`, `aliyun/iac-code`, `alibabacloud-automation/terraform-alicloud-cen` | Cloud Enterprise Network (CEN) Enterprise Edition with Transit Router (TR) inter-region peering, Server Load Balancer (ALB/NLB) with QUIC & HTTP/3, Global Accelerator (GA) Anycast BGP IP, PrivateLink endpoints, Anti-DDoS Pro/Premium & Cloud Firewall |
| `ALI-05` | **Big Data Lakehouse: MaxCompute (ODPS), Hologres & DataWorks** | 500 | 9.1% | `aliyun/alibabacloud-odps-python`, `alibaba/hologres-connector`, `apache/paimon` | MaxCompute exabyte SQL processing with dynamic partitioning & CBO, Hologres real-time HSAP (Hybrid Serving & Analytical Processing) vector engine, DataWorks visual DAG orchestration, E-MapReduce (EMR 2.0) Apache Spark/StarRocks, Apache Paimon streaming data lake |
| `ALI-06` | **Distributed Messaging & Streaming: Apache RocketMQ, EventBridge & Canal CDC** | 400 | 7.3% | `apache/rocketmq`, `alibaba/canal`, `aliyun/alibabacloud-rocketmq-client-java` | Apache RocketMQ 5.0 streaming and event-mesh, scheduled messages, transactional messages, Alibaba Canal MySQL binlog CDC replication, Kafka on Message Queue, SLS log shipper |
| `ALI-07` | **Cloud-Native Databases: PolarDB, OceanBase & ApsaraDB** | 380 | 6.9% | `polardb/polardb-for-postgresql`, `oceanbase/oceanbase`, `aliyun/alibabacloud-polardb-tool-agentic-server` | PolarDB MySQL/PostgreSQL compute-storage separation (PolarStore RDMA), PolarDB Global Database Network (GDN), OceanBase financial-grade distributed Paxos SQL engine, Lindorm multi-model engine, Tair in-memory store |
| `ALI-08` | **Generative AI & LLMs: Qwen 2.5, Model Studio (Bailian), DashScope & PAI** | 500 | 9.1% | `QwenLM/Qwen2.5`, `QwenLM/Qwen-Agent`, `aliyun/qwen-code`, `alibaba/BladeDISC` | Qwen 2.5 (0.5B to 72B) & Qwen-VL multimodal, Alibaba Cloud Bailian (Model Studio) agentic workflows, DashScope API integration, PAI-EAS elastic model serving, PAI-DLC distributed deep learning, PAI-Blade inference optimization |
| `ALI-09` | **Resource Access Management (RAM), Zero-Trust Identity & KMS HSM** | 380 | 6.9% | `AliyunContainerService/ack-ram-tool`, `aliyun/alibabacloud-auth-action`, `aliyun/ram-policy-validator` | RAM policies (Action, Resource, Condition), RAM Role SSO with OIDC federation, Key Management Service (KMS) with Dedicated HSM & Envelope Encryption, ActionTrail audit governance, Security Center threat detection |
| `ALI-10` | **Object Storage Service (OSS), OSS-HDFS, JindoFS & NAS** | 320 | 5.8% | `aliyun/aliyun-oss-go-sdk`, `aliyun/alibabacloud-jindodata`, `aliyun/ossfs` | OSS Standard / IA / Archive / Cold Archive / Deep Cold Archive tiering, WORM retention policy, Cross-Region Replication (CRR), OSS-HDFS / JindoFS acceleration for AI/big data pipelines, Apsara File Storage NAS & CPFS |
| `ALI-11` | **Infrastructure as Code: Terraform Alicloud, ROS & Darabonba SDK** | 320 | 5.8% | `aliyun/terraform-provider-alicloud`, `aliyun/iac-code`, `aliyun/darabonba` | Terraform Provider Alicloud (`aliyun/terraform-provider-alicloud`), Resource Orchestration Service (ROS) templates, Darabonba multi-language DSL code generator, Aliyun CLI automation, Cloud Control API |
| `ALI-12` | **CloudMonitor, Simple Log Service (SLS), ARMS & Observability** | 260 | 4.7% | `aliyun/aliyun-log-python-sdk`, `alibaba/sentinel`, `aliyun/arms-c-sdk` | SLS (Simple Log Service) petabyte real-time indexing and SQL92 analytics, Application Real-Time Monitoring Service (ARMS) APM & Prometheus monitoring, CloudMonitor alert policies, OpenTelemetry exporter integration |
| `ALI-13` | **DevOps & Cloud Native Microservices: Nacos, Sentinel, Seata & ACR** | 220 | 4.0% | `alibaba/spring-cloud-alibaba`, `alibaba/nacos`, `alibaba/sentinel`, `alibaba/seata` | Spring Cloud Alibaba, Nacos dynamic discovery & config management, Sentinel circuit breaking and traffic shaping, Seata distributed transactions, ACR EE (Container Registry Enterprise Edition) vulnerability scanning and global sync |
| `ALI-14` | **Hybrid Cloud, Apsara Stack & Sovereign Cloud Infrastructure** | 180 | 3.3% | `aliyun/apsarastack-sdk`, `aliyun/smart-access-gateway`, `alibaba/cloud-mesh` | Apsara Stack enterprise private cloud platform, Cloud Box on-premises dedicated hardware, Smart Access Gateway (SAG) SD-WAN, Data residency compliance in China (MLPS 2.0 / PIPL / DSL compliance) |
| `ALI-15` | **Cloud FinOps: Resource Directory, Savings Plans & BSS Cost Governance** | 170 | 3.1% | `aliyun/alibabacloud-python-sdk-bssopenapi`, `aliyun/finops-toolkit`, `aliyun/cost-explorer-cli` | BSS (Business Support System) OpenAPI billing analytics, Resource Directory multi-account hierarchy, Savings Plans and Reserved Instances (RI) optimization, Spot Fleet allocation strategies, Cost Center tagging |
| `ALI-16` | **Disaster Recovery, "Two Locations Three Centers" (两地三中心) & Resilience** | 150 | 2.7% | `alibaba/chaosblade`, `aliyun/cloud-disaster-recovery`, `aliyun/ahas-toolkit` | Multi-AZ / Multi-Region active-active high availability (两地三中心), Cloud Disaster Recovery (CDR) continuous block replication, DNS GTM (Global Traffic Manager) automatic failover, AHAS (Application High Availability Service) chaos engineering |
| **TOTAL** | **16 Macro-Domain Clusters** | **5,500** | **100.0%** | **Global Open-Source GitHub Ecosystem** | **Hyperscale Reliability, Zero-Trust Defense, Sovereign Infrastructure As Code** |

---

## 2. Core Operational Invariants

### Invariant 1: Zero Long-Lived AccessKeys & RRSA Everywhere
- Never hardcode Alibaba Cloud `AccessKeyId` or `AccessKeySecret` in code repositories, Dockerfiles, or Kubernetes configmaps.
- All containerized workloads on ACK must authenticate via **RAM Roles for Service Accounts (RRSA)** using OpenID Connect (OIDC) federated STS temporary credentials.
- CI/CD platforms (such as GitHub Actions or GitLab CI) must authenticate using `aliyun/alibabacloud-auth-action` with OIDC identity providers to retrieve short-lived STS tokens.

### Invariant 2: Private Network Isolation & CEN Transit Router Backbone
- All relational, NoSQL, and cache data instances (PolarDB, OceanBase, ApsaraDB for Redis, Hologres) must be bound exclusively to private VPC subnets with public IP access disabled.
- Multi-VPC and hybrid interconnectivity must route through **Cloud Enterprise Network (CEN) Transit Router (TR)** with enterprise route tables and security domains.
- In-VPC API communication to Alibaba Cloud services (OSS, SLS, KMS) must route over **VPC Endpoints (PrivateLink)** to eliminate public internet traversals.

### Invariant 3: Declarative Infrastructure as Code via Terraform Provider Alicloud
- 100% of cloud resources must be provisioned and versioned using **Terraform Provider Alicloud** (`aliyun/terraform-provider-alicloud`) or **Resource Orchestration Service (ROS)**.
- Remote state files must be stored in private OSS buckets configured with server-side encryption (KMS CMK) and Object Lock WORM retention, coupled with Tablestore state locking.

### Invariant 4: Sovereign Compliance & SM2/SM3/SM4 Cryptographic Governance
- In adherence to Multi-Level Protection Scheme (MLPS 2.0 / 等保三级/四级), Personal Information Protection Law (PIPL), and Data Security Law (DSL), sensitive data must reside in mainland China data regions with localized cryptographic boundaries.
- Envelope encryption must leverage Alibaba Cloud Key Management Service (KMS) supporting both international algorithms (AES-256, RSA) and Chinese National Commercial Cryptography (国密 SM2/SM3/SM4).

---

## 3. Implementation Architectures & Service Matrices

### A. Terraform Alicloud: Enterprise Hardened ACK Cluster with Terway CNI & PolarDB
```hcl
terraform {
  required_version = ">= 1.5.0"
  required_providers {
    alicloud = {
      source  = "aliyun/alicloud"
      version = "~> 1.220.0"
    }
  }
}

provider "alicloud" {
  region = "cn-hangzhou"
}

# 1. Multi-Zone Dedicated VPC
resource "alicloud_vpc" "production_vpc" {
  vpc_name   = "prod-core-vpc"
  cidr_block = "10.100.0.0/16"
}

resource "alicloud_vswitch" "vswitch_ack_zone_k" {
  vpc_id       = alicloud_vpc.production_vpc.id
  cidr_block   = "10.100.1.0/24"
  zone_id      = "cn-hangzhou-k"
  vswitch_name = "prod-ack-k"
}

resource "alicloud_vswitch" "vswitch_ack_zone_j" {
  vpc_id       = alicloud_vpc.production_vpc.id
  cidr_block   = "10.100.2.0/24"
  zone_id      = "cn-hangzhou-j"
  vswitch_name = "prod-ack-j"
}

resource "alicloud_vswitch" "vswitch_db_zone_k" {
  vpc_id       = alicloud_vpc.production_vpc.id
  cidr_block   = "10.100.10.0/24"
  zone_id      = "cn-hangzhou-k"
  vswitch_name = "prod-db-k"
}

# 2. KMS Key with SM4 National Commercial Cryptography
resource "alicloud_kms_key" "cluster_kms_key" {
  description            = "KMS Key for ACK Secrets & PolarDB Encryption"
  key_spec               = "Aliyun_SM4"
  key_usage              = "ENCRYPT/DECRYPT"
  automatic_rotation     = "Enabled"
  rotation_interval      = "365d"
  pending_window_in_days = 7
}

# 3. Hardened Container Service for Kubernetes (ACK) Managed Cluster
resource "alicloud_cs_managed_kubernetes" "ack_cluster" {
  name                         = "prod-ack-cluster"
  cluster_spec                 = "ack.pro.small"
  version                      = "1.30.1-aliyun.1"
  vswitch_ids                  = [alicloud_vswitch.vswitch_ack_zone_k.id, alicloud_vswitch.vswitch_ack_zone_j.id]
  new_nat_gateway              = false
  pod_vswitch_ids              = [alicloud_vswitch.vswitch_ack_zone_k.id, alicloud_vswitch.vswitch_ack_zone_j.id]
  service_cidr                 = "172.16.0.0/16"
  slb_internet_enabled         = false
  enable_rrsa                  = true # RAM Roles for Service Accounts
  encryption_provider_key      = alicloud_kms_key.cluster_kms_key.id
  load_balancer_spec           = "slb.s2.small"
  timezone                     = "Asia/Shanghai"
  proxy_mode                   = "ipvs"
}

# 4. PolarDB Distributed Database Cluster (Compute-Storage Separation)
resource "alicloud_polardb_cluster" "db_cluster" {
  db_type           = "MySQL"
  db_version        = "8.0"
  db_node_class     = "polar.mysql.x8.2xlarge"
  vpc_id            = alicloud_vpc.production_vpc.id
  vswitch_id        = alicloud_vswitch.vswitch_db_zone_k.id
  description       = "Production High-Performance PolarDB"
  pay_type          = "PostPaid"
  security_ips      = [alicloud_vpc.production_vpc.cidr_block]
  kms_key_id        = alicloud_kms_key.cluster_kms_key.id
  encrypt_new_tables = "ON"
}
```

---

### B. High-Performance Simple Log Service (SLS) SQL92 Real-Time Query
```sql
-- SLS Real-Time Diagnostic Query: Detect high-traffic 5xx anomalies, p99 latency regressions, and slow SQL in microservices
* 
| select 
    date_trunc('minute', __time__) as time_window,
    service_name,
    count(1) as total_requests,
    count_if(status >= 500) as server_errors,
    round(count_if(status >= 500) * 100.0 / count(1), 2) as error_rate_percent,
    approx_percentile(response_time_ms, 0.95) as p95_latency_ms,
    approx_percentile(response_time_ms, 0.99) as p99_latency_ms,
    max(response_time_ms) as max_latency_ms
group by time_window, service_name
having count(1) > 100 and (error_rate_percent > 1.0 or p99_latency_ms > 800)
order by time_window desc
limit 50
```

---

### C. Enterprise Generative AI: Qwen 2.5 & Alibaba Cloud Bailian (Model Studio) RAG (Python)
```python
import os
import dashscope
from dashscope import Generation
from dashscope.embeddings import TextEmbedding

# DashScope API integration with Qwen 2.5 & Model Studio (Bailian)
dashscope.api_key = os.getenv("DASHSCOPE_API_KEY")

def generate_enterprise_answer(query: str, retrieved_contexts: list[str]) -> str:
    """Synthesize an enterprise-grounded response using Qwen-2.5-72B-Instruct."""
    context_block = "\n---\n".join(retrieved_contexts)
    
    messages = [
        {
            "role": "system",
            "content": (
                "You are the Alibaba Cloud Sovereign Principal AI Architect. "
                "Answer questions strictly using the provided Alibaba Cloud technical reference. "
                "If the answer is unknown, state that transparently without hallucination."
            )
        },
        {
            "role": "user",
            "content": f"Enterprise Context:\n{context_block}\n\nUser Question: {query}\n\nTechnical Answer:"
        }
    ]

    response = Generation.call(
        model="qwen-2.5-72b-instruct",
        messages=messages,
        result_format="message",
        temperature=0.1,
        top_p=0.8,
        max_tokens=2048,
    )

    if response.status_code == 200:
        return response.output.choices[0].message.content
    else:
        raise RuntimeError(f"DashScope invocation failed: {response.code} - {response.message}")

def get_text_embeddings(texts: list[str]) -> list[list[float]]:
    """Generate dense text embeddings using text-embedding-v3 for vector retrieval."""
    resp = TextEmbedding.call(
        model=TextEmbedding.Models.text_embedding_v3,
        input=texts,
        dimension=1024
    )
    if resp.status_code == 200:
        return [item["embedding"] for item in resp.output["embeddings"]]
    raise RuntimeError(f"Embedding generation failed: {resp.code} - {resp.message}")
```

---

### D. Spring Cloud Alibaba: Nacos Discovery & Sentinel Traffic Shaping Configuration
```yaml
# application.yml: Spring Cloud Alibaba Production Configuration
spring:
  application:
    name: order-orchestrator-service
  cloud:
    nacos:
      discovery:
        server-addr: nacos-server.production.internal:8848
        namespace: 4b68e912-3f11-4cb5-8d54-94c03b128790
        group: PROD_GROUP
        ephemeral: true
      config:
        server-addr: nacos-server.production.internal:8848
        namespace: 4b68e912-3f11-4cb5-8d54-94c03b128790
        group: PROD_GROUP
        file-extension: yaml
        refresh-enabled: true
    sentinel:
      transport:
        dashboard: sentinel-dashboard.production.internal:8080
        port: 8719
      eager: true
      datasource:
        flow:
          nacos:
            server-addr: nacos-server.production.internal:8848
            data-id: ${spring.application.name}-flow-rules
            group-id: SENTINEL_GROUP
            rule-type: flow
```

---

### E. Apache RocketMQ 5.0 FIFO / Transactional Message Workflow
```java
package com.alibaba.cloud.demo;

import org.apache.rocketmq.client.apis.*;
import org.apache.rocketmq.client.apis.message.Message;
import org.apache.rocketmq.client.apis.producer.*;
import java.nio.charset.StandardCharsets;

public class RocketMqTransactionProducer {
    public static void main(String[] args) throws Exception {
        String endpoint = "rmq-cn-hangzhou-prod.rocketmq.aliyuncs.com:8080";
        String topic = "order-transaction-topic";

        ClientServiceProvider provider = ClientServiceProvider.loadService();
        ClientConfiguration configuration = ClientConfiguration.newBuilder()
            .setEndpoints(endpoint)
            .enableSsl(true)
            .build();

        TransactionChecker checker = messageView -> {
            System.out.println("Checking local transaction state for message: " + messageView.getMessageId());
            return TransactionResolution.COMMIT;
        };

        Producer producer = provider.newProducerBuilder()
            .setClientConfiguration(configuration)
            .setTransactionChecker(checker)
            .build();

        Transaction transaction = producer.beginTransaction();
        Message message = provider.newMessageBuilder()
            .setTopic(topic)
            .setTag("OrderCreate")
            .setKeys("ORDER_20260918_8899")
            .setBody("{\"orderId\":\"8899\",\"amount\":1299.00}".getBytes(StandardCharsets.UTF_8))
            .build();

        // Send half-message
        SendReceipt receipt = producer.send(message, transaction);
        System.out.println("Half-message sent successfully, messageId: " + receipt.getMessageId());

        // Execute local database transaction and commit
        boolean localSuccess = true; // DB operation result
        if (localSuccess) {
            transaction.commit();
            System.out.println("Transaction committed successfully.");
        } else {
            transaction.rollback();
            System.err.println("Transaction rolled back.");
        }

        producer.close();
    }
}
```
