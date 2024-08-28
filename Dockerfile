ARG WINBASE=mcr.microsoft.com/oss/kubernetes/windows-host-process-containers-base-image:v1.0.0
FROM --platform=$BUILDPLATFORM rust AS build
WORKDIR /src
RUN apt-get update && apt-get install -y mingw-w64 && rustup target add x86_64-pc-windows-gnu
COPY . .
RUN cargo build --target x86_64-pc-windows-gnu --release

FROM ${WINBASE}
COPY --from=build /src/target/x86_64-pc-windows-gnu/release/windows-netns-test.exe windows-netns-test.exe
ENTRYPOINT [ "windows-netns-test.exe" ]
