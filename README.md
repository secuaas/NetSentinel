# NetSentinel

Scanner passif de réseau IT/OT pour audit et surveillance.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         NetSentinel                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   Capture    │ -> │  Aggregator  │ -> │     API      │       │
│  │    (Rust)    │    │    (Rust)    │    │   (Python)   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│         │                   │                   │                │
│         v                   v                   v                │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │    Redis     │    │  PostgreSQL  │    │   Web UI     │       │
│  │   Streams    │    │ + TimescaleDB│    │   (Vue.js)   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Modules

| Module | Stack | Description | Status |
|--------|-------|-------------|--------|
| **capture** | Rust | Capture passive AF_PACKET | ✅ Compilé |
| **aggregator** | Rust | Agrégation et persistence | ✅ Compilé |
| **api** | Python FastAPI | API REST | ✅ Créé |
| **web** | Vue.js 3 | Interface web | 🚧 À faire |

## Prérequis

- Rust 1.75+
- Python 3.11+
- PostgreSQL 16 + TimescaleDB
- Redis 7.x
- Node.js 20+ (pour le frontend)

## Installation

### 1. Base de données

```bash
# Démarrer les services avec Docker
cd docker
docker compose -f docker-compose.dev.yml up -d

# Appliquer les migrations
psql -h localhost -U netsentinel -d netsentinel -f migrations/001_initial_schema.sql
```

### 2. Modules Rust

```bash
# Capture
cd capture
cargo build --release

# Aggregator
cd ../aggregator
cargo build --release

# Copier les binaires
cp target/release/netsentinel-capture ../bin/
cp target/release/netsentinel-aggregator ../bin/
```

### 3. API Python

```bash
cd api
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Démarrer
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
```

## Configuration

### capture.toml

```toml
[capture]
mode = "mirror"

[[capture.interfaces]]
name = "eth0"
promiscuous = true

[redis]
url = "redis://127.0.0.1:6379"
stream_name = "netsentinel:frames"
```

### aggregator.toml

```toml
[redis]
url = "redis://127.0.0.1:6379"
stream_name = "netsentinel:frames"

[database]
url = "postgresql://netsentinel:netsentinel@localhost:5432/netsentinel"

[aggregation]
persist_interval_secs = 60
```

## Utilisation

```bash
# Lister les interfaces disponibles
./bin/netsentinel-capture --list-interfaces

# Démarrer la capture (nécessite root)
sudo ./bin/netsentinel-capture -c config/capture.toml

# Démarrer l'agrégateur
./bin/netsentinel-aggregator -c config/aggregator.toml

# Démarrer l'API
cd api && uvicorn app.main:app --host 0.0.0.0 --port 8000
```

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/devices` | Liste des appareils |
| `GET /api/v1/devices/{id}` | Détail d'un appareil |
| `PATCH /api/v1/devices/{id}` | Modifier un appareil |
| `GET /api/v1/flows` | Liste des flux |
| `GET /api/v1/stats/dashboard` | Statistiques dashboard |
| `POST /api/v1/auth/token` | Authentification |

## Licence

Proprietary - SecuAAS
