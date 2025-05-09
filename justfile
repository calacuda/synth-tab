_:
  @just --list

run:
  x run --arch arm64 --device $(x devices | rg "android arm64" | cut -d ' ' -f 1)

run-release:
  x run -r --arch arm64 --device $(x devices | rg "android arm64" | cut -d ' ' -f 1)

x-build:
  x build -r --arch arm64 --platform android
  cp target/x/release/android/arm64/cargo/aarch64-linux-android/release/libandroid_iced_example.so app/src/main/jniLibs/arm64-v8a/libexample.so

build:
  cargo apk build -r --target aarch64-linux-android || true
  cp target/aarch64-linux-android/release/libandroid_iced_example.so app/src/main/jniLibs/arm64-v8a/libexample.so

dev:
  cargo apk build --target aarch64-linux-android || true
  cp target/aarch64-linux-android/debug/libandroid_iced_example.so app/src/main/jniLibs/arm64-v8a/libexample.so

get-devs:
  x devices | rg "android arm64" | cut -d ' ' -f 1
