# NetSentinel - Spécification Complète pour Claude Code

> **Document de référence pour le développement avec Claude Code**
> Scanner passif de réseau IT/OT pour audit et surveillance

---

## Table des Matières

1. [Vision du Projet](#1-vision-du-projet)
2. [Architecture Système](#2-architecture-système)
3. [Stack Technologique](#3-stack-technologique)
4. [Structure du Projet](#4-structure-du-projet)
5. [Base de Données](#5-base-de-données)
6. [Module Capture (Rust)](#6-module-capture-rust)
7. [Module Agrégation (Rust)](#7-module-agrégation-rust)
8. [Module API (Python FastAPI)](#8-module-api-python-fastapi)
9. [Module Web UI (Vue.js)](#9-module-web-ui-vuejs)
10. [Module Export (Python)](#10-module-export-python)
11. [Environnement de Développement](#11-environnement-de-développement)
12. [Ordre de Développement](#12-ordre-de-développement)
13. [Tests et Qualité](#13-tests-et-qualité)
14. [Déploiement](#14-déploiement)

---

## 1. Vision du Projet

### 1.1 Objectif Principal

Développer une **appliance de surveillance réseau passive** capable de :
- Analyser le trafic réseau **sans émettre de paquets**
- Générer automatiquement un **inventaire des équipements (CMDB)**
- **Cartographier les flux** de communication
- Fournir une **interface web** pour visualisation et configuration

### 1.2 Cas d'Usage

| Cas d'Usage | Description |
|-------------|-------------|
| **Audit de sécurité** | Déploiement temporaire chez un client pour cartographier l'infrastructure |
| **Surveillance continue** | Installation permanente pour monitoring des flux |
| **Conformité OT** | Analyse des réseaux industriels sans risque d'interférence |
| **Migration réseau** | Identification des dépendances avant changements |

### 1.3 Contraintes Clés

- ✅ **Passif uniquement** : Aucun paquet émis, aucun scan actif
- ✅ **Non-intrusif** : Mode bypass transparent ou écoute passive
- ✅ **Stockage local** : Rétention minimum 30 jours
- ✅ **Résidence des données** : Export cloud vers Québec uniquement (OVH ca-1)
- ✅ **Performance** : Support liens 1 Gbps (≈1.4 Mpps)

---

## 2. Architecture Système

### 2.1 Configuration Matérielle

```
┌─────────────────────────────────────────────────────────────────┐
│                      NetSentinel Appliance                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐                   │
│  │  eth0    │    │  eth1    │    │  eth2    │                   │
│  │ MONITOR1 │    │ MONITOR2 │    │ MGMT     │                   │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘                   │
│       │               │               │                          │
└───────┼───────────────┼───────────────┼──────────────────────────┘
        │               │               │
        ▼               ▼               ▼
   [Analyse L2/L3] [Analyse L2/L3]  [Web UI + API]
```

### 2.2 Modes de Fonctionnement

#### Mode A : Mirror (SPAN/TAP) - **MVP**
```
[Switch] ────SPAN────► [eth0 NetSentinel]
              │
         Port Mirror
```
- Interface en mode promiscuous
- Réception passive du trafic dupliqué

#### Mode B : Bypass (Inline) - **Phase 2**
```
[Switch A] ──eth0──► [NetSentinel] ──eth1──► [Switch B]
                          │
                     Bridge br0
                    (transparent)
```
- Bridge Linux entre eth0 et eth1
- Aucune adresse IP sur le bridge

### 2.3 Flux de Données

```
[Paquet Réseau]
       │
       ▼
┌──────────────────┐
│  AF_PACKET Ring  │  ◄── Zero-copy capture (Rust)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Frame Decoder   │  ◄── Parse Ethernet/VLAN/QinQ/IP/TCP/UDP
└────────┬─────────┘
         │
         ├─────────────────────────────────┐
         │                                 │
         ▼                                 ▼
┌──────────────────┐              ┌──────────────────┐
│  Redis Stream    │              │  PostgreSQL      │
│  (Hot Data)      │              │  (Cold Storage)  │
└────────┬─────────┘              └────────┬─────────┘
         │                                 │
         ▼                                 ▼
┌──────────────────┐              ┌──────────────────┐
│  WebSocket Push  │              │  TimescaleDB     │
│  (Live View)     │              │  (Historical)    │
└──────────────────┘              └──────────────────┘
```

---

## 3. Stack Technologique

| Composant | Technologie | Version | Justification |
|-----------|-------------|---------|---------------|
| **OS** | Debian | 12 Bookworm | Stabilité, support LTS |
| **Capture** | Rust + AF_PACKET | 1.75+ | Performance maximale, zero-copy |
| **Agrégation** | Rust + DashMap | 1.75+ | Lock-free, haute concurrence |
| **Base données** | PostgreSQL + TimescaleDB | 16 + 2.13 | Time-series optimisé |
| **Cache** | Redis | 7.x | Agrégation temps réel, streams |
| **API** | Python + FastAPI | 3.11+ | Rapidité dev, async natif |
| **Frontend** | Vue.js 3 + Tailwind | 3.4+ | SPA réactive, composants modernes |
| **Visualisation** | vis-network | - | Graphes réseau interactifs |

---

## 4. Structure du Projet

```
netsentinel/
├── README.md
├── CLAUDE_CODE_SPEC.md          # Ce fichier
│
├── docs/                         # Documentation détaillée
│   ├── 01-ARCHITECTURE.md
│   ├── 02-DATABASE.md
│   ├── 03-MODULES.md
│   └── 04-DEVELOPMENT.md
│
├── capture/                      # Module Rust de capture
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── config.rs
│       ├── capture/
│       │   ├── mod.rs
│       │   ├── af_packet.rs
│       │   ├── bridge.rs
│       │   └── interface.rs
│       ├── decode/
│       │   ├── mod.rs
│       │   ├── ethernet.rs
│       │   ├── vlan.rs
│       │   ├── ipv4.rs
│       │   └── transport.rs
│       └── output/
│           ├── mod.rs
│           └── redis.rs
│
├── aggregator/                   # Module Rust d'agrégation
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── config.rs
│       ├── state/
│       │   ├── mod.rs
│       │   ├── device.rs
│       │   ├── flow.rs
│       │   └── protocol.rs
│       ├── pipeline/
│       │   ├── mod.rs
│       │   ├── consumer.rs
│       │   └── persister.rs
│       └── db/
│           ├── mod.rs
│           ├── models.rs
│           └── queries.rs
│
├── api/                          # API Python FastAPI
│   ├── pyproject.toml
│   ├── requirements.txt
│   └── app/
│       ├── __init__.py
│       ├── main.py
│       ├── config.py
│       ├── database.py
│       ├── models/
│       ├── schemas/
│       ├── routers/
│       ├── services/
│       ├── websocket/
│       └── utils/
│
├── web/                          # Frontend Vue.js
│   ├── package.json
│   ├── vite.config.js
│   ├── tailwind.config.js
│   └── src/
│       ├── main.js
│       ├── App.vue
│       ├── components/
│       ├── views/
│       ├── stores/
│       ├── services/
│       ├── composables/
│       └── router/
│
├── export/                       # Module d'export cloud
│   ├── pyproject.toml
│   └── export/
│       ├── __init__.py
│       ├── config.py
│       ├── service.py
│       └── scheduler.py
│
├── migrations/                   # Migrations SQL
│   └── 001_initial_schema.sql
│
├── config/                       # Fichiers de configuration
│   └── capture.toml
│
├── docker/                       # Développement Docker
│   └── docker-compose.dev.yml
│
└── systemd/                      # Services systemd
    ├── netsentinel-capture.service
    ├── netsentinel-aggregator.service
    └── netsentinel-api.service
```

---

## 5. Base de Données

### 5.1 Schéma Général

```
┌─────────────────────────────────────────────────────────────────┐
│                     PostgreSQL + TimescaleDB                     │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   CMDB Schema   │  │  Metrics Schema │  │  Config Schema  │  │
│  │   (Inventaire)  │  │  (Time-series)  │  │  (Paramètres)   │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Tables CMDB

#### devices
```sql
CREATE TABLE devices (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mac_address             MACADDR NOT NULL UNIQUE,
    oui_vendor              VARCHAR(128),
    oui_prefix              CHAR(8),
    device_type             VARCHAR(50) DEFAULT 'unknown',
    device_name             VARCHAR(255),
    device_notes            TEXT,
    first_seen              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_packets_sent      BIGINT DEFAULT 0,
    total_packets_received  BIGINT DEFAULT 0,
    total_bytes_sent        BIGINT DEFAULT 0,
    total_bytes_received    BIGINT DEFAULT 0,
    is_gateway              BOOLEAN DEFAULT FALSE,
    is_active               BOOLEAN DEFAULT TRUE,
    is_flagged              BOOLEAN DEFAULT FALSE,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### device_ips
```sql
CREATE TABLE device_ips (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id        UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    ip_address       INET NOT NULL,
    ip_version       SMALLINT NOT NULL DEFAULT 4,
    vlan_id          SMALLINT,
    subnet_mask      INET,
    first_seen       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    packets_sent     BIGINT DEFAULT 0,
    packets_received BIGINT DEFAULT 0,
    bytes_sent       BIGINT DEFAULT 0,
    bytes_received   BIGINT DEFAULT 0,
    CONSTRAINT uq_device_ip_vlan UNIQUE (device_id, ip_address, COALESCE(vlan_id, -1))
);
```

#### vlans
```sql
CREATE TABLE vlans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vlan_id         SMALLINT NOT NULL,
    outer_vlan_id   SMALLINT,  -- QinQ
    vlan_name       VARCHAR(64),
    vlan_description TEXT,
    device_count    INTEGER DEFAULT 0,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_packets   BIGINT DEFAULT 0,
    total_bytes     BIGINT DEFAULT 0,
    CONSTRAINT uq_vlan_ids UNIQUE (vlan_id, COALESCE(outer_vlan_id, -1))
);
```

### 5.3 Tables Metrics (TimescaleDB)

#### traffic_flows
```sql
CREATE TABLE traffic_flows (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    src_device_id   UUID REFERENCES devices(id),
    src_mac         MACADDR NOT NULL,
    src_ip          INET,
    src_port        INTEGER,
    dst_device_id   UUID REFERENCES devices(id),
    dst_mac         MACADDR NOT NULL,
    dst_ip          INET,
    dst_port        INTEGER,
    vlan_id         SMALLINT,
    outer_vlan_id   SMALLINT,
    ethertype       SMALLINT,
    ip_protocol     SMALLINT,
    first_seen      TIMESTAMPTZ NOT NULL,
    last_seen       TIMESTAMPTZ NOT NULL,
    packet_count    BIGINT NOT NULL DEFAULT 0,
    byte_count      BIGINT NOT NULL DEFAULT 0,
    tcp_flags_seen  SMALLINT
);

-- Index unique pour éviter les doublons
CREATE UNIQUE INDEX idx_flows_unique_tuple ON traffic_flows (
    src_mac, 
    COALESCE(src_ip, '0.0.0.0'::inet), 
    COALESCE(src_port, 0),
    dst_mac, 
    COALESCE(dst_ip, '0.0.0.0'::inet), 
    COALESCE(dst_port, 0),
    COALESCE(vlan_id, 0), 
    COALESCE(ip_protocol, 0)
);
```

#### traffic_metrics (Hypertable)
```sql
CREATE TABLE traffic_metrics (
    time            TIMESTAMPTZ NOT NULL,
    bucket_size     INTERVAL NOT NULL,
    flow_id         UUID REFERENCES traffic_flows(id),
    device_id       UUID REFERENCES devices(id),
    metric_type     VARCHAR(20) NOT NULL,  -- 'device_in', 'device_out', 'flow'
    packet_count    BIGINT DEFAULT 0,
    byte_count      BIGINT DEFAULT 0,
    avg_packet_size REAL,
    max_packet_size INTEGER,
    min_packet_size INTEGER,
    tcp_syn_count   INTEGER DEFAULT 0,
    tcp_rst_count   INTEGER DEFAULT 0,
    tcp_fin_count   INTEGER DEFAULT 0
);

-- Conversion en hypertable
SELECT create_hypertable('traffic_metrics', 'time', chunk_time_interval => INTERVAL '1 day');
SELECT add_retention_policy('traffic_metrics', INTERVAL '30 days');
```

### 5.4 Tables Configuration

#### config_settings
```sql
CREATE TABLE config_settings (
    key          VARCHAR(128) PRIMARY KEY,
    value        JSONB NOT NULL,
    value_type   VARCHAR(20) NOT NULL,  -- string, integer, boolean, json
    category     VARCHAR(64) NOT NULL,
    description  TEXT,
    is_sensitive BOOLEAN DEFAULT FALSE
);
```

#### users
```sql
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      VARCHAR(64) NOT NULL UNIQUE,
    email         VARCHAR(255),
    password_hash VARCHAR(255) NOT NULL,  -- bcrypt
    role          VARCHAR(20) NOT NULL DEFAULT 'viewer',  -- admin, operator, viewer
    is_active     BOOLEAN DEFAULT TRUE,
    last_login    TIMESTAMPTZ
);

-- Admin par défaut (password: changeme)
INSERT INTO users (username, password_hash, role) VALUES
    ('admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.mHFHmYCg1HO.3q', 'admin');
```

---

## 6. Module Capture (Rust)

### 6.1 Cargo.toml

```toml
[package]
name = "netsentinel-capture"
version = "0.1.0"
edition = "2021"

[dependencies]
# Capture réseau
pnet = "0.35"
etherparse = "0.15"
libc = "0.2"

# Async runtime
tokio = { version = "1", features = ["full"] }
crossbeam = "0.8"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Redis
redis = { version = "0.25", features = ["tokio-comp", "streams"] }

# Configuration
toml = "0.8"
config = "0.14"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Error handling
anyhow = "1"
thiserror = "1"

[profile.release]
opt-level = 3
lto = true
```

### 6.2 Structures de Données

```rust
// src/capture/frame.rs

use std::net::Ipv4Addr;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Adresse MAC (6 octets)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacAddr([u8; 6]);

impl MacAddr {
    pub fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }
    
    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
    }
    
    pub fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 == 0x01
    }
    
    pub fn oui_prefix(&self) -> String {
        format!("{:02X}:{:02X}:{:02X}", self.0[0], self.0[1], self.0[2])
    }
}

/// Informations VLAN (802.1Q)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlanInfo {
    pub id: u16,
    pub priority: u8,
    pub dei: bool,
}

/// Informations QinQ (802.1ad)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QinQInfo {
    pub outer_vlan: VlanInfo,
    pub inner_vlan: VlanInfo,
}

/// Flags TCP
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TcpFlags {
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
}

impl TcpFlags {
    pub fn from_byte(flags: u8) -> Self {
        Self {
            fin: flags & 0x01 != 0,
            syn: flags & 0x02 != 0,
            rst: flags & 0x04 != 0,
            psh: flags & 0x08 != 0,
            ack: flags & 0x10 != 0,
            urg: flags & 0x20 != 0,
        }
    }
    
    pub fn to_byte(&self) -> u8 {
        let mut flags = 0u8;
        if self.fin { flags |= 0x01; }
        if self.syn { flags |= 0x02; }
        if self.rst { flags |= 0x04; }
        if self.psh { flags |= 0x08; }
        if self.ack { flags |= 0x10; }
        if self.urg { flags |= 0x20; }
        flags
    }
}

/// Trame capturée complète
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedFrame {
    pub timestamp: DateTime<Utc>,
    pub interface: String,
    
    // Layer 2
    pub src_mac: MacAddr,
    pub dst_mac: MacAddr,
    pub ethertype: u16,
    pub vlan: Option<VlanInfo>,
    pub qinq: Option<QinQInfo>,
    
    // Layer 3
    pub src_ip: Option<Ipv4Addr>,
    pub dst_ip: Option<Ipv4Addr>,
    pub ip_protocol: Option<u8>,
    pub ttl: Option<u8>,
    
    // Layer 4
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub tcp_flags: Option<TcpFlags>,
    
    // Metadata
    pub frame_size: u32,
    pub payload_size: u32,
}
```

### 6.3 Configuration (TOML)

```toml
# /opt/netsentinel/config/capture.toml

[capture]
mode = "mirror"  # ou "bypass"
ring_buffer_size = 8192
snap_length = 1518
flush_interval_ms = 100
batch_size = 1000

[[capture.interfaces]]
name = "eth0"
promiscuous = true

[[capture.interfaces]]
name = "eth1"
promiscuous = true

[redis]
url = "redis://127.0.0.1:6379"
stream_name = "netsentinel:frames"
max_stream_length = 100000

[logging]
level = "info"
file = "/var/log/netsentinel/capture.log"
```

### 6.4 Ordre d'Implémentation

1. `src/config.rs` - Parsing de la configuration TOML
2. `src/capture/interface.rs` - Gestion des interfaces réseau
3. `src/capture/af_packet.rs` - Capture via AF_PACKET (PACKET_MMAP)
4. `src/decode/ethernet.rs` - Décodage Ethernet de base
5. `src/decode/vlan.rs` - Support VLAN (802.1Q) et QinQ (802.1ad)
6. `src/decode/ipv4.rs` - Décodage IPv4
7. `src/decode/transport.rs` - Décodage TCP/UDP (ports uniquement)
8. `src/output/redis.rs` - Envoi vers Redis Streams
9. `src/main.rs` - Assemblage et boucle principale
10. `src/capture/bridge.rs` - Mode bypass (Phase 2)

---

## 7. Module Agrégation (Rust)

### 7.1 Cargo.toml

```toml
[package]
name = "netsentinel-aggregator"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "macaddr"] }
redis = { version = "0.25", features = ["tokio-comp", "streams"] }
dashmap = "5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1"
thiserror = "1"
toml = "0.8"
```

### 7.2 Structures de Données

```rust
// src/state/mod.rs

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use chrono::{DateTime, Utc};

/// État d'un device en mémoire
pub struct DeviceState {
    pub mac: MacAddr,
    pub first_seen: DateTime<Utc>,
    pub last_seen: AtomicU64,  // Unix timestamp
    
    // Compteurs atomiques
    pub packets_sent: AtomicU64,
    pub packets_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    
    // IPs associées
    pub ips: DashMap<Ipv4Addr, IpState>,
}

/// Clé unique pour un flow
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct FlowKey {
    pub src_mac: MacAddr,
    pub dst_mac: MacAddr,
    pub src_ip: Option<Ipv4Addr>,
    pub dst_ip: Option<Ipv4Addr>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub vlan_id: Option<u16>,
    pub protocol: Option<u8>,
}

/// État d'un flow en mémoire
pub struct FlowState {
    pub key: FlowKey,
    pub first_seen: DateTime<Utc>,
    pub last_seen: AtomicU64,
    pub packet_count: AtomicU64,
    pub byte_count: AtomicU64,
    pub tcp_flags_seen: AtomicU8,
}

/// État global de l'agrégateur
pub struct AggregatorState {
    pub devices: DashMap<MacAddr, DeviceState>,
    pub flows: DashMap<FlowKey, FlowState>,
    pub protocols: DashMap<(u16, Option<u8>), ProtocolStats>,
    
    // Métriques globales
    pub total_packets: AtomicU64,
    pub total_bytes: AtomicU64,
    pub start_time: DateTime<Utc>,
}
```

### 7.3 Pipeline de Traitement

```rust
// src/pipeline/mod.rs

impl Pipeline {
    pub async fn run(&self) -> Result<()> {
        // 1. Consumer Redis en parallèle
        let ingest_handle = tokio::spawn(self.run_ingest());
        
        // 2. Aggregation (reçoit depuis ingest)
        let aggregate_handle = tokio::spawn(self.run_aggregate());
        
        // 3. Persistence périodique (60s)
        let persist_handle = tokio::spawn(self.run_persist());
        
        // 4. Publication événements (nouveaux devices/flows)
        let event_handle = tokio::spawn(self.run_event_publisher());
        
        tokio::try_join!(ingest_handle, aggregate_handle, persist_handle, event_handle)?;
        Ok(())
    }
}
```

---

## 8. Module API (Python FastAPI)

### 8.1 Requirements

```txt
fastapi>=0.109.0
uvicorn[standard]>=0.27.0
sqlalchemy[asyncio]>=2.0.0
asyncpg>=0.29.0
redis>=5.0.0
pydantic>=2.5.0
pydantic-settings>=2.1.0
python-jose[cryptography]>=3.3.0
passlib[bcrypt]>=1.7.4
python-multipart>=0.0.6
httpx>=0.26.0
```

### 8.2 Structure

```
api/
├── app/
│   ├── __init__.py
│   ├── main.py              # FastAPI app
│   ├── config.py            # Pydantic settings
│   ├── database.py          # Async SQLAlchemy
│   │
│   ├── models/              # SQLAlchemy ORM
│   │   ├── device.py
│   │   ├── flow.py
│   │   └── user.py
│   │
│   ├── schemas/             # Pydantic validation
│   │   ├── device.py
│   │   ├── flow.py
│   │   └── stats.py
│   │
│   ├── routers/             # API endpoints
│   │   ├── devices.py
│   │   ├── flows.py
│   │   ├── stats.py
│   │   ├── topology.py
│   │   └── auth.py
│   │
│   ├── services/            # Business logic
│   │   ├── device_service.py
│   │   ├── flow_service.py
│   │   └── topology_service.py
│   │
│   └── websocket/
│       └── live.py          # Real-time streaming
```

### 8.3 Endpoints API

```
# Devices
GET  /api/v1/devices                 # Liste paginée avec filtres
GET  /api/v1/devices/{id}            # Détail device
PATCH /api/v1/devices/{id}           # Mise à jour métadonnées
GET  /api/v1/devices/{id}/flows      # Flows d'un device
GET  /api/v1/devices/{id}/bandwidth  # Historique bande passante

# Flows
GET  /api/v1/flows                   # Liste avec filtres
GET  /api/v1/flows/{id}              # Détail flow

# Topology
GET  /api/v1/topology                # Données graphe (nodes + edges)
GET  /api/v1/topology/matrix         # Matrice de communication

# Stats
GET  /api/v1/stats/dashboard         # KPIs dashboard
GET  /api/v1/stats/bandwidth         # Bande passante globale
GET  /api/v1/stats/protocols         # Distribution protocoles

# WebSocket
WS   /api/v1/ws/live                 # Événements temps réel

# Export
GET  /api/v1/export/cmdb             # Export CMDB (CSV/JSON)
```

### 8.4 Schémas Pydantic

```python
# app/schemas/device.py

from pydantic import BaseModel, Field
from datetime import datetime
from typing import Optional, List
from uuid import UUID
from ipaddress import IPv4Address

class DeviceResponse(BaseModel):
    id: UUID
    mac_address: str
    oui_vendor: Optional[str]
    device_type: Optional[str]
    device_name: Optional[str]
    first_seen: datetime
    last_seen: datetime
    is_active: bool
    is_flagged: bool
    total_packets_sent: int
    total_packets_received: int
    total_bytes_sent: int
    total_bytes_received: int
    ip_addresses: List[str]
    vlans: List[int]
    
    class Config:
        from_attributes = True

class DeviceListResponse(BaseModel):
    items: List[DeviceResponse]
    total: int
    page: int
    page_size: int
    pages: int
```

---

## 9. Module Web UI (Vue.js)

### 9.1 Package.json (dépendances clés)

```json
{
  "dependencies": {
    "vue": "^3.4.0",
    "vue-router": "^4.2.0",
    "pinia": "^2.1.0",
    "axios": "^1.6.0",
    "chart.js": "^4.4.0",
    "vis-network": "^9.1.0"
  },
  "devDependencies": {
    "vite": "^5.0.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }
}
```

### 9.2 Structure

```
web/src/
├── main.js
├── App.vue
├── router/
│   └── index.js
├── stores/
│   ├── auth.js
│   ├── devices.js
│   ├── flows.js
│   └── stats.js
├── services/
│   ├── api.js
│   └── websocket.js
├── composables/
│   ├── useDevices.js
│   ├── useFlows.js
│   └── useWebSocket.js
├── views/
│   ├── DashboardView.vue
│   ├── DevicesView.vue
│   ├── FlowsView.vue
│   ├── TopologyView.vue
│   └── SettingsView.vue
└── components/
    ├── common/
    │   ├── LoadingSpinner.vue
    │   ├── Pagination.vue
    │   └── DataTable.vue
    ├── dashboard/
    │   ├── StatsCard.vue
    │   ├── BandwidthChart.vue
    │   └── TopTalkersWidget.vue
    ├── devices/
    │   ├── DeviceList.vue
    │   └── DeviceDetail.vue
    └── topology/
        ├── NetworkGraph.vue
        └── NodePopup.vue
```

### 9.3 Composant NetworkGraph.vue (vis-network)

```vue
<template>
  <div class="relative h-full">
    <!-- Contrôles -->
    <div class="absolute top-4 left-4 z-10 flex gap-2">
      <select v-model="layout" @change="updateLayout">
        <option value="force">Force-directed</option>
        <option value="hierarchical">Hiérarchique</option>
        <option value="circular">Circulaire</option>
      </select>
      <select v-model="vlanFilter">
        <option :value="null">Tous les VLANs</option>
        <option v-for="v in vlans" :key="v" :value="v">VLAN {{ v }}</option>
      </select>
    </div>
    
    <!-- Container du graphe -->
    <div ref="networkContainer" class="w-full h-full"></div>
    
    <!-- Popup détail -->
    <NodePopup v-if="selectedNode" :node="selectedNode" :position="popupPosition" />
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Network, DataSet } from 'vis-network/standalone'

const options = {
  nodes: {
    shape: 'dot',
    size: 20,
    font: { size: 12, color: '#ffffff' },
  },
  edges: {
    smooth: { type: 'continuous' }
  },
  physics: {
    barnesHut: {
      gravitationalConstant: -2000,
      springLength: 95
    }
  },
  groups: {
    router: { color: '#ef4444', shape: 'diamond' },
    switch: { color: '#f97316', shape: 'square' },
    server: { color: '#3b82f6', shape: 'dot' },
    workstation: { color: '#22c55e', shape: 'dot' },
    iot: { color: '#a855f7', shape: 'triangle' },
    unknown: { color: '#6b7280', shape: 'dot' }
  }
}
</script>
```

---

## 10. Module Export (Python)

### 10.1 Configuration

```python
# export/config.py

from pydantic import BaseSettings

class ExportConfig(BaseSettings):
    enabled: bool = False
    schedule: str = "0 2 * * *"  # 2h du matin
    
    # OVH Object Storage (Québec)
    s3_endpoint: str = "https://s3.ca-1.cloud.ovh.net"
    s3_bucket: str = "netsentinel-exports"
    s3_access_key: str
    s3_secret_key: str
    
    # Chiffrement
    encryption_enabled: bool = True
    encryption_key: Optional[str] = None
    
    # Options
    export_format: str = "json"  # json, csv
    compress: bool = True
    
    class Config:
        env_prefix = "NETSENTINEL_EXPORT_"
```

### 10.2 Service d'Export

```python
# export/service.py

import boto3
import gzip
import json
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

class ExportService:
    async def run_export(self):
        """Export quotidien vers OVH Object Storage."""
        timestamp = datetime.utcnow()
        date_str = timestamp.strftime("%Y-%m-%d")
        
        for entity_type, data in [
            ("devices", await self.export_devices()),
            ("flows", await self.export_flows()),
        ]:
            content = json.dumps(data, default=str).encode()
            
            if self.config.compress:
                content = gzip.compress(content)
            
            if self.config.encryption_enabled:
                nonce = os.urandom(12)
                content = nonce + self.cipher.encrypt(nonce, content, None)
            
            self.s3.put_object(
                Bucket=self.config.s3_bucket,
                Key=f"{date_str}/{entity_type}.json.gz.enc",
                Body=content
            )
```

---

## 11. Environnement de Développement

### 11.1 Docker Compose

```yaml
# docker/docker-compose.dev.yml

version: '3.8'

services:
  postgres:
    image: timescale/timescaledb:latest-pg16
    environment:
      POSTGRES_DB: netsentinel
      POSTGRES_USER: netsentinel
      POSTGRES_PASSWORD: devpassword
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ../migrations:/docker-entrypoint-initdb.d

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data

  adminer:
    image: adminer
    ports:
      - "8080:8080"

volumes:
  postgres_data:
  redis_data:
```

### 11.2 Commandes de Développement

```bash
# Docker
docker compose -f docker/docker-compose.dev.yml up -d
docker compose logs -f postgres

# Base de données
psql -h localhost -U netsentinel -d netsentinel -f migrations/001_initial_schema.sql

# Rust (Capture)
cd capture
cargo build --release
cargo test
cargo clippy -- -D warnings

# Rust (Aggregator)
cd aggregator
cargo build --release
cargo test

# Python (API)
cd api
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000

# Vue.js (Web)
cd web
npm install
npm run dev
npm run build
```

---

## 12. Ordre de Développement

### Phase 1 : MVP (Semaines 1-6)

| Semaine | Tâche | Module |
|---------|-------|--------|
| 1 | Setup Docker, migrations SQL | Infra |
| 2 | Capture basique Ethernet/IPv4 | Rust |
| 3 | Support VLAN, Redis output | Rust |
| 4 | Agrégateur, persistence PostgreSQL | Rust |
| 5 | API CRUD devices/flows | Python |
| 6 | Web UI dashboard, liste devices | Vue.js |

### Phase 2 : Core Features (Semaines 6-10)

- Support QinQ (802.1ad)
- Visualisation topologie réseau
- Export CMDB (CSV/JSON)
- Gestion rétention automatique

### Phase 3 : Production Ready (Semaines 10-12)

- Mode bypass avec bridge
- Export cloud OVH
- Interface de configuration complète
- Packaging Debian
- Tests de charge

---

## 13. Tests et Qualité

### 13.1 Tests Unitaires Rust

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_vlan_frame() {
        let raw = hex::decode(
            "ffffffffffff"  // dst mac (broadcast)
            "001122334455"  // src mac
            "8100"          // 802.1Q ethertype
            "0064"          // VLAN ID 100
            "0800"          // IPv4
        ).unwrap();
        
        let frame = parse_ethernet_frame(&raw).unwrap();
        
        assert!(frame.vlan.is_some());
        assert_eq!(frame.vlan.unwrap().id, 100);
    }
    
    #[test]
    fn test_parse_qinq_frame() {
        let raw = hex::decode(
            "ffffffffffff001122334455"
            "88a8"  // 802.1ad (QinQ outer)
            "00c8"  // Outer VLAN 200
            "8100"  // 802.1Q (inner)
            "0064"  // Inner VLAN 100
            "0800"  // IPv4
        ).unwrap();
        
        let frame = parse_ethernet_frame(&raw).unwrap();
        
        assert!(frame.qinq.is_some());
        assert_eq!(frame.qinq.unwrap().outer_vlan.id, 200);
        assert_eq!(frame.qinq.unwrap().inner_vlan.id, 100);
    }
}
```

### 13.2 Tests API Python

```python
# tests/test_devices.py

import pytest
from httpx import AsyncClient

@pytest.mark.asyncio
async def test_list_devices(client: AsyncClient):
    response = await client.get("/api/v1/devices")
    assert response.status_code == 200
    data = response.json()
    assert "items" in data
    assert "total" in data

@pytest.mark.asyncio
async def test_get_device_not_found(client: AsyncClient):
    response = await client.get("/api/v1/devices/00000000-0000-0000-0000-000000000000")
    assert response.status_code == 404
```

---

## 14. Déploiement

### 14.1 Services Systemd

```ini
# /etc/systemd/system/netsentinel-capture.service

[Unit]
Description=NetSentinel Packet Capture Service
After=network.target redis.service
Requires=redis.service

[Service]
Type=simple
User=root
Group=netsentinel
ExecStart=/opt/netsentinel/bin/netsentinel-capture --config /opt/netsentinel/config/capture.toml
Restart=always
RestartSec=5
AmbientCapabilities=CAP_NET_RAW CAP_NET_ADMIN
LimitNOFILE=65535
LimitMEMLOCK=infinity

[Install]
WantedBy=multi-user.target
```

### 14.2 Structure de Déploiement

```
/opt/netsentinel/
├── bin/
│   ├── netsentinel-capture
│   ├── netsentinel-aggregator
│   └── netsentinel-api
├── config/
│   ├── netsentinel.toml
│   └── capture.toml
├── web/
│   └── dist/
├── data/
│   └── oui.db
└── logs/

/var/lib/netsentinel/
├── db/
└── export/
```

### 14.3 Checklist Production

- [ ] Certificats TLS configurés
- [ ] Firewall restrictif (port management uniquement)
- [ ] Backup automatisé
- [ ] Monitoring en place
- [ ] Logs rotés
- [ ] Mot de passe admin changé

---

## Conventions de Code

### Rust
- `anyhow::Result` pour les erreurs applicatives
- `thiserror` pour les erreurs de librairie
- Documentation `///` pour les fonctions publiques
- `cargo clippy` sans warnings

### Python
- Type hints obligatoires
- Docstrings Google style
- Formatage Black
- Linting Ruff

### Vue.js
- Composition API uniquement
- `<script setup>` syntax
- Props avec types et valeurs par défaut
- Tailwind CSS pour le styling

---

## Points d'Attention Performance

1. **Capture** : Ne jamais allouer dans la boucle chaude, utiliser ring buffer
2. **Agrégation** : DashMap pour éviter les locks globaux
3. **API** : Requêtes async, cache Redis (TTL 5 min)
4. **DB** : Index sur colonnes WHERE/ORDER BY, hypertables compressés

---

> **Note pour Claude Code** : Ce document contient toutes les spécifications nécessaires pour implémenter NetSentinel. Commencer par le setup Docker et les migrations SQL, puis suivre l'ordre de développement indiqué. Les fichiers de documentation détaillés sont disponibles dans `/home/claude/netscanner/docs/`.
