ARG WINBASE=mcr.microsoft.com/windows/servercore:ltsc2022
FROM --platform=$BUILDPLATFORM rust AS build
WORKDIR /src
RUN apt-get update && apt-get install -y mingw-w64 && rustup target add x86_64-pc-windows-gnu
COPY . .
RUN cargo build --target x86_64-pc-windows-gnu --release

FROM ${WINBASE}
COPY --from=build /src/target/x86_64-pc-windows-gnu/release/windows-netns-test.exe windows-netns-test.exe
ENTRYPOINT [ "windows-netns-test.exe" ]
