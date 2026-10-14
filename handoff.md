# Handoff — tempspartiel.laurans.org

## Objectif
Application web Rust (Axum) de simulation TPT (temps partiel thérapeutique) selon la loi française (IJSS) et la convention collective métallurgie. Code sur GitHub, déployée sur OCIUB derrière Caddy.

## Décisions prises
- Stack : Rust 2024 + Axum 0.8 + tokio + serde + tower-http.
- Build/runtime : image `rust:1.98-alpine` → binaire musl, image finale `alpine:3.22`.
- Build Docker sur OCIUB avec `network: host` pour éviter le DNS link-local des ponts (`proxy.golang.org` / registre).
- Déploiement similaire à `topps` : `docker compose up -d --build`, aucun port publié, réseau externe `guacamole_internal`.
- Domaine et TLS gérés par Caddy (`/app/guacamole/Caddyfile`), ajout d'un bloc `tempspartiel.laurans.org { reverse_proxy tempspartiel:8080 }`.
- Calcul TPT basé sur la métallurgie (CCN 3248) :
  - maintien 100 % selon l'ancienneté (90/120/150/180 j) ;
  - cadres : puis 50 % pour la même durée ;
  - non-cadres : plus de complément après la période à 100 %.
- IJSS estimées : plafond 41,95 €/j (2026), 360 IJ max sur 3 ans, net après CGS/RDS (~6,7 %).
- Hypothèses numériques marquées comme approximations et site déclaré généré par IA pouvant contenir des erreurs.
- Remplacement général d'« Airbus » par « avionneur » puis passage aux règles « métallurgie » sur demande utilisateur.

## Fichiers à rouvrir
- Projet local : `/home/dieu/tempspartiel`
- Source principale : `src/main.rs`
- Frontend : `static/index.html`
- Manifeste : `Cargo.toml`, `Cargo.lock`
- Conteneur : `Dockerfile`, `.dockerignore`, `docker-compose.yml`, `.env.example`, `README.md`
- Proxy distant : `ociub:/app/guacamole/Caddyfile`
- Dépôt distant cloné : `ociub:/app/tempspartiel`
- GitHub : `blaurans/tempspartiel` (branche `main`)

## Ce qui a été fait et vérifié
- Repo Git initialisé, poussé sur GitHub.
- App compilée localement (`cargo build`, `cargo test` : 2 tests OK dont `/healthz` et `/api/calc`).
- Image Docker construite sur OCIUB (Alpine/musl).
- Conteneur démarré et joint au réseau `guacamole_internal`.
- Caddy recharge sa config sans erreur bloquante ; certificat Let's Encrypt obtenu pour `tempspartiel.laurans.org`.
- Vérifications :
  - `/healthz` OK côté conteneur et via Caddy ;
  - page HTTPS s'affiche ;
  - `/api/calc` renvoie les mois avec phase, IJSS, complément, total et cumuls ;
  - extraction de données réelle vérifiée pour plusieurs durées/statuts.
- UI refaite : grille responsive, tableau avec en-tête sticky, info-bulles « ? », bannière d'avertissement IA.

## Problèmes encore ouverts
- Les noms de conteneurs Docker peuvent devenir préfixés par un hash après recréation (`..._tempspartiel-tempspartiel-1`), mais l'alias réseau `tempspartiel` continue de résoudre. Surveillance recommandée.
- `docker compose up -d --build` peut se bloquer sur un conteneur en suppression ; attendre quelques secondes puis relancer `docker compose up -d`.
- Avertissement Caddy : `Caddyfile input is not formatted` (non bloquant).
- Aucun pipeline CI/CD ni script de reprise de type `charge-up` dans ce projet.
- Formule de rétablissement CPAM pour l'absence dans la période de référence est approximative.
- Pas de tests unitaires sur la logique TPT (phases, plafonds, épuisement 360 IJ).

## Ce que je ne peux pas retrouver / à confirmer
- Données de paie réelles pour valider les montants IJSS et le complément conventionnel.
- Amélioration possible de la formule « jours d'arrêt dans les 3 mois » avec la règle CPAM exacte.
- Confirmations terrain (statut exact, ancienneté, statut cadre) à fournir par l'utilisateur.

## Prochaine action recommandée
1. Ajouter des tests unitaires Rust sur `calc` (maintien métallurgie 90/120/150/180 j, phase cadre à 50 %, plafond IJSS, épuisement 360 IJ).
2. Valider le calcul avec un bulletin de paie réel et un retour CPAM.
3. Mettre éventuellement en place un script `tempspartiel-up` (modèle `charge/deploy/charge-up`) pour robustifier le redéploiement.
4. Éventuellement nettoyer/formater le `Caddyfile` côté serveur.

## Secrets
Aucun secret n'est consigné dans ce fichier.
