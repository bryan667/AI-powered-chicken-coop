# AI-Powered Chicken Coop

AI Chicken Coop🐔
"bok bok bokok!"

Rust project for controlling an AI-powered chicken coop.
- sensor readings (temperature, humidity, motion, egg detection)
- automated feeders and coop doors
- caching & logging
- async scheduling for routine tasks
- CLI commands for status, feeding, and AI detection

## Environment Variables

Create a local `.env` file from the template:

```bash
cp .env.example .env
```

On Windows PowerShell:

```powershell
Copy-Item .env.example .env
```

Then edit `.env` and set real values for:

- `TEMP_SENSOR_KEY`
- `HUMIDITY_SENSOR_KEY`
- `MOTION_SENSOR_KEY`
- `EGG_SENSOR_KEY`
- `FEEDER_KEY`
- `DOOR_KEY`
- `ACTUATOR_API_KEY`
- `AI_KEY`
- `CACHE_KEY`

Optional:

- `EGG_MODEL_PATH` (default: `/models/egg_detector.pt`)
- `PREDATOR_MODEL_PATH` (default: `/models/predator_detector.pt`)
- `SENSOR_API_BASE_URL` (default: `http://127.0.0.1:8080`)
- `ACTUATOR_API_BASE_URL` (default: `http://127.0.0.1:8081`)
- `ACTUATOR_BIND_ADDR` (default: `0.0.0.0:8081`)
- `ACTUATOR_ALLOWED_ORIGIN` (default: `*`)
- `FEEDER_ACTIVATE_CMD` (shell command executed on feeder activation)
- `DOOR_OPEN_CMD` (shell command executed on door open)
- `DOOR_CLOSE_CMD` (shell command executed on door close)

`.env` is git-ignored, so it should not be committed.

Sensor endpoints used by the app:
- `GET /sensors/temperature`
- `GET /sensors/humidity`
- `GET /sensors/motion`
- `GET /sensors/eggs`

Actuator endpoints used by the app:
- `POST /actuators/feeder/activate`
- `POST /actuators/door/open`
- `POST /actuators/door/close`

Actuator endpoints expect:
- Header: `x-api-key: <ACTUATOR_API_KEY>`
- JSON body for feeder: `{"device_key":"<FEEDER_KEY>","duration_ms":2500}`
- JSON body for door: `{"device_key":"<DOOR_KEY>"}`

## Usage

```bash
cargo run -- status
cargo run -- feed now
cargo run -- run ai-vision
cargo run -- serve actuators
```