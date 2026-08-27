# Odido Bundle Replenisher
> Rust rewrite van [TMobile-NL-Unlimited-Bundle-Automated](https://github.com/lodu/TMobile-NL-Unlimited-Bundle-Automated) met bijgevoegde logica van Guus' [Odido.Authenticator](https://github.com/GuusBackup/Odido.Authenticator/) & [TMobile.Api](https://github.com/GuusBackup/TMobile.Api)

Vraag automatisch nieuwe Odido databundels aan als je bijna bij de daglimiet van je (fair use) onbeperkte data-abonnement zit.

## Authorizatie Token verkrijgen
1. Download de laatste [release](https://github.com/lodu/odido-bundle-replenisher/releases/latest) onder `Assets`.
2. Pak het archief (zip) uit.
3. Voer de app in je terminal uit `./odido-bundle-replenisher` of `.\odido-bundle-replenisher.exe`.
4. Open de gegeven link, log in en kopieer de URL na het inloggen.
5. Plak de URL terug in de terminal en klik op enter.
6. 🥳


## Quick Start
`AUTHORIZATION_TOKEN` en `MSISDN` zijn de enige verplichte instellingen. Kies zelf hoe je ze
doorgeeft:

- `.env`-bestand (zie [`.env.example`](.env.example))
- environment variables
- CLI-flags: `--authorization-token` / `--msisdn`

```bash
AUTHORIZATION_TOKEN=xxxxxxxxxx
MSISDN=+31612345678
```

`AUTHORIZATION_TOKEN` kun je verkrijgen door het programma op te starten zonder die waarde en de instructies te volgen. Zie [hierboven](#authorizatie-token-verkrijgen).

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

```bash
./odido-bundle-replenisher --msisdn +31612345678 --authorization-token xxxxxxxxxx [--once]
```

Download de laatste [release](https://github.com/lodu/odido-bundle-replenisher/releases/latest) onder `Assets`.

## Eigen scheduler (cron, Ofelia, ...)
Met `RUN_ONCE=true` (of `--once`) loopt het programma niet, dan checkt de app één keer en stopt.  
Combineer met `--msisdn` en `--authorization-token` voor een losse aanroep zonder `.env` of environment variables.
