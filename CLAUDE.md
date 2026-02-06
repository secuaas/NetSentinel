# CLAUDE.md — NetSentinel

> Configuration et spécificités pour ce projet SecuAAS
> Lire aussi le fichier centralisé `/home/ubuntu/projects/CLAUDE.md`

---

## 🤖 Configuration Claude

**Modèle recommandé pour ce projet:**
```bash
claude --model opus
```

**Pourquoi Opus?**
- Capacité de raisonnement supérieure pour tâches complexes
- Meilleure compréhension du contexte projet
- Performances optimales pour l'architecture et le code
- Recommandé pour tous les projets SecuAAS

---

## 📋 Informations Projet

**Nom:** NetSentinel
**Type:** À définir
**État:** À définir
**Version actuelle:** À définir

**Description:**
Projet NetSentinel

**Repository GitHub:**
https://github.com/secuaas/NetSentinel

---

## 🛠️ Stack Technique

**Langages/Frameworks:**
- Docker

**Dépendances principales:**
- À documenter

**Structure du projet:**
```
NetSentinel/
├── [À documenter selon la structure réelle]
```

---

## 🚀 Développement Local

### Installation

```bash
cd /home/ubuntu/projects/NetSentinel
[Commandes d'installation spécifiques]
```

### Compilation/Build

```bash
[Commandes de build spécifiques au projet]
```

### Tests

```bash
[Commandes de test]
```

---

## 📦 Déploiement

**Environnements:**
- k8s-dev: Environnement de développement
- k8s-prod: Environnement de production (si applicable)

**Commandes secuops:**

```bash
# Build
secuops build --app=NetSentinel --env=k8s-dev

# Deploy
secuops deploy --app=NetSentinel --env=k8s-dev

# Status
secuops status --app=NetSentinel --env=k8s-dev

# Logs
secuops logs --app=NetSentinel --env=k8s-dev
```

---

## ⚠️ Pièges Spécifiques à ce Projet

[Liste des problèmes connus, bugs récurrents, configurations délicates, etc.]

---

## 📝 Notes Importantes

[Toute information critique spécifique à ce projet]

---

## 📚 Documentation

- README: [lien si existe]
- Documentation technique: [lien si existe]
- API Documentation: [lien si existe]

---

**Dernière mise à jour:** 2026-02-06
