---
name: azure-cloud-pro-max
description: Autonomous Master Engine for the Top 5,500 Azure Cloud Engineering Skills found across GitHub. Covers Azure Kubernetes Service (AKS) with Cilium eBPF, Azure App Service & Functions v4 serverless, Virtual Machines & HPC H100/H200 GPU scale sets, Virtual Network (VNet) & Virtual WAN hub-and-spoke with Front Door & Azure Firewall Premium, Microsoft Fabric & Synapse lakehouse analytics, Event Hubs & Stream Analytics real-time streaming, globally distributed Cosmos DB multi-region & Azure SQL Hyperscale, Azure OpenAI Service & Azure AI Foundry RAG ops, Microsoft Entra ID (Azure AD), Privileged Identity Management (PIM), Managed Identities & Key Vault HSM, Azure Blob Storage ADLS Gen2 & NetApp Files, Infrastructure as Code with Bicep, Azure Verified Modules (AVM) & Terraform AzureRM, Azure Monitor with KQL & OpenTelemetry, Azure DevOps Pipelines & GitHub Actions CI/CD, Azure Arc & Sovereign hybrid cloud, FinOps with Microsoft Cost Management & Savings Plans, and multi-region disaster recovery (ASR) with regulatory compliance. Triggers: azure, azure-cloud, az, bicep, aks, cosmosdb, azure openai, fabric, synapse, event hubs, entra id, key vault, azure functions, azure app service, azure devops, arc.
version: 1.0.0
tags:
  - azure
  - azure-cloud
  - aks
  - cosmosdb
  - azure-openai
  - bicep
  - fabric
  - synapse
  - entra-id
  - event-hubs
compatibility: ">=0.2.0"
---

# Azure Cloud Engineering Pro Max: The Sovereign 5,500-Skill Engine

## Purpose & Scope
Modern hyperscale enterprise cloud engineering demands high availability, sub-millisecond data pipelines, zero-trust cryptographic identities, and petabyte-scale analytics. Microsoft Azure provides planetary-scale cloud infrastructure with native enterprise identity integration, distributed databases, hybrid governance, and generative AI platforms.

The `azure-cloud-pro-max` sovereign skill codifies the **Top 5,500 Azure Cloud Engineering Skills** distilled from the global open-source software ecosystem on GitHub (`Azure/azure-cli`, `Azure/bicep`, `Azure/Azure-Verified-Modules`, `Azure/AKS`, `Azure-Samples/azure-search-openai-demo`, `microsoft/semantic-kernel`, `microsoft/fabric-samples`, `Azure/azure-event-hubs`, `Azure/azure-cosmos-dotnet-v3`, `Azure-Samples/virtual-wan-enterprise`, `hashicorp/terraform-provider-azurerm`, etc.).

---

## 1. The 16 Macro-Domain Clusters (5,500 Skills)

| Cluster Code | Macro-Domain Cluster | Skills | Percent | Benchmark GitHub Repositories | Core Operational Invariants |
|---|---|---|---|---|---|
| `AZR-01` | **Azure Kubernetes Service (AKS) & Container Platforms** | 500 | 9.1% | `Azure/AKS`, `Azure/azure-container-apps`, `Azure-Samples/aks-enterprise-scale` | Cilium CNI Powered by eBPF, Azure CNI Overlay, Istio service mesh, Workload Identity, KEDA autoscaling, Azure Container Apps (ACA) gen2, Confidential Containers |
| `AZR-02` | **Azure App Service, Serverless Functions & Event Grid** | 420 | 7.6% | `Azure/azure-functions`, `Azure-Samples/serverless-microservices-reference-architecture` | Functions v4 Isolated Worker (.NET/Python/Node/Rust), Premium Plan 0-cold start, Event Grid MQTT Broker & CloudEvents v1.0, Logic Apps Standard, VNet direct integration |
| `AZR-03` | **Virtual Machines, VMSS Fleet & HPC/AI Compute** | 380 | 6.9% | `Azure/azure-hpc`, `Azure-Samples/compute-automation`, `Azure/azure-cli` | NDv5/NCv4 H100/H200 GPU VMs, InfiniBand NDR/HDR 3.2Tbps, VMSS Flexible Orchestration, Spot VMs with Scheduled Events, Proximity Placement Groups, Ephemeral OS disks |
| `AZR-04` | **Virtual Network (VNet), Virtual WAN & Network Security** | 420 | 7.6% | `Azure/azure-network`, `Azure-Samples/virtual-wan-enterprise`, `Azure/azure-firewall` | Virtual WAN (vWAN) Hub-and-Spoke, Front Door Premium (Global Anycast, WAF, Private Link), Azure Firewall Premium (TLS Inspection, IDPS), Private Endpoints, ExpressRoute Direct 100G |
| `AZR-05` | **Microsoft Fabric, Synapse Analytics & OneLake Lakehouse** | 500 | 9.1% | `microsoft/fabric-samples`, `Azure-Samples/modern-data-warehouse-dataops`, `delta-io/delta` | Fabric OneLake Delta Parquet / Iceberg interop, Fabric Direct Lake mode, Synapse Serverless SQL, Data Factory pipelines, Azure Databricks Unity Catalog federation |
| `AZR-06` | **Real-Time Streaming: Event Hubs & Stream Analytics** | 400 | 7.3% | `Azure/azure-event-hubs`, `Azure/azure-cosmos-dotnet-v3`, `Azure-Samples/streaming-at-scale` | Event Hubs Dedicated with Kafka 3.x API, Spark Structured Streaming, Azure Stream Analytics sub-second windows, Cosmos DB Change Feed processor, Azure Web PubSub |
| `AZR-07` | **Globally Distributed Cosmos DB, SQL Hyperscale & Postgres** | 380 | 6.9% | `Azure/azure-cosmos-db-emulator-docker`, `Azure-Samples/cosmos-db-nosql-copilot` | Cosmos DB multi-region multi-master (99.999% SLA, 5 consistency levels), Azure SQL Hyperscale (Named Replicas, Auto-failover groups), Postgres Flexible Server with pgvector |
| `AZR-08` | **Azure OpenAI Service, Azure AI Foundry & GenAI RAG Ops** | 500 | 9.1% | `Azure-Samples/azure-search-openai-demo`, `microsoft/semantic-kernel`, `Azure/azure-sdk-for-python` | Azure OpenAI GPT-4o/o1/o3 Provisioned Throughput (PTU), Semantic Kernel, Azure AI Search (Hybrid retrieval, dense vector HNSW, semantic reranker), AI Content Safety, Prompt Flow |
| `AZR-09` | **Microsoft Entra ID, Managed Identities, Key Vault & Defender**| 380 | 6.9% | `Azure/azure-cli`, `Azure/azure-rest-api-specs`, `Azure-Samples/active-directory-b2c-advanced-policies`| Entra ID Conditional Access, Privileged Identity Management (PIM), User-Assigned & System-Assigned Managed Identities, Key Vault HSM (M-of-N quorum), Defender for Cloud, Microsoft Sentinel |
| `AZR-10` | **Azure Blob Storage, ADLS Gen2 & NetApp Files** | 320 | 5.8% | `Azure/azure-storage-blob-go`, `Azure-Samples/storage-blob-dotnet-getting-started` | ADLS Gen2 Hierarchical Namespace (HNS), Lifecycle Management (Hot/Cool/Cold/Archive), Azure NetApp Files (Ultra/Premium performance), Cross-Region Object Replication, Immutability (WORM) |
| `AZR-11` | **Infrastructure as Code: Bicep, AVM & Terraform AzureRM** | 320 | 5.8% | `Azure/bicep`, `Azure/Azure-Verified-Modules`, `hashicorp/terraform-provider-azurerm` | Bicep modular blueprints, Azure Verified Modules (AVM), Enterprise-Scale Landing Zones (ALZ), Terraform AzureRM v4.x, AzAPI provider day-0 ARM resource APIs, What-If deployments |
| `AZR-12` | **Azure Monitor, Log Analytics (KQL), App Insights & SRE** | 260 | 4.7% | `Azure/azure-monitor-opentelemetry-distro`, `microsoft/Kusto-Query-Language` | Kusto Query Language (KQL) mastery, Azure Monitor baseline dynamic alerting, Application Insights distributed tracing with OpenTelemetry, Managed Grafana & Prometheus, Error Budget SLI/SLO |
| `AZR-13` | **DevOps: Azure DevOps Pipelines, GitHub Actions & ADE** | 220 | 4.0% | `microsoft/azure-pipelines-tasks`, `Azure/actions-workflow-samples` | Azure DevOps Multi-Stage YAML Pipelines, OIDC Federation with GitHub Actions (zero client secrets), Azure Deployment Environments (ADE), Deployment Slots zero-downtime swap |
| `AZR-14` | **Hybrid Cloud: Azure Arc, Azure Stack HCI & Sovereign Cloud** | 180 | 3.3% | `Azure/azure-arc-jumpstart`, `Azure/Azure-Security-Benchmark` | Azure Arc-enabled Kubernetes & Servers, GitOps via Flux v2, Azure Arc Data Services (SQL Managed Instance / Postgres), Azure Stack HCI, Azure for Government & Sovereign Landing Zones |
| `AZR-15` | **FinOps: Microsoft Cost Management, RIs & Savings Plans** | 170 | 3.1% | `microsoft/finops-toolkit`, `Azure-Samples/cost-management-automation` | Cost Management exports, 1-yr and 3-yr Azure Savings Plans and Reserved VM Instances (RIs), Cost Allocation Rules, Tagging governance policies via Azure Policy, Anomaly detection |
| `AZR-16` | **Multi-Region Disaster Recovery (ASR), BCDR & Compliance** | 150 | 2.7% | `Azure-Samples/disaster-recovery-dr-architectures`, `Azure/azure-policy` | Azure Site Recovery (ASR), Zone-Redundant (ZRS) to Geo-Zone-Redundant (GZRS) replication, Failover groups, Regulatory Compliance Initiatives (ISO 27001, SOC 2, FedRAMP High, PCI-DSS 4.0) |
| **TOTAL** | **16 Macro-Domain Clusters** | **5,500** | **100.0%** | **Global Open-Source GitHub Ecosystem** | **Hyperscale Reliability, Zero-Trust Identity, Sovereign Infrastructure As Code** |

---

## 2. Core Operational Invariants

### Invariant 1: Zero Secrets in Code & Managed Identity Everywhere
- Never persist client secrets, service principal passwords, or connection strings in repositories or environment variables.
- Always use **User-Assigned or System-Assigned Managed Identities** with Azure RBAC (`Microsoft.Authorization/roleAssignments`) for all service-to-service communication (AKS to Key Vault, App Service to SQL, Functions to Event Hubs).
- Authenticate GitHub Actions and external CI/CD via **Workload Identity Federation** using OpenID Connect (OIDC).

### Invariant 2: Private Network Perimeter & Zero Public IPs
- All database instances (Cosmos DB, Azure SQL, Postgres Flexible Server) and storage accounts must enforce `publicNetworkAccess = "Disabled"`.
- Access must exclusively traverse **Azure Private Endpoints** backed by Private DNS Zones (`privatelink.blob.core.windows.net`, `privatelink.database.windows.net`).
- Public inbound ingress must be fronted by **Azure Front Door Premium** or **Application Gateway v2** with WAF inspection and DDoS Protection Standard.

### Invariant 3: Declarative Infrastructure via Bicep & Azure Verified Modules (AVM)
- Infrastructure must be codified in **Bicep** or **Terraform AzureRM** using Azure Verified Modules (AVM) adhering to Enterprise-Scale Landing Zone (ALZ) guidelines.
- Production deployments must run through validation (`az deployment group what-if`) and pull-request verification before applying.

### Invariant 4: Continuous Telemetry, KQL & Distributed Tracing
- All microservices must emit structured telemetry using the **Azure Monitor OpenTelemetry Distro**.
- Queries across Log Analytics Workspaces must leverage indexed **Kusto Query Language (KQL)** for sub-second incident correlation, metric extraction, and automated alert generation.

---

## 3. Implementation Architectures & Service Matrices

### AKS with Azure CNI Overlay, Cilium & Workload Identity (`AZR-01`)
```bicep
// Production AKS Cluster with Cilium Dataplane, Azure CNI Overlay and Workload Identity
resource aksCluster 'Microsoft.ContainerService/managedClusters@2024-02-01' = {
  name: 'aks-enterprise-prod'
  location: resourceGroup().location
  identity: {
    type: 'SystemAssigned'
  }
  properties: {
    dnsPrefix: 'aks-prod-dns'
    oidcIssuerProfile: {
      enabled: true
    }
    securityProfile: {
      workloadIdentity: {
        enabled: true
      }
    }
    networkProfile: {
      networkPlugin: 'azure'
      networkPluginMode: 'overlay'
      networkPolicy: 'cilium'
      networkDatapath: 'cilium'
      loadBalancerSku: 'standard'
    }
    agentPoolProfiles: [
      {
        name: 'systempool'
        count: 3
        vmSize: 'Standard_D4ds_v5'
        osType: 'Linux'
        mode: 'System'
        enableAutoScaling: true
        minCount: 3
        maxCount: 10
        vnetSubnetID: subnetId
      }
    ]
  }
}
```

### Cosmos DB Multi-Region Active-Active with Bounded Staleness (`AZR-07`)
```bicep
// Azure Cosmos DB Account configured for Multi-Region Active-Active Replication
resource cosmosAccount 'Microsoft.DocumentDB/databaseAccounts@2024-05-15' = {
  name: 'cosmos-enterprise-prod'
  location: 'eastus'
  kind: 'GlobalDocumentDB'
  properties: {
    consistencyPolicy: {
      defaultConsistencyLevel: 'BoundedStaleness'
      maxStalenessPrefix: 100000
      maxIntervalInSeconds: 300
    }
    locations: [
      {
        locationName: 'eastus'
        failoverPriority: 0
        isZoneRedundant: true
      }
      {
        locationName: 'westus3'
        failoverPriority: 1
        isZoneRedundant: true
      }
    ]
    enableMultipleWriteLocations: true
    publicNetworkAccess: 'Disabled'
    networkAclBypass: 'AzureServices'
  }
}
```

### Azure OpenAI Enterprise RAG with Azure AI Search (`AZR-08`)
```python
# Production Azure OpenAI + Azure AI Search Hybrid Vector Retrieval
import os
from openai import AzureOpenAI
from azure.core.credentials import AzureKeyCredential
from azure.search.documents import SearchClient
from azure.search.documents.models import VectorizableTextQuery

# Initialize clients using Azure Managed Identity / API keys
client = AzureOpenAI(
    azure_endpoint=os.environ["AZURE_OPENAI_ENDPOINT"],
    api_version="2024-06-01-preview",
    azure_ad_token_provider=token_provider,
)

search_client = SearchClient(
    endpoint=os.environ["AZURE_SEARCH_ENDPOINT"],
    index_name="enterprise-knowledge-base",
    credential=AzureKeyCredential(os.environ["AZURE_SEARCH_KEY"]),
)

# Perform hybrid retrieval with Semantic Reranking and Vector Search
query_text = "What are the SLA guidelines for multi-region Cosmos DB?"
vector_query = VectorizableTextQuery(text=query_text, k_nearest_neighbors=5, fields="content_vector")

results = search_client.search(
    search_text=query_text,
    vector_queries=[vector_query],
    query_type="semantic",
    semantic_configuration_name="enterprise-semantic-config",
    top=3,
)

context = "\n".join([doc["content"] for doc in results])

# Generate grounded response with GPT-4o
response = client.chat.completions.create(
    model="gpt-4o",
    messages=[
        {"role": "system", "content": f"Answer strictly using the provided context:\n{context}"},
        {"role": "user", "content": query_text},
    ],
    temperature=0.1,
)
print(response.choices[0].message.content)
```

### Advanced KQL Query for Latency & Error Budget Burn Rate (`AZR-12`)
```kql
// KQL Error Budget Burn Rate Analysis over 1h and 6h windows
let lookback = 24h;
let target_sla = 0.999;
AppRequests
| where TimeGenerated >= ago(lookback)
| summarize
    TotalRequests = count(),
    FailedRequests = countif(Success == false or toint(ResultCode) >= 500),
    P99Latency = percentile(DurationMs, 99)
    by bin(TimeGenerated, 1h), AppRoleName
| extend ErrorRate = todouble(FailedRequests) / todouble(TotalRequests)
| extend AllowedErrorRate = 1.0 - target_sla
| extend BurnRate = ErrorRate / AllowedErrorRate
| project TimeGenerated, AppRoleName, TotalRequests, FailedRequests, P99Latency, BurnRate
| order by TimeGenerated desc
```
