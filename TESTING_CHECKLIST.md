# NetSentinel - Checklist de Tests Complets

> Guide pratique pour tester toutes les fonctionnalités de NetSentinel

## Tests Rapides (5 minutes)

### ✅ Test 1: Démarrage des Services

```bash
cd /home/ubuntu/projects/NetSentinel
docker compose ps
```

**Résultat attendu**: Tous les conteneurs avec status `Up` et `healthy` pour postgres/redis.

### ✅ Test 2: Santé de l'API

```bash
curl http://localhost:8080/api/health
```

**Résultat attendu**:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "database": "connected",
  "redis": "connected"
}
```

### ✅ Test 3: Interface Web Accessible

1. Ouvrir navigateur: `http://localhost:8080`
2. Vérifier page de login visible
3. Login: `admin` / `changeme`
4. Redirection vers Dashboard

**Résultat attendu**: Dashboard visible avec widgets.

### ✅ Test 4: Capture Active

```bash
docker compose logs capture | tail -20
```

**Résultat attendu**: Messages `Captured X frames` réguliers.

### ✅ Test 5: Devices Découverts

```bash
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel -c "SELECT COUNT(*) FROM devices;"
```

**Résultat attendu**: `count > 0` (après 2-3 minutes de capture).

---

## Tests Complets (30 minutes)

### Module 1: Capture (Rust)

#### Test 1.1: Vérification Interface Réseau

```bash
# Lister interfaces disponibles
ip link show

# Vérifier l'interface configurée
grep CAPTURE_INTERFACES .env
```

**Attendu**: Interface existe et est UP.

#### Test 1.2: Mode Promiscuous

```bash
# Vérifier si promiscuous est actif
ip link show eth0 | grep PROMISC

# Si absent, activer manuellement
sudo ip link set eth0 promisc on
```

**Attendu**: Flag `PROMISC` présent.

#### Test 1.3: Capture de Paquets

```bash
# Logs capture
docker compose logs capture -n 100

# Générer du trafic
ping -c 50 8.8.8.8 &
curl https://www.google.com

# Vérifier les logs
docker compose logs capture | grep "Captured"
```

**Attendu**: Messages `Captured batch of X frames`.

#### Test 1.4: Redis Stream

```bash
docker exec -it netsentinel-redis redis-cli XLEN netsentinel:frames
```

**Attendu**: Nombre > 0 et augmente régulièrement.

### Module 2: Aggregator (Rust)

#### Test 2.1: Connexion Base de Données

```bash
docker compose logs aggregator | grep "Connected to PostgreSQL"
```

**Attendu**: Message de connexion réussie.

#### Test 2.2: Découverte de Devices

```bash
docker compose logs aggregator | grep "Discovered new device"
```

**Attendu**: Nouveaux devices découverts.

#### Test 2.3: Création de Flows

```bash
docker compose logs aggregator | grep "New flow"
```

**Attendu**: Flows créés entre devices.

#### Test 2.4: Persistence en DB

```bash
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel <<EOF
SELECT COUNT(*) FROM devices;
SELECT COUNT(*) FROM device_ips;
SELECT COUNT(*) FROM traffic_flows;
SELECT COUNT(*) FROM vlans;
EOF
```

**Attendu**: Toutes les tables contiennent des données.

### Module 3: API (Python FastAPI)

#### Test 3.1: Health Check

```bash
curl http://localhost:8080/api/health
```

**Attendu**: JSON avec `status: healthy`.

#### Test 3.2: Authentification

```bash
# Créer un token
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=admin&password=changeme" | jq -r '.access_token')

echo "Token: $TOKEN"
```

**Attendu**: Token JWT valide retourné.

#### Test 3.3: Liste Devices

```bash
curl -s http://localhost:8080/api/v1/devices \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Attendu**:
```json
{
  "items": [
    {
      "id": "uuid",
      "mac_address": "00:11:22:33:44:55",
      "oui_vendor": "Vendor Name",
      "device_type": "workstation",
      "ip_addresses": ["192.168.1.100"],
      "first_seen": "2026-01-31T12:00:00Z",
      "last_seen": "2026-01-31T12:30:00Z",
      "is_active": true
    }
  ],
  "total": 10,
  "page": 1,
  "page_size": 50
}
```

#### Test 3.4: Détail d'un Device

```bash
# Récupérer ID du premier device
DEVICE_ID=$(curl -s http://localhost:8080/api/v1/devices \
  -H "Authorization: Bearer $TOKEN" | jq -r '.items[0].id')

# Obtenir le détail
curl -s http://localhost:8080/api/v1/devices/$DEVICE_ID \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Attendu**: Détails complets du device (IPs, VLANs, stats).

#### Test 3.5: Liste Flows

```bash
curl -s http://localhost:8080/api/v1/flows \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Attendu**: Liste des flux réseau.

#### Test 3.6: Topologie

```bash
curl -s http://localhost:8080/api/v1/topology \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Attendu**:
```json
{
  "nodes": [
    {"id": "uuid", "label": "192.168.1.1", "group": "router"}
  ],
  "edges": [
    {"from": "uuid1", "to": "uuid2", "value": 1024}
  ]
}
```

#### Test 3.7: Statistiques Dashboard

```bash
curl -s http://localhost:8080/api/v1/stats/dashboard \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Attendu**:
```json
{
  "total_devices": 15,
  "active_devices": 12,
  "total_flows": 250,
  "total_packets": 1500000,
  "total_bytes": 2147483648,
  "uptime_seconds": 3600
}
```

#### Test 3.8: Export CMDB

```bash
curl -s "http://localhost:8080/api/v1/export/cmdb?format=json" \
  -H "Authorization: Bearer $TOKEN" \
  -o export_devices.json

cat export_devices.json | jq .
```

**Attendu**: Fichier JSON avec tous les devices.

### Module 4: Web UI (Vue.js)

#### Test 4.1: Page Login

1. **URL**: `http://localhost:8080/`
2. **Action**: Tester login invalide (`admin` / `wrong`)
3. **Attendu**: Message d'erreur "Invalid credentials"
4. **Action**: Login valide (`admin` / `changeme`)
5. **Attendu**: Redirection vers `/dashboard`

#### Test 4.2: Dashboard

1. **KPIs Widgets**:
   - Total Devices (nombre)
   - Active Devices (nombre)
   - Total Flows (nombre)
   - Total Bandwidth (bytes)

2. **Graphiques**:
   - Bandwidth Chart (Chart.js)
   - Top Talkers Widget (liste)

3. **Actualisation**:
   - Vérifier que les données se mettent à jour automatiquement (WebSocket)

#### Test 4.3: Page Devices

1. **Liste**:
   - Tableau avec colonnes: MAC, IP, Vendor, Type, Last Seen, Actions
   - Pagination fonctionnelle
   - Recherche par MAC/IP

2. **Filtres**:
   - Par type de device (router, switch, server, workstation, iot, unknown)
   - Par VLAN
   - Par état (active/inactive)

3. **Actions**:
   - Cliquer sur un device → Voir détails
   - Éditer metadata (nom, type, notes)

#### Test 4.4: Device Detail

1. **Informations Générales**:
   - MAC, OUI Vendor
   - Device Type (éditable)
   - Device Name (éditable)
   - First/Last Seen

2. **IP Addresses**:
   - Liste des IPs associées
   - VLANs associés

3. **Statistiques**:
   - Packets Sent/Received
   - Bytes Sent/Received
   - Graphique historique de bande passante

4. **Flows**:
   - Liste des flows impliquant ce device (source ou destination)

#### Test 4.5: Page Flows

1. **Liste**:
   - Colonnes: Source, Destination, Protocol, Ports, Packets, Bytes, Last Seen
   - Pagination

2. **Filtres**:
   - Par protocole (TCP, UDP, ICMP, Other)
   - Par port (22, 80, 443, etc.)
   - Par VLAN
   - Recherche par IP

3. **Tri**:
   - Par nombre de packets (desc)
   - Par bande passante (desc)
   - Par date (desc)

#### Test 4.6: Page Topology

1. **Graphe Réseau**:
   - Nodes visibles (devices)
   - Edges visibles (flows)
   - Groupes de couleurs (par type)

2. **Contrôles**:
   - Layout: Force-directed, Hierarchical, Circular
   - Filtre VLAN
   - Zoom in/out
   - Drag & drop nodes

3. **Interaction**:
   - Click sur node → Popup avec détails
   - Click sur edge → Infos du flow

4. **Performance**:
   - Graphe fluide avec 50+ nodes
   - Pas de lag lors du drag

#### Test 4.7: Page Settings

1. **Configuration Capture**:
   - Liste des interfaces
   - Enable/disable par interface

2. **Utilisateurs** (si implémenté):
   - Créer/modifier/supprimer users
   - Changer mot de passe

3. **Rétention**:
   - Politique de rétention (30 jours par défaut)
   - Compression automatique

4. **Export Cloud** (si activé):
   - Configuration OVH S3
   - Test de connexion

### Module 5: Base de Données (PostgreSQL + TimescaleDB)

#### Test 5.1: Schema Complet

```bash
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel <<EOF
\dt
EOF
```

**Attendu**: Tables présentes:
- `devices`
- `device_ips`
- `vlans`
- `traffic_flows`
- `traffic_metrics` (hypertable)
- `config_settings`
- `users`

#### Test 5.2: Indexes

```bash
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel <<EOF
\di
EOF
```

**Attendu**: Indexes sur:
- `devices(mac_address)`
- `device_ips(device_id, ip_address)`
- `traffic_flows(src_mac, dst_mac)`
- `traffic_metrics(time DESC)`

#### Test 5.3: Hypertable TimescaleDB

```bash
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel <<EOF
SELECT * FROM timescaledb_information.hypertables;
EOF
```

**Attendu**: `traffic_metrics` listé comme hypertable.

#### Test 5.4: Rétention Policy

```bash
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel <<EOF
SELECT * FROM timescaledb_information.jobs WHERE proc_name = 'policy_retention';
EOF
```

**Attendu**: Policy active avec interval 30 days.

### Module 6: Performance

#### Test 6.1: Capture Throughput

```bash
# Générer du trafic intense
ping -f 8.8.8.8 &  # Flood ping (nécessite sudo)
PID=$!

# Attendre 30 secondes
sleep 30

# Arrêter
kill $PID

# Vérifier les stats capture
docker compose logs capture | grep "fps"
```

**Attendu**: > 1000 fps (frames per second).

#### Test 6.2: Ressources CPU/RAM

```bash
docker stats --no-stream
```

**Attendu**:
- capture: < 10% CPU, < 100 MB RAM
- aggregator: < 5% CPU, < 200 MB RAM
- api: < 5% CPU, < 150 MB RAM
- postgres: < 15% CPU, < 500 MB RAM
- redis: < 2% CPU, < 256 MB RAM
- web: < 1% CPU, < 50 MB RAM

#### Test 6.3: Latence API

```bash
for i in {1..100}; do
  curl -s -o /dev/null -w "%{time_total}\n" http://localhost:8080/api/health
done | awk '{ sum += $1; n++ } END { print "Average: " sum/n " seconds" }'
```

**Attendu**: Moyenne < 0.05 secondes (50ms).

#### Test 6.4: Requêtes Concurrentes

```bash
# Installer Apache Bench si nécessaire
sudo apt-get install apache2-utils

# Test de charge (100 requêtes, 10 concurrentes)
ab -n 100 -c 10 http://localhost:8080/api/health
```

**Attendu**:
- Time per request: < 100ms
- Requests per second: > 100
- Failed requests: 0

---

## Tests de Régression

### Scénario 1: Redémarrage Complet

```bash
# 1. Arrêter tous les services
docker compose down

# 2. Attendre 10 secondes
sleep 10

# 3. Redémarrer
docker compose up -d

# 4. Vérifier santé
sleep 30
docker compose ps
curl http://localhost:8080/api/health
```

**Attendu**: Tous les services redémarrent sans erreur.

### Scénario 2: Perte Connexion Redis

```bash
# 1. Arrêter Redis
docker compose stop redis

# 2. Vérifier logs capture
docker compose logs capture | tail -10

# 3. Redémarrer Redis
docker compose start redis

# 4. Vérifier reconnexion
docker compose logs capture | tail -10
```

**Attendu**: Capture se reconnecte automatiquement à Redis.

### Scénario 3: Perte Connexion PostgreSQL

```bash
# 1. Arrêter PostgreSQL
docker compose stop postgres

# 2. Vérifier logs aggregator
docker compose logs aggregator | tail -10

# 3. Redémarrer PostgreSQL
docker compose start postgres

# 4. Vérifier reconnexion
docker compose logs aggregator | tail -10
```

**Attendu**: Aggregator se reconnecte automatiquement.

### Scénario 4: Saturation Disque

```bash
# Simuler saturation (ne pas faire en production)
# dd if=/dev/zero of=/tmp/fillup bs=1M count=10000

# Vérifier comportement
docker compose logs aggregator | grep -i "error"
docker compose logs postgres | grep -i "disk"
```

**Attendu**: Messages d'erreur clairs, pas de crash.

### Scénario 5: Volume de Données Important

```bash
# Laisser tourner 24h avec génération de trafic

# Vérifier taille DB
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel <<EOF
SELECT pg_size_pretty(pg_database_size('netsentinel'));
EOF

# Vérifier performance
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/api/v1/devices
```

**Attendu**: Performance stable, taille DB croissance linéaire.

---

## Checklist Finale de Validation

### Fonctionnalités Core

- [ ] Capture passive fonctionne (pas d'émission de paquets)
- [ ] Devices découverts automatiquement
- [ ] IPs associées aux devices (multi-IP support)
- [ ] VLANs détectés (802.1Q)
- [ ] Flows créés entre devices
- [ ] Statistiques précises (packets, bytes)
- [ ] Timestamps corrects (UTC)

### API

- [ ] Health check répond
- [ ] Authentification JWT fonctionne
- [ ] CRUD devices opérationnel
- [ ] Liste flows avec filtres
- [ ] Topologie retourne graph valide
- [ ] Stats dashboard correctes
- [ ] Export CMDB (JSON/CSV)
- [ ] WebSocket temps réel (optionnel MVP)

### Interface Web

- [ ] Login/logout fonctionnels
- [ ] Dashboard avec KPIs et graphiques
- [ ] Page devices avec recherche/filtres
- [ ] Device detail avec édition metadata
- [ ] Page flows avec filtres protocole/port
- [ ] Topologie interactive (vis-network)
- [ ] Page settings (configuration basique)
- [ ] Responsive (mobile/tablet)

### Base de Données

- [ ] Schema complet appliqué
- [ ] Hypertable TimescaleDB active
- [ ] Rétention policy configurée (30j)
- [ ] Indexes performants
- [ ] Compression activée (si > 7j)
- [ ] User admin créé (password: changeme)

### Performance

- [ ] Capture: > 1000 fps sur lien 100 Mbps
- [ ] API latence: < 100ms (p95)
- [ ] Web UI fluide (< 2s chargement pages)
- [ ] CPU total: < 50% sur serveur 2 cores
- [ ] RAM total: < 1.5 GB

### Sécurité

- [ ] Secrets générés aléatoirement (.env)
- [ ] Mot de passe hashé bcrypt (users)
- [ ] JWT signé avec secret fort
- [ ] Pas de données sensibles dans logs
- [ ] Conteneurs isolés (network bridge)
- [ ] Capabilities minimales (capture: NET_RAW, NET_ADMIN)

### Déploiement

- [ ] Docker Compose up -d réussit
- [ ] Tous les conteneurs healthy
- [ ] Script deploy.sh fonctionnel
- [ ] Logs accessibles (deploy.sh logs)
- [ ] Redémarrage automatique (restart: unless-stopped)
- [ ] Volumes persistants (postgres_data, redis_data)

### Documentation

- [ ] README.md complet
- [ ] DEPLOYMENT_GUIDE.md disponible
- [ ] TESTING_CHECKLIST.md validé
- [ ] CLAUDE_CODE_SPEC.md à jour
- [ ] Commentaires code suffisants

---

## Rapport de Test (Template)

```
NetSentinel - Test Report
========================

Date: 2026-01-31
Testeur: [Nom]
Version: 0.1.0
Environnement: [Debian 12 / Ubuntu 22.04 / Autre]

Tests Rapides (5 min)
---------------------
[✅] Test 1: Démarrage services
[✅] Test 2: Santé API
[✅] Test 3: Interface web
[✅] Test 4: Capture active
[✅] Test 5: Devices découverts

Tests Complets (30 min)
-----------------------
Module Capture:
[✅] Interface réseau OK
[✅] Mode promiscuous actif
[✅] Paquets capturés
[✅] Redis stream alimenté

Module Aggregator:
[✅] Connexion DB OK
[✅] Devices découverts
[✅] Flows créés
[✅] Persistence DB

Module API:
[✅] Health check
[✅] Authentification
[✅] Liste devices
[✅] Détail device
[✅] Liste flows
[✅] Topologie
[✅] Stats dashboard
[✅] Export CMDB

Module Web UI:
[✅] Login
[✅] Dashboard
[✅] Page devices
[✅] Device detail
[✅] Page flows
[✅] Page topology
[✅] Page settings

Base de Données:
[✅] Schema complet
[✅] Indexes
[✅] Hypertable
[✅] Rétention policy

Performance:
[✅] Capture throughput: XXX fps
[✅] Ressources: CPU XX% / RAM XX MB
[✅] Latence API: XX ms
[✅] Charge concurrente OK

Tests Régression:
[✅] Redémarrage complet
[✅] Perte Redis
[✅] Perte PostgreSQL
[  ] Saturation disque (non testé)
[  ] Volume données 24h (non testé)

Checklist Finale:
[✅] Fonctionnalités core: XX/YY
[✅] API: XX/YY
[✅] Web UI: XX/YY
[✅] Base de données: XX/YY
[✅] Performance: XX/YY
[✅] Sécurité: XX/YY
[✅] Déploiement: XX/YY
[✅] Documentation: XX/YY

Problèmes Identifiés:
--------------------
1. [Aucun / Liste des bugs]

Recommandations:
---------------
1. [Améliorations suggérées]

Conclusion:
----------
[PASS / FAIL / PASS WITH ISSUES]

NetSentinel est [prêt / non prêt] pour déploiement en production.
```

---

**Projet**: NetSentinel v0.1.0
**Organisation**: SecuAAS
**Date de création**: 2026-01-31
