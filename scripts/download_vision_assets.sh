#!/usr/bin/env bash
set -euo pipefail

mkdir -p models

MODEL_URL="https://github.com/onnx/models/raw/main/validated/vision/classification/mobilenet/model/mobilenetv2-7.onnx"
LABELS_URL="https://raw.githubusercontent.com/anishathalye/imagenet-simple-labels/master/imagenet-simple-labels.json"

echo "Downloading MobileNetV2 ONNX model..."
curl -L "$MODEL_URL" -o models/mobilenetv2-7.onnx

echo "Downloading ImageNet labels JSON..."
curl -L "$LABELS_URL" -o models/imagenet-labels.json

echo "Converting labels JSON to newline synset.txt..."
python3 - <<'PY'
import json
from pathlib import Path
labels = json.loads(Path("models/imagenet-labels.json").read_text(encoding="utf-8"))
Path("models/synset.txt").write_text("\n".join(labels), encoding="utf-8")
PY

echo "Vision assets ready:"
echo " - models/mobilenetv2-7.onnx"
echo " - models/synset.txt"
