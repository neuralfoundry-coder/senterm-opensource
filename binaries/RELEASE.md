# Release Guide

GitHub Releases를 통해 바이너리를 배포하는 방법을 안내합니다.

## 사전 요구사항

- GitHub CLI (`gh`) 설치: `brew install gh`
- GitHub 로그인: `gh auth login`

## 빌드 방법

### macOS Universal Binary (Intel + Apple Silicon)

```bash
# 두 아키텍처용 빌드
cargo build --release --target x86_64-apple-darwin --no-default-features
cargo build --release --target aarch64-apple-darwin --no-default-features

# Universal binary 생성
lipo -create \
    target/x86_64-apple-darwin/release/senterm \
    target/aarch64-apple-darwin/release/senterm \
    -output senterm

# 패키징
tar -czvf senterm-macos-universal.tar.gz senterm
```

### Linux x86_64 Binary

```bash
# 타겟 추가 (최초 1회)
rustup target add x86_64-unknown-linux-gnu

# cargo-zigbuild 설치 (최초 1회)
cargo install cargo-zigbuild
brew install zig

# 빌드
cargo zigbuild --release --target x86_64-unknown-linux-gnu --no-default-features

# 패키징
cp target/x86_64-unknown-linux-gnu/release/senterm .
tar -czvf senterm-linux-x86_64.tar.gz senterm
```

## GitHub Release 생성

### 방법 1: GitHub CLI 사용 (권장)

```bash
# 버전 태그 생성
VERSION="v0.1.0"
git tag -a $VERSION -m "Release $VERSION"
git push origin $VERSION

# Release 생성 및 바이너리 업로드
gh release create $VERSION \
    --title "Senterm $VERSION" \
    --notes "## What's New
- Initial release
- Miller Columns file navigation
- Integrated shell panel
- Syntax highlighting & image preview
- macOS (Universal) and Linux (x86_64) support" \
    senterm-macos-universal.tar.gz \
    senterm-linux-x86_64.tar.gz
```

### 방법 2: GitHub 웹 UI 사용

1. GitHub 레포지토리로 이동
2. **Releases** → **Draft a new release** 클릭
3. **Choose a tag** → 새 태그 입력 (예: `v0.1.0`)
4. **Release title** 입력 (예: `Senterm v0.1.0`)
5. **Release notes** 작성
6. **Attach binaries** 섹션에서 파일 업로드:
   - `senterm-macos-universal.tar.gz`
   - `senterm-linux-x86_64.tar.gz`
7. **Publish release** 클릭

## 버전 네이밍 규칙

- Semantic Versioning 사용: `vMAJOR.MINOR.PATCH`
- 예: `v0.1.0`, `v0.1.1`, `v0.2.0`, `v1.0.0`

## 릴리스 확인

```bash
# 최신 릴리스 확인
gh release list

# 특정 릴리스 상세 보기
gh release view v0.1.0
```

## 릴리스 후 설치 테스트

```bash
# macOS/Linux에서 테스트
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-universal.sh | bash

# 특정 버전 설치 테스트
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-universal.sh | bash -s -- --version v0.1.0
```

## 원클릭 릴리스 스크립트

아래 스크립트를 사용하면 빌드부터 릴리스까지 한 번에 수행할 수 있습니다:

```bash
#!/bin/bash
set -e

VERSION="${1:-v0.1.0}"
NOTES="${2:-Release $VERSION}"

echo "=== Building for macOS (x86_64) ==="
cargo build --release --target x86_64-apple-darwin --no-default-features

echo "=== Building for macOS (arm64) ==="
cargo build --release --target aarch64-apple-darwin --no-default-features

echo "=== Creating Universal Binary ==="
lipo -create \
    target/x86_64-apple-darwin/release/senterm \
    target/aarch64-apple-darwin/release/senterm \
    -output senterm-macos
tar -czvf senterm-macos-universal.tar.gz -C . senterm-macos
mv senterm-macos senterm && rm -f senterm

echo "=== Building for Linux (x86_64) ==="
cargo zigbuild --release --target x86_64-unknown-linux-gnu --no-default-features
cp target/x86_64-unknown-linux-gnu/release/senterm .
tar -czvf senterm-linux-x86_64.tar.gz senterm
rm senterm

echo "=== Creating Release $VERSION ==="
git tag -a $VERSION -m "$NOTES"
git push origin $VERSION

gh release create $VERSION \
    --title "Senterm $VERSION" \
    --notes "$NOTES" \
    senterm-macos-universal.tar.gz \
    senterm-linux-x86_64.tar.gz

echo "=== Cleanup ==="
rm -f senterm-macos-universal.tar.gz senterm-linux-x86_64.tar.gz

echo "=== Done! ==="
echo "Release $VERSION created successfully!"
```

사용법:
```bash
./release.sh v0.1.0 "Initial release with Miller Columns navigation"
```

