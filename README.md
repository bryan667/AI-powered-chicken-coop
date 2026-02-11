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
- `ACTUATOR_API_BASE_URL` (default: `http://127.0.0.1:8080`)

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

## Usage

```bash
cargo run -- status
cargo run -- feed now
cargo run -- run ai-vision
```
