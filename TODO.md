# TODO - SecuOps Conformité pour NetSentinel

Ce projet nécessite les modifications suivantes pour être conforme SecuOps:

## ❌ .secuops.yaml manquant

Créer un fichier .secuops.yaml à la racine du projet:

```yaml
name: NetSentinel
type: unknown
description: Description du projet
port: 8080

github:
  repo: https://github.com/secuaas/NetSentinel.git
  branch: main

kubernetes:
  namespace: NetSentinel
  replicas: 1
```

## ❌ Manifests Kubernetes manquants

Créer deploy/k8s/NetSentinel.yaml avec:
- Namespace
- Deployment avec resources limits/requests
- Service
- Ingress avec TLS
- Health probes (liveness/readiness)

Utiliser 'secuops new NetSentinel' pour générer un template

## ❌ .dockerignore manquant

Créer un .dockerignore pour exclure:
- .git/
- node_modules/
- .env
- README.md
- .gitignore

## Commandes pour déployer

Une fois conforme:

```bash
# Build l'image
secuops build --app=NetSentinel

# Déployer en dev
secuops deploy --app=NetSentinel --env=k8s-dev

# Déployer en prod
secuops deploy --app=NetSentinel --env=k8s-prod
```
