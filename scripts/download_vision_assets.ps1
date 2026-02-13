$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path "models" | Out-Null

$modelUrl = "https://github.com/onnx/models/raw/main/validated/vision/classification/mobilenet/model/mobilenetv2-7.onnx"
$labelsUrl = "https://raw.githubusercontent.com/anishathalye/imagenet-simple-labels/master/imagenet-simple-labels.json"

Write-Host "Downloading MobileNetV2 ONNX model..."
Invoke-WebRequest -Uri $modelUrl -OutFile "models/mobilenetv2-7.onnx"

Write-Host "Downloading ImageNet labels JSON..."
Invoke-WebRequest -Uri $labelsUrl -OutFile "models/imagenet-labels.json"

Write-Host "Converting labels JSON to newline synset.txt..."
$labels = Get-Content "models/imagenet-labels.json" -Raw | ConvertFrom-Json
$labels | Set-Content "models/synset.txt"

Write-Host "Vision assets ready:"
Write-Host " - models/mobilenetv2-7.onnx"
Write-Host " - models/synset.txt"
