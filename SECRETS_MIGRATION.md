# Secrets Migration - NetSentinel

**Date:** 2026-02-06
**Status:** ✅ Code migré - En attente initialisation OVH SM

---

## 🎯 Objectif

Migrer NetSentinel de la gestion secrets via `.env` vers OVH Secret Manager centralisé via le module `secuaas-secrets`.

---

## ✅ Changements Effectués

### Code Migré

**Package `secretsloader` créé:**
- ✅ Chargement automatique depuis OVH SM ou env vars
- ✅ Fallback gracieux si OVH SM indisponible
- ✅ Cache en mémoire des secrets
- ✅ Support multi-environnement (dev/staging/prod)

**Dépendances:**
- ✅ go.mod: `github.com/secuaas/secuaas-secrets@v0.6.1`
- ✅ Compilation sans erreurs

**Documentation:**
- ✅ `.env.example` créé (sans secrets sensibles)
- ✅ `SECRETS_MIGRATION.md` ce document

---

## 🔄 Comportement Actuel

### Mode Fallback (Par défaut)

**Sans credentials SecuOps:**
```
[SecretLoader] No OVH SM credentials found, using environment variables
[SecretLoader] Loaded N secrets from environment variables
```

✅ NetSentinel démarre normalement avec `.env` actuel
✅ **Aucune régression** - comportement identique à avant

### Mode OVH SM (Après configuration)

**Avec credentials SecuOps:**
```
[SecretLoader] Connected to OVH Secret Manager
[SecretLoader] Loaded N secrets from OVH SM (path: apps/netsentinel/dev)
```

✅ Secrets chargés depuis OVH SM
✅ Rotation facile via `secuops secrets set`
✅ Audit trail complet des accès

---

## 🚀 Prochaines Étapes - Initialisation OVH SM

### Étape 1: Configurer Credentials SecuOps

```bash
# Initialiser les credentials OVH
secuops secrets init \
  --region ca-east-bhs \
  --okms-id <OKMS_ID> \
  --pat-token <PAT_TOKEN>

# Vérifier la configuration
secuops secrets status --app=netsentinel --env=dev
```

### Étape 2: Générer Nouveaux Secrets

```bash
# Générer les secrets auto-générables
secuops secrets generate --app=netsentinel --env=dev --dry-run

# Appliquer
secuops secrets generate --app=netsentinel --env=dev
```

### Étape 3: Ajouter Secrets Static

```bash
# Copier valeurs depuis .env actuel
secuops secrets set apps/netsentinel/dev \
  DB_PASSWORD="<valeur_actuelle>" \
  API_KEY="<valeur_actuelle>"
# ... autres secrets static
```

### Étape 4: Tester avec OVH SM

```bash
export SECUOPS_CREDENTIALS_PATH=/home/ubuntu/.secuops/credentials.enc
cd /home/ubuntu/projects/NetSentinel
# Démarrer l'application
```

**Logs attendus:**
```
[SecretLoader] Connected to OVH Secret Manager
[SecretLoader] Loaded N secrets from OVH SM (path: apps/netsentinel/dev)
```

### Étape 5: Valider Fonctionnalités

- [ ] Application démarre sans erreurs
- [ ] Connexion base de données fonctionne
- [ ] Authentification fonctionne
- [ ] Fonctionnalités critiques OK

### Étape 6: Cleanup .env (APRÈS validation 100%)

⚠️ **ATTENTION: Ne faire QU'APRÈS validation complète**

```bash
# 1. Backup .env actuel
cp .env .env.backup.$(date +%Y%m%d)

# 2. Vérifier .gitignore
grep "^\.env$" .gitignore || echo ".env" >> .gitignore

# 3. Rotation secrets exposés (si nécessaire)
secuops secrets set apps/netsentinel/dev SECRET_NAME="new_value"
```

---

**Status:** ✅ Migration code complétée - Prêt pour initialisation OVH SM  
**Version:** NetSentinel (migration en cours)  
**Auteur:** Claude Sonnet 4.5  
**Date:** 2026-02-06
