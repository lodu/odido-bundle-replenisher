# ODIDO Bundle Replenisher
Rust rewrite van [TMobile-NL-Unlimited-Bundle-Automated](https://github.com/lodu/TMobile-NL-Unlimited-Bundle-Automated) met bijgevoegde logica van Guus' [Odido.Authenticator](https://github.com/GuusBackup/Odido.Authenticator/) & [TMobile.Api](https://github.com/GuusBackup/TMobile.Api)


## Setup
`AUTHORIZATION_TOKEN` en `MSISDN` zijn de enige verplichte instellingen. Kies zelf hoe je ze
doorgeeft:

- `.env`-bestand (zie [`.env.example`](.env.example))
- environment variables
- CLI-flags: `--authorization-token` / `--msisdn`

```bash
AUTHORIZATION_TOKEN=xxxxxxxxxx
MSISDN=+3161234567890
```

`AUTHORIZATION_TOKEN` kun je verkrijgen door het programma op te starten zonder die waarde en de instructies te volgen. Zie hieronder.

Waarschijnlijk hoeft je environment ook niet meer te bevatten. Zie [ENVIRONMENT.md](./ENVIRONMENT.md) voor alle env vars en CLI-flags.

## Usage
Drie opties:
1. docker compose
2. docker
3. binary


### 1. Docker Compose

Zet de environmentvariabelen in `.env`, zie [`.env.example`](.env.example).

```bash
docker compose up -d
```

Bij ontbrekende `AUTHORIZATION_TOKEN` of `REFRESH_TOKEN`:

```bash
docker compose run --rm odido-bundle-replenisher
```

Volg instructies en zet de waarde in je `.env`.

### 2. Docker

Zonder Compose kun je de container op de achtergrond draaien met:

```bash
docker run -d --name odido-bundle-replenisher --restart unless-stopped \
  --env-file .env ghcr.io/lodu/odido-bundle-replenisher:latest
```

Bij ontbrekende `AUTHORIZATION_TOKEN` of `REFRESH_TOKEN`:
```bash
docker run --rm -it --env-file .env ghcr.io/lodu/odido-bundle-replenisher:latest
```

### 3. Binaries

Zet de environmentvariabelen in een `.env`-bestand naast de binary. Binaries worden gebouwd door [GitHub Actions](https://github.com/lodu/odido-bundle-replenisher/actions). Klik hierbij op de laatste run van `Build artifacts` en scroll naar beneden voor artifacts.

## Eigen scheduler (cron, Ofelia, ...)
Met `RUN_ONCE=true` (of `--once`) loopt het programma niet, dan checkt de app één keer en stopt.  
Combineer met `--msisdn` en `--authorization-token` voor een losse aanroep zonder `.env` of environment variables.
