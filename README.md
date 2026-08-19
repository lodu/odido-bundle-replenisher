# ODIDO Bundle Replenisher
Rust rewrite van [TMobile-NL-Unlimited-Bundle-Automated](https://github.com/lodu/TMobile-NL-Unlimited-Bundle-Automated) met bijgevoegde logica van Guus' [Odido.Authenticator](https://github.com/GuusBackup/Odido.Authenticator/) & [TMobile.Api](https://github.com/GuusBackup/TMobile.Api)


## Setup
Om het werkend te maken moet je environment MINIMAAL bevatten:

```bash
AUTHORIZATION_TOKEN=xxxxxxxxxx
MSISDN=+3161234567890
```

`AUTHORIZATION_TOKEN` kun je verkrijgen door het programma op te starten zonder `AUTHORIZATION_TOKEN` waarde en de instructies te volgen.

### Docker
Environment in [`.env`](.env) of direct in [docker-compose.yaml](./docker-compose.yaml).  

```bash
docker compose up -d
```

Dit gebruikt `ghcr.io/lodu/odido-bundle-replenisher:main` en leest de variabelen uit `.env`.

### Binaries
Environment in [`.env`](.env).  
Worden gebouwd door [GitHub Actions](https://github.com/lodu/odido-bundle-replenisher/actions).
