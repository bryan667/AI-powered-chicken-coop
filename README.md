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
- `ACTUATOR_BACKEND` (`command` or `rpi-gpio`, default: `command`)
- `FEEDER_ACTIVATE_CMD` (shell command executed on feeder activation)
- `DOOR_OPEN_CMD` (shell command executed on door open)
- `DOOR_CLOSE_CMD` (shell command executed on door close)
- `FEEDER_GPIO_PIN` (default: `17`, for `rpi-gpio`)
- `DOOR_OPEN_GPIO_PIN` (default: `27`, for `rpi-gpio`)
- `DOOR_CLOSE_GPIO_PIN` (default: `22`, for `rpi-gpio`)
- `ACTUATOR_ACTIVE_HIGH` (default: `true`, for `rpi-gpio`)
- `DOOR_PULSE_MS` (default: `1200`, for `rpi-gpio`)

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

Actuator backend modes:
- `command`: executes `FEEDER_ACTIVATE_CMD`, `DOOR_OPEN_CMD`, `DOOR_CLOSE_CMD`
- `rpi-gpio`: drives Raspberry Pi GPIO pins directly

Raspberry Pi GPIO startup:
1. Set `ACTUATOR_BACKEND=rpi-gpio` in `.env`.
2. Set pin env vars (`FEEDER_GPIO_PIN`, `DOOR_OPEN_GPIO_PIN`, `DOOR_CLOSE_GPIO_PIN`).
3. Start receiver:

```bash
cargo run --features pi-hw -- serve actuators
```
