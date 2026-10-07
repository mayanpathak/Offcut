#!/bin/sh
# Uploads files to the asset host and prints the public path of each.
#
#   sh scripts/upload-assets.sh <folder> <file>...
#   sh scripts/upload-assets.sh media demo-1.mp4 demo-2.mp4 demo-3.mp4
#
# <folder> is `media` (demo clips, the sample clip) or `models` (V2).
#
# A file is stored as <folder>/<name>.<hash>.<extension>, where <hash> is the
# start of the SHA-256 of its content. A changed file therefore gets a new
# path, and the old one can be cached for ever: every upload is sent with
#   Cache-Control: public, max-age=31536000, immutable
#
# The printed path is what `assetUrl(...)` takes: relative to
# VITE_ASSET_BASE_URL. Nothing else is written to standard output.
#
# The asset host is reached through the S3 API, which Cloudflare R2 offers.
# Four variables must be set in the shell; they belong in no file of this
# repository:
#
#   ASSET_S3_ENDPOINT            https://<account-id>.r2.cloudflarestorage.com
#   ASSET_S3_BUCKET              the bucket name
#   ASSET_S3_ACCESS_KEY_ID       an API token of the bucket, with write access
#   ASSET_S3_SECRET_ACCESS_KEY
#   ASSET_S3_REGION              optional; `auto`, which is what R2 expects
#
# Needs curl 8.3 or later (for --aws-sigv4 with an upload) and sha256sum.
set -eu

CACHE_CONTROL="public, max-age=31536000, immutable"
HASH_LENGTH=16

fail() {
  echo "upload-assets: $1" >&2
  exit 1
}

[ "$#" -ge 2 ] || fail "usage: sh scripts/upload-assets.sh <media|models> <file>..."

folder=$1
shift
case "$folder" in
  media | models) ;;
  *) fail "the folder must be media or models, not: $folder" ;;
esac

for name in ASSET_S3_ENDPOINT ASSET_S3_BUCKET ASSET_S3_ACCESS_KEY_ID ASSET_S3_SECRET_ACCESS_KEY; do
  eval "value=\${$name:-}"
  [ -n "$value" ] || fail "$name is not set."
done
region=${ASSET_S3_REGION:-auto}
endpoint=${ASSET_S3_ENDPOINT%/}

# The type the asset host will answer with. A browser refuses a video or a
# WASM module that is served as the wrong type.
content_type() {
  case "$1" in
    mp4) echo "video/mp4" ;;
    webm) echo "video/webm" ;;
    jpg | jpeg) echo "image/jpeg" ;;
    png) echo "image/png" ;;
    webp) echo "image/webp" ;;
    json) echo "application/json" ;;
    wasm) echo "application/wasm" ;;
    onnx | bin) echo "application/octet-stream" ;;
    *) return 1 ;;
  esac
}

# Check every file before the first upload, so a wrong name uploads nothing.
for file in "$@"; do
  [ -f "$file" ] || fail "not a file: $file"
  base=$(basename "$file")
  case "$base" in
    *.*) ;;
    *) fail "the file has no extension: $file" ;;
  esac
  # The name becomes part of a URL.
  case "$base" in
    *[!A-Za-z0-9._-]*) fail "the name may hold only letters, digits, '.', '_' and '-': $base" ;;
  esac
  content_type "${base##*.}" > /dev/null || fail "no content type is known for: $base"
done

for file in "$@"; do
  base=$(basename "$file")
  extension=${base##*.}
  stem=${base%.*}
  sha256=$(sha256sum "$file" | cut -d' ' -f1)
  hash=$(printf '%s' "$sha256" | cut -c1-"$HASH_LENGTH")
  path="$folder/$stem.$hash.$extension"

  # The credentials go to curl on its standard input, not on its command
  # line, where every process of the machine could read them.
  printf 'user = "%s:%s"\n' "$ASSET_S3_ACCESS_KEY_ID" "$ASSET_S3_SECRET_ACCESS_KEY" \
    | curl --config - \
      --fail --silent --show-error \
      --aws-sigv4 "aws:amz:$region:s3" \
      --header "x-amz-content-sha256: $sha256" \
      --header "Cache-Control: $CACHE_CONTROL" \
      --header "Content-Type: $(content_type "$extension")" \
      --upload-file "$file" \
      --output /dev/null \
      "$endpoint/$ASSET_S3_BUCKET/$path" \
    || fail "the upload failed: $file"

  echo "$path"
done
