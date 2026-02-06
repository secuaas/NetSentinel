# TODO - SecuOps Conformité pour netsentinel

Ce projet nécessite les modifications suivantes pour être conforme SecuOps:

## ❌ Dockerfile manquant

Créer un Dockerfile avec build multi-stage:


## ❌ .secuops.yaml manquant

Créer un fichier .secuops.yaml à la racine du projet:

```yaml
name: netsentinel
type: unknown
description: Description du projet
port: 8080

github:
  repo: https://github.com/secuaas/netsentinel.git
  branch: main

kubernetes:
  namespace: netsentinel
  replicas: 1
```

## ❌ Manifests Kubernetes manquants

Créer deploy/k8s/netsentinel.yaml avec:
- Namespace
- Deployment avec resources limits/requests
- Service
- Ingress avec TLS
- Health probes (liveness/readiness)

Utiliser 'secuops new netsentinel' pour générer un template

## ❌ README.md manquant

Créer un README.md documentant:
- Description du projet
- Prérequis
- Installation
- Utilisation
- Déploiement

## ❌ .gitignore manquant

Créer un .gitignore pour exclure:
- Fichiers de build
- node_modules/, vendor/
- .env, secrets
- IDE files (.vscode/, .idea/)

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
secuops build --app=netsentinel

# Déployer en dev
secuops deploy --app=netsentinel --env=k8s-dev

# Déployer en prod
secuops deploy --app=netsentinel --env=k8s-prod
```
