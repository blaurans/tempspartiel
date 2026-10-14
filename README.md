# tempspartiel

Application web en Rust (Axum) : calculette de salaire pour un temps partiel.

- Binaire unique, aucune dépendance externe.
- Endpoints : `/`, `/healthz`, `POST /api/calc`.
- Image Docker multi-arch (build sur l'hôte, Alpine/musl).

## Développement local

```sh
cargo run
# http://127.0.0.1:8080
```

## Déploiement sur ociub

```sh
ssh ociub 'cd /app/tempspartiel && git pull && docker compose up -d --build'
```

Reverse proxy : Caddy (`guacamole_internal`) expose `https://tempspartiel.laurans.org`.