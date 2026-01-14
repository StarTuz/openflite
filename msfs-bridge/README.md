# MSFS WASM Bridge

A WASM gauge module that runs inside Microsoft Flight Simulator and exposes SimVars via HTTP for cross-platform access.

## Installation

1. Copy the `openflite-msfs-bridge` folder to your MSFS Community packages folder
2. Start MSFS - the bridge will auto-start on port 8080
3. Connect OpenFlite to the MSFS simulator type

## API Reference

### `GET /status`

Check if the bridge is running.

**Response:**

```json
{
  "status": "ok",
  "simulator": "MSFS 2024",
  "connected": true
}
```

---

### `GET /simvars`

Get all subscribed SimVar values.

**Response:**

```json
{
  "INDICATED ALTITUDE": 35000.0,
  "AIRSPEED INDICATED": 250.5,
  "HEADING INDICATOR": 180.3,
  "GEAR HANDLE POSITION": 0.0,
  "AUTOPILOT MASTER": 1.0
}
```

---

### `POST /simvar`

Write a SimVar value.

**Request:**

```json
{
  "name": "HEADING INDICATOR",
  "value": 270.0
}
```

**Response:**

```json
{ "success": true }
```

---

### `POST /command`

Execute a K: event (calculator code).

**Request:**

```json
{
  "event": "TOGGLE_GEAR"
}
```

**Response:**

```json
{ "success": true }
```

---

## Default SimVars

The bridge subscribes to these SimVars by default:

| SimVar | Unit |
|--------|------|
| INDICATED ALTITUDE | feet |
| AIRSPEED INDICATED | knots |
| HEADING INDICATOR | degrees |
| GEAR HANDLE POSITION | percent |
| AUTOPILOT MASTER | bool |
| FLAPS HANDLE PERCENT | percent |
| NAV1 ACTIVE FREQUENCY | mhz |
| COM1 ACTIVE FREQUENCY | mhz |
| TRANSPONDER CODE | number |

## Adding Custom SimVars

POST to `/subscribe`:

```json
{
  "simvar": "PROP RPM:1",
  "unit": "rpm"
}
```
