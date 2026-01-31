# NetSentinel - Quick Start Guide

> Démarrage rapide en 5 minutes

## Installation Automatique (Recommandé)

```bash
# 1. Cloner le projet
git clone https://github.com/secuaas/NetSentinel.git
cd NetSentinel

# 2. Installer (génère automatiquement les secrets)
sudo ./deploy.sh install

# 3. Configurer votre interface réseau
sudo nano /opt/netsentinel/.env
# Modifier la ligne: CAPTURE_INTERFACES=eth0  (remplacer eth0 par votre interface)

# 4. Démarrer NetSentinel
sudo ./deploy.sh start

# 5. Accéder à l'interface web
# http://<votre-ip>:8080
# Login: admin / changeme
```

## Vérifier que tout fonctionne

```bash
# Voir le statut des services
sudo ./deploy.sh status

# Voir les logs en temps réel
sudo ./deploy.sh logs

# Tester l'API
curl http://localhost:8080/api/health
```

## Commandes Essentielles

```bash
# Démarrer
sudo ./deploy.sh start

# Arrêter
sudo ./deploy.sh stop

# Redémarrer
sudo ./deploy.sh restart

# Voir les logs d'un service spécifique
sudo ./deploy.sh logs capture
sudo ./deploy.sh logs aggregator
sudo ./deploy.sh logs api

# Mettre à jour
sudo ./deploy.sh update

# Désinstaller
sudo ./deploy.sh uninstall
```

## Identifier votre Interface Réseau

```bash
# Lister toutes les interfaces
ip link show

# Exemple de sortie:
# 1: lo: <LOOPBACK>
# 2: eth0: <BROADCAST,MULTICAST,UP>     ← Interface filaire
# 3: wlan0: <BROADCAST,MULTICAST,UP>    ← Interface WiFi
# 4: docker0: <BROADCAST,MULTICAST>     ← Interface Docker

# Utiliser eth0 ou wlan0 selon votre réseau
```

## Premiers Pas dans l'Interface Web

1. **Login**
   - URL: `http://<ip>:8080`
   - Username: `admin`
   - Password: `changeme`
   - ⚠️ Changer le mot de passe après premier login!

2. **Dashboard**
   - Vue d'ensemble du réseau
   - Statistiques en temps réel
   - Top talkers

3. **Devices**
   - Liste des appareils découverts
   - Cliquer sur un device pour voir les détails
   - Éditer le nom et le type

4. **Topology**
   - Graphe visuel du réseau
   - Cliquer sur les nodes pour détails
   - Filtrer par VLAN

5. **Flows**
   - Voir les communications entre devices
   - Filtrer par protocole, port, VLAN

## Troubleshooting Rapide

### Problème: Aucun device découvert après 5 minutes

```bash
# 1. Vérifier que la capture fonctionne
sudo ./deploy.sh logs capture | grep "Captured"
# Doit afficher: "Captured XXX frames"

# 2. Vérifier l'interface réseau
ip link show eth0  # Remplacer eth0 par votre interface
# Doit afficher: "state UP"

# 3. Générer du trafic pour tester
ping -c 10 8.8.8.8
curl https://www.google.com

# 4. Vérifier la base de données
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel -c "SELECT COUNT(*) FROM devices;"
```

### Problème: Interface web inaccessible

```bash
# 1. Vérifier que les conteneurs tournent
docker compose ps
# Tous doivent être "Up"

# 2. Vérifier le port 8080
sudo netstat -tulpn | grep 8080

# 3. Tester l'API directement
curl http://localhost:8080/api/health

# 4. Vérifier les logs
sudo ./deploy.sh logs web
```

### Problème: Mot de passe admin oublié

```bash
# Réinitialiser (password: changeme)
docker exec -it netsentinel-db psql -U netsentinel -d netsentinel -c \
  "UPDATE users SET password_hash = '\$2b\$12\$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.mHFHmYCg1HO.3q' WHERE username = 'admin';"
```

## Documentation Complète

- **Guide de Déploiement**: `DEPLOYMENT_GUIDE.md`
- **Checklist de Tests**: `TESTING_CHECKLIST.md`
- **Spécification Technique**: `CLAUDE_CODE_SPEC.md`
- **README Principal**: `README.md`

## Support

- **Logs**: `sudo ./deploy.sh logs`
- **Status**: `sudo ./deploy.sh status`
- **GitHub**: https://github.com/secuaas/NetSentinel

---

**Bon monitoring!** 🔍
