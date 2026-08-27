
## Mogelijke Environmental Variables

### Minimaal nodig om te runnen

| Variabele | CLI-flag | Beschrijving | Default |
|---|---|---|---|
| `MSISDN` | `--msisdn`/`-m` | Je telefoonnummer, bijv. `+31612345678`. | verplicht |
| `AUTHORIZATION_TOKEN` | `--authorization-token`/`-a` | Sessietoken om bij Odido in te loggen. Ontbreekt hij, dan valt de app terug op `REFRESH_TOKEN`; ontbreken beide, dan moet je inloggen. Zie [README](README.md#setup). | verplicht* |

### OAuth om tot een AUTHORIZATION_TOKEN te komen

| Variabele | CLI-flag | Beschrijving | Default |
|---|---|---|---|
| `REFRESH_TOKEN` | `--refresh-token`/`-r` | OAuth-code om automatisch een nieuw `AUTHORIZATION_TOKEN` te genereren. Wordt alleen gebruikt als `AUTHORIZATION_TOKEN` ontbreekt. **Let op dat deze tijdgevoelig en dus waarschijnlijk maar bij één startup werkt.** | - |

### Configureerbaar

| Variabele | CLI-flag | Beschrijving | Default |
|---|---|---|---|
| `ODIDO_BUYING_CODE` | - | Welke bundel wordt aangevraagd. | `A0DAY01` |
| `MB_THRESHOLD` | - | Onder hoeveel MB een nieuwe bundel wordt aangevraagd. | `2000` |
| `RUN_ONCE` | `--once`/`-o` | Draai het programma één keer i.p.v. in een oneindige loop. | `false` |

### Check-interval

Kies statisch óf dynamisch. Dynamisch checkt vaker zodra je nog weinig MB's over hebt.

| Variabele | Beschrijving | Default |
|---|---|---|
| `DYNAMIC_INTERVAL_MB_THRESHOLD` | Dynamisch: onder hoeveel MB er sneller gecheckt wordt. | `4000` |
| `DYNAMIC_INTERVAL_LOW` | Dynamisch: interval in minuten zodra je onder die grens zit. | `1` |
| `DYNAMIC_INTERVAL_HIGH` | Dynamisch: interval in minuten zolang je erboven zit. | `10` |
| `CHECK_INTERVAL` | Statisch: vast interval in minuten. Zet je deze, dan negeert de app de dynamische vars hieronder. | - |


### Praktisch nooit aanpassen

| Variabele | Beschrijving | Default |
|---|---|---|
| `ODIDO_URL` | Basis-URL van odido.nl, gebruikt tijdens de login-flow. | `https://odido.nl` |
| `ODIDO_API_URL` | Basis-URL van de Odido API. | `https://capi.odido.nl` |
| `ODIDO_FERNET_KEY` | Sleutel die Odido zelf gebruikt voor de login-flow. | - |
| `ODIDO_OAUTH_KEY` | Sleutel die Odido zelf gebruikt voor de OAuth-flow. | - |
| `ODIDO_USER_AGENT` | User agent waarmee de app zich voordoet als de officiële Odido-app. | `ODIDO 8.0.0 (Android 12; 12)` |
| `HTTP_MAX_RETRIES` | Hoeveel keer een mislukte request wordt herhaald. | `10` |
| `HTTP_RETRY_DELAY_STEP` | Wachttijd tussen retries, loopt op per poging (stap × 100ms × pogingnummer). | `10` |

\* of `REFRESH_TOKEN`, zie hierboven.

