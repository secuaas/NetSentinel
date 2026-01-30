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
| **capture** | Rust | Capture passive AF_PACKET | ✅ Complet |
| **aggregator** | Rust | Agrégation et persistence | ✅ Complet |
| **api** | Python FastAPI | API REST + WebSocket | ✅ Complet |
| **web** | Vue.js 3 | Interface web | ✅ Complet |

## Déploiement Docker (Debian Embarqué)

### Prérequis

- Debian 11+ ou Ubuntu 22.04+
- Docker 24+
- Docker Compose v2+
- 2 Go RAM minimum
- Accès réseau (interface à surveiller)

### Installation rapide

```bash
# 1. Cloner le repository
git clone https://github.com/secuaas/NetSentinel.git
cd NetSentinel

# 2. Lancer l'installation
sudo ./deploy.sh install
```

Le script va :
- Vérifier les dépendances
- Générer des mots de passe sécurisés
- Builder les images Docker
- Créer un service systemd

### Configuration

Avant de démarrer, éditez les fichiers de configuration :

```bash
# Variables d'environnement
sudo nano /opt/netsentinel/.env

# Interface réseau à surveiller
sudo nano /opt/netsentinel/config/capture.docker.toml
```

**Configuration de l'interface réseau** (`capture.docker.toml`) :

```toml
[[capture.interfaces]]
name = "eth0"           # Adapter à votre interface
promiscuous = true
description = "Primary monitoring interface"
```

### Démarrage

```bash
# Démarrer NetSentinel
sudo systemctl start netsentinel

# Vérifier le status
sudo systemctl status netsentinel

# Ou via le script
sudo ./deploy.sh status
```

### Accès Web

Ouvrez votre navigateur : `http://<adresse-ip>:8080`

**Identifiants par défaut :**
- Utilisateur : `admin`
- Mot de passe : `changeme`

### Commandes utiles

```bash
# Démarrer/arrêter
sudo ./deploy.sh start
sudo ./deploy.sh stop
sudo ./deploy.sh restart

# Voir les logs
sudo ./deploy.sh logs              # Tous les services
sudo ./deploy.sh logs capture      # Service spécifique
sudo ./deploy.sh logs api

# Statut et ressources
sudo ./deploy.sh status

# Mise à jour
sudo ./deploy.sh update

# Désinstallation
sudo ./deploy.sh uninstall
```

## Installation manuelle (développement)

### Prérequis

- Rust 1.75+
- Python 3.11+
- Node.js 20+
- PostgreSQL 16 + TimescaleDB
- Redis 7.x

### 1. Base de données

```bash
# Démarrer les services avec Docker
cd docker
docker compose -f docker-compose.dev.yml up -d

# Appliquer les migrations
psql -h localhost -U netsentinel -d netsentinel -f migrations/01_init.sql
```

### 2. Modules Rust

```bash
# Capture
cd capture && cargo build --release
cp target/release/netsentinel-capture ../bin/

# Aggregator
cd ../aggregator && cargo build --release
cp target/release/netsentinel-aggregator ../bin/
```

### 3. API Python

```bash
cd api
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
```

### 4. Frontend Vue.js

```bash
cd web
npm install
npm run dev     # Développement
npm run build   # Production
```

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/devices` | Liste des appareils |
| `GET /api/v1/devices/{id}` | Détail d'un appareil |
| `PATCH /api/v1/devices/{id}` | Modifier un appareil |
| `GET /api/v1/flows` | Liste des flux |
| `GET /api/v1/topology` | Topologie réseau |
| `GET /api/v1/alerts` | Liste des alertes |
| `GET /api/v1/stats/dashboard` | Statistiques dashboard |
| `POST /api/v1/auth/token` | Authentification |
| `WS /ws/events` | WebSocket temps réel |

## Structure des fichiers

```
NetSentinel/
├── capture/            # Module Rust - Capture AF_PACKET
├── aggregator/         # Module Rust - Agrégation
├── api/                # Python FastAPI
├── web/                # Vue.js 3 Frontend
├── config/             # Fichiers de configuration
│   ├── capture.toml
│   ├── capture.docker.toml
│   ├── aggregator.toml
│   └── aggregator.docker.toml
├── migrations/         # Schéma PostgreSQL
├── docker/             # Configs Docker
│   ├── nginx.conf
│   └── docker-compose.dev.yml
├── systemd/            # Services systemd
├── Dockerfile          # Multi-stage build
├── docker-compose.yml  # Production compose
├── deploy.sh           # Script de déploiement
└── .env.example        # Variables d'environnement
```

## Sécurité

- Changez le mot de passe admin après la première connexion
- Utilisez des mots de passe forts dans `.env`
- Limitez l'accès au port 8080 (firewall)
- Le module capture nécessite les capabilities `NET_ADMIN` et `NET_RAW`

## Licence

Proprietary - SecuAAS
