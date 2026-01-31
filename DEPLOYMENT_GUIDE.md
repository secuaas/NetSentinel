# NetSentinel - Guide de Déploiement et Tests

> Guide complet pour déployer et tester NetSentinel en environnement de production

## Table des Matières

1. [Prérequis](#prérequis)
2. [Installation Rapide](#installation-rapide)
3. [Configuration](#configuration)
4. [Déploiement Docker](#déploiement-docker)
5. [Tests de Validation](#tests-de-validation)
6. [Troubleshooting](#troubleshooting)
7. [Maintenance](#maintenance)

---

## Prérequis

### Système d'Exploitation
- **OS**: Debian 11+ ou Ubuntu 22.04+
- **Architecture**: x86_64 (AMD64)
- **RAM**: 2 Go minimum, 4 Go recommandé
- **Stockage**: 20 Go minimum (pour rétention 30 jours)
- **CPU**: 2 cores minimum

### Logiciels Requis
```bash
# Vérifier les versions
docker --version          # 24.0+
docker compose version    # 2.20+
```

### Permissions Réseau
- Accès interface réseau en mode promiscuous (pour capture)
- Port 8080 disponible (interface web)
- Ports internes Docker : 5432 (PostgreSQL), 6379 (Redis), 8000 (API)

### Installation Docker (si nécessaire)
```bash
# Installation automatique Docker
curl -fsSL https://get.docker.com | sh

# Ajouter l'utilisateur au groupe docker
sudo usermod -aG docker $USER
newgrp docker
```

---

## Installation Rapide

### Méthode 1 : Script Automatique (Recommandé)

```bash
# 1. Cloner le repository
git clone https://github.com/secuaas/NetSentinel.git
cd NetSentinel

# 2. Lancer l'installation
sudo ./deploy.sh install

# 3. Configurer l'interface réseau à surveiller
sudo nano /opt/netsentinel/.env
# Modifier: CAPTURE_INTERFACES=eth0  (adapter à votre interface)

# 4. Démarrer NetSentinel
sudo ./deploy.sh start

# 5. Vérifier le statut
sudo ./deploy.sh status
```

### Méthode 2 : Manuel Docker Compose

```bash
# 1. Cloner le repository
git clone https://github.com/secuaas/NetSentinel.git
cd NetSentinel

# 2. Créer le fichier .env
cp .env.example .env

# 3. Générer des secrets sécurisés
echo "SECRET_KEY=$(openssl rand -hex 32)" >> .env
echo "DB_PASSWORD=$(openssl rand -hex 16)" >> .env

# 4. Configurer l'interface réseau
nano .env
# Ajouter: CAPTURE_INTERFACES=eth0

# 5. Démarrer avec Docker Compose
docker compose up -d

# 6. Vérifier les logs
docker compose logs -f
```

---

## Configuration

### Fichier .env Principal

```bash
# Database
DB_PASSWORD=<généré_automatiquement>

# API Secret Key
SECRET_KEY=<généré_automatiquement>

# Web interface port
WEB_PORT=8080

# Network interface to capture (comma-separated for multiple)
CAPTURE_INTERFACES=eth0

# Log level (trace, debug, info, warn, error)
LOG_LEVEL=info
```

### Configuration Capture (Avancée)

**Fichier**: `config/capture.docker.toml`

```toml
[capture]
mode = "mirror"  # ou "bypass" (Phase 2)
ring_buffer_size = 8192
snap_length = 1518
flush_interval_ms = 100
batch_size = 1000

[[capture.interfaces]]
name = "eth0"           # ADAPTER À VOTRE INTERFACE
promiscuous = true
description = "Primary monitoring interface"

[redis]
url = "redis://redis:6379"
stream_name = "netsentinel:frames"
max_stream_length = 100000

[logging]
level = "info"
```

### Identifier vos Interfaces Réseau

```bash
# Lister toutes les interfaces
ip link show

# Exemple de sortie:
# 1: lo: <LOOPBACK,UP,LOWER_UP>
# 2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP>  ← Interface principale
# 3: wlan0: <BROADCAST,MULTICAST,UP,LOWER_UP>

# Vérifier le trafic sur une interface
sudo tcpdump -i eth0 -c 10
```

---

## Déploiement Docker

### Architecture des Conteneurs

```
┌─────────────────────────────────────────────────────┐
│  netsentinel-web (nginx:alpine)                      │
│  Port: 8080 → 80                                     │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│  netsentinel-api (python:3.11)                       │
│  Port: 8000                                          │
└─────────────────┬───────────────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
┌───▼──────┐ ┌───▼──────┐ ┌───▼──────────────────────┐
│ postgres │ │  redis   │ │ netsentinel-capture      │
│ (DB)     │ │ (cache)  │ │ (network_mode=host)      │
└──────────┘ └──────────┘ └──────────────────────────┘
                  ▲
                  │
       ┌──────────┴──────────┐
       │  netsentinel-       │
       │  aggregator         │
       └─────────────────────┘
```

### Build Multi-Stage

Le Dockerfile utilise un build multi-stage pour optimiser la taille:

1. **rust-builder**: Compile capture + aggregator
2. **web-builder**: Build frontend Vue.js
3. **capture**: Runtime Rust + capabilities réseau
4. **aggregator**: Runtime Rust + DB
5. **api**: Runtime Python FastAPI
6. **web**: Runtime Nginx + SPA

### Commandes Docker Compose

```bash
# Démarrer tous les services
docker compose up -d

# Voir les logs en temps réel
docker compose logs -f

# Logs d'un service spécifique
docker compose logs -f capture
docker compose logs -f api

# Statut des conteneurs
docker compose ps

# Redémarrer un service
docker compose restart capture

# Arrêter tous les services
docker compose down

# Arrêter et supprimer les volumes (⚠️ perte de données)
docker compose down -v
```

---

## Tests de Validation

### 1. Vérification Santé des Services

```bash
# Vérifier que tous les conteneurs sont UP
docker compose ps

# Output attendu:
# NAME                   STATUS              PORTS
# netsentinel-api        Up 2 minutes        8000/tcp
# netsentinel-capture    Up 2 minutes
# netsentinel-aggregator Up 2 minutes
# netsentinel-db         Up 2 minutes (healthy)
# netsentinel-redis      Up 2 minutes (healthy)
# netsentinel-web        Up 2 minutes        0.0.0.0:8080->80/tcp
```

### 2. Test Connectivité Base de Données

```bash
# Se connecter à PostgreSQL
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel

# Dans psql:
\dt              # Lister les tables
SELECT COUNT(*) FROM devices;  # Compter les devices découverts
\q               # Quitter
```

### 3. Test Redis Streams

```bash
# Vérifier le stream de capture
docker exec -it netsentinel-redis redis-cli

# Dans redis-cli:
XINFO STREAM netsentinel:frames
XLEN netsentinel:frames  # Nombre de frames dans le stream
quit
```

### 4. Test API REST

```bash
# Health check
curl http://localhost:8080/api/health
# Attendu: {"status":"healthy","version":"0.1.0"}

# Authentification (créer un token)
curl -X POST http://localhost:8080/api/v1/auth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=admin&password=changeme"
# Attendu: {"access_token":"...", "token_type":"bearer"}

# Liste des devices (après quelques minutes de capture)
curl http://localhost:8080/api/v1/devices \
  -H "Authorization: Bearer <token>"
# Attendu: {"items": [...], "total": X, "page": 1}
```

### 5. Test Interface Web

1. **Ouvrir le navigateur**: `http://<adresse-ip>:8080`

2. **Page de Login**:
   - Username: `admin`
   - Password: `changeme`

3. **Dashboard** (après login):
   - Vérifier les KPIs (Total Devices, Active Flows, etc.)
   - Graphiques de bande passante
   - Top Talkers widget

4. **Page Devices**:
   - Liste des appareils découverts
   - Filtrer par type, VLAN
   - Cliquer sur un device pour détails

5. **Page Topology**:
   - Graphe réseau interactif (vis-network)
   - Filtrer par VLAN
   - Changer le layout (Force-directed, Hiérarchique, Circulaire)

6. **Page Flows**:
   - Liste des flux de communication
   - Filtrer par protocole, ports
   - Recherche par IP/MAC

### 6. Test Capture Passive

```bash
# Vérifier que le module capture fonctionne
docker compose logs capture | tail -50

# Logs attendus (exemples):
# [INFO] Starting NetSentinel Capture v0.1.0
# [INFO] Listening on interface: eth0 (promiscuous mode)
# [INFO] Redis connection established
# [INFO] Captured 1000 frames in 1.2s (833 fps)
# [INFO] Sent batch of 1000 frames to Redis

# Générer du trafic pour tester
ping -c 10 8.8.8.8
curl https://www.google.com

# Vérifier les logs aggregator
docker compose logs aggregator | tail -50

# Logs attendus:
# [INFO] Starting NetSentinel Aggregator v0.1.0
# [INFO] Connected to PostgreSQL
# [INFO] Consuming from Redis stream: netsentinel:frames
# [INFO] Discovered new device: 00:11:22:33:44:55 (192.168.1.100)
# [INFO] New flow: 192.168.1.100:54321 -> 8.8.8.8:443 (TCP)
```

### 7. Test Performance

```bash
# Vérifier la charge CPU/RAM
docker stats

# Output attendu (exemple):
# CONTAINER           CPU %   MEM USAGE / LIMIT
# netsentinel-capture    5%    50MB / 2GB
# netsentinel-aggregator 3%    150MB / 2GB
# netsentinel-api        2%    100MB / 2GB
# netsentinel-db        10%    300MB / 2GB
# netsentinel-redis      1%    50MB / 256MB
# netsentinel-web       <1%    20MB / 2GB
```

### 8. Test Export CMDB (Optionnel)

```bash
# Export des devices en CSV
curl http://localhost:8080/api/v1/export/cmdb?format=csv \
  -H "Authorization: Bearer <token>" \
  -o devices_export.csv

# Vérifier le contenu
head devices_export.csv
```

---

## Troubleshooting

### Problème: Les conteneurs ne démarrent pas

```bash
# Vérifier les logs Docker
docker compose logs

# Vérifier l'espace disque
df -h

# Vérifier les ports en conflit
sudo netstat -tulpn | grep -E "8080|5432|6379|8000"

# Nettoyer et redémarrer
docker compose down
docker compose up -d
```

### Problème: Capture ne voit aucun paquet

```bash
# 1. Vérifier que l'interface est UP
ip link show eth0

# 2. Vérifier les capabilities du conteneur
docker inspect netsentinel-capture | grep -A 10 CapAdd

# 3. Tester capture manuelle avec tcpdump
sudo tcpdump -i eth0 -c 10

# 4. Vérifier les logs capture
docker compose logs capture

# 5. Si besoin, activer le mode promiscuous manuellement
sudo ip link set eth0 promisc on
```

### Problème: Aucun device découvert

```bash
# 1. Vérifier que la capture fonctionne
docker compose logs capture | grep "Captured"

# 2. Vérifier Redis stream
docker exec -it netsentinel-redis redis-cli
XLEN netsentinel:frames  # Doit être > 0

# 3. Vérifier aggregator
docker compose logs aggregator | grep "Discovered"

# 4. Vérifier la base de données
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel
SELECT COUNT(*) FROM devices;
SELECT * FROM devices LIMIT 5;
```

### Problème: Interface web inaccessible

```bash
# 1. Vérifier que le conteneur web tourne
docker compose ps web

# 2. Vérifier les logs nginx
docker compose logs web

# 3. Vérifier que l'API répond
curl http://localhost:8080/api/health

# 4. Vérifier le firewall
sudo ufw status
sudo ufw allow 8080/tcp
```

### Problème: Mot de passe admin perdu

```bash
# Réinitialiser le mot de passe admin
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel

# Dans psql (password: changeme):
UPDATE users SET password_hash = '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.mHFHmYCg1HO.3q'
WHERE username = 'admin';
```

### Problème: Performance dégradée

```bash
# 1. Vérifier l'utilisation ressources
docker stats

# 2. Vérifier la taille de la base de données
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel
SELECT pg_size_pretty(pg_database_size('netsentinel'));

# 3. Nettoyer les anciennes données (> 30 jours)
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel
DELETE FROM traffic_metrics WHERE time < NOW() - INTERVAL '30 days';
VACUUM ANALYZE;

# 4. Vérifier Redis memory
docker exec -it netsentinel-redis redis-cli INFO memory
```

---

## Maintenance

### Sauvegarde

```bash
# Backup PostgreSQL
docker exec netsentinel-db pg_dump -U netsentinel netsentinel | gzip > backup_$(date +%Y%m%d).sql.gz

# Restauration
gunzip -c backup_20260131.sql.gz | docker exec -i netsentinel-db psql -U netsentinel netsentinel
```

### Mise à Jour

```bash
# 1. Arrêter les services
sudo ./deploy.sh stop

# 2. Mettre à jour le code
cd /opt/netsentinel
git pull

# 3. Reconstruire les images
docker compose build --no-cache

# 4. Redémarrer
sudo ./deploy.sh start

# 5. Vérifier
sudo ./deploy.sh status
```

### Monitoring

```bash
# Logs en continu
sudo ./deploy.sh logs

# Logs d'un service spécifique
sudo ./deploy.sh logs capture

# Statut global
sudo ./deploy.sh status

# Ressources utilisées
docker stats --no-stream
```

### Nettoyage

```bash
# Nettoyer les logs Docker (si trop volumineux)
docker system prune -a --volumes

# ⚠️ Cela supprime TOUT (images non utilisées, volumes orphelins, etc.)
# Utiliser avec précaution
```

---

## Commandes Utiles (Script deploy.sh)

```bash
# Installation initiale
sudo ./deploy.sh install

# Démarrer NetSentinel
sudo ./deploy.sh start

# Arrêter NetSentinel
sudo ./deploy.sh stop

# Redémarrer NetSentinel
sudo ./deploy.sh restart

# Voir le statut
sudo ./deploy.sh status

# Voir tous les logs
sudo ./deploy.sh logs

# Voir les logs d'un service
sudo ./deploy.sh logs capture
sudo ./deploy.sh logs aggregator
sudo ./deploy.sh logs api
sudo ./deploy.sh logs web

# Mettre à jour
sudo ./deploy.sh update

# Désinstaller complètement
sudo ./deploy.sh uninstall
```

---

## Sécurité

### Checklist Post-Installation

- [ ] Changer le mot de passe admin après premier login
- [ ] Générer des secrets forts dans `.env` (fait automatiquement par deploy.sh)
- [ ] Limiter l'accès au port 8080 via firewall
- [ ] Activer HTTPS avec certificat TLS (nginx + Let's Encrypt)
- [ ] Désactiver l'accès externe à PostgreSQL/Redis
- [ ] Mettre en place un backup régulier
- [ ] Configurer la rotation des logs

### Hardening Firewall (Exemple UFW)

```bash
# Autoriser uniquement l'accès web
sudo ufw allow 8080/tcp

# Bloquer tous les autres ports
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw enable
```

---

## Support

### Logs de Debug

```bash
# Activer le mode debug
nano .env
# Changer: LOG_LEVEL=debug

# Redémarrer
docker compose restart

# Voir les logs détaillés
docker compose logs -f capture
```

### Informations Système

```bash
# Générer un rapport système
cat > system_report.txt <<EOF
=== NetSentinel System Report ===
Date: $(date)
Hostname: $(hostname)
OS: $(lsb_release -d)
Kernel: $(uname -r)
Docker: $(docker --version)
Docker Compose: $(docker compose version)

=== Docker Status ===
$(docker compose ps)

=== Container Logs (last 50 lines) ===
$(docker compose logs --tail=50)

=== Disk Usage ===
$(df -h)

=== Memory ===
$(free -h)
EOF

# Envoyer ce fichier au support
```

---

## Checklist de Déploiement Complet

### Avant le Déploiement

- [ ] Serveur Debian/Ubuntu avec 2 Go RAM minimum
- [ ] Docker + Docker Compose installés
- [ ] Interface réseau identifiée (ex: eth0)
- [ ] Port 8080 disponible
- [ ] Accès root/sudo

### Installation

- [ ] Repository cloné
- [ ] `./deploy.sh install` exécuté
- [ ] `.env` configuré (interface réseau)
- [ ] Services démarrés (`./deploy.sh start`)

### Validation

- [ ] Tous les conteneurs UP (`docker compose ps`)
- [ ] PostgreSQL healthy
- [ ] Redis healthy
- [ ] Capture reçoit des paquets (logs)
- [ ] Aggregator persiste en DB
- [ ] API répond (`curl /api/health`)
- [ ] Interface web accessible (http://IP:8080)
- [ ] Login admin fonctionne
- [ ] Dashboard affiche des données

### Post-Déploiement

- [ ] Mot de passe admin changé
- [ ] Firewall configuré
- [ ] Backup configuré
- [ ] Monitoring en place
- [ ] Documentation client fournie

---

**Projet**: NetSentinel v0.1.0
**Organisation**: SecuAAS
**Licence**: Proprietary
**Support**: dev@secuaas.com
