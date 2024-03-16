FROM mcr.microsoft.com/windows/servercore:ltsc2019

SHELL ["powershell", "-Command", "$ErrorActionPreference = 'Stop';"]

ENV OHOS_NDK_HOME "C:\sdk"

ADD ./config C:\\Users\\ContainerAdministrator\\.cargo\\config

RUN New-Item -ItemType Directory -Path C:\\harmony, C:\\sdk; \
    Start-BitsTransfer -Source https://repo.huaweicloud.com/openharmony/os/4.0-Release/ohos-sdk-windows_linux-public.tar.gz -Destination C:\\harmony\\ohos-sdk-windows_linux-public.tar.gz; \
    New-Item -ItemType Directory -Path C:\\harmony\\ohos-sdk-windows_linux-public; \
    tar -xzf C:\\harmony\\ohos-sdk-windows_linux-public.tar.gz -C C:\\harmony\\ohos-sdk-windows_linux-public; \
    Get-ChildItem -Path "C:\\harmony\\ohos-sdk-windows_linux-public\\ohos-sdk\\windows\\*.zip" | ForEach-Object {Expand-Archive -Path $_.FullName -DestinationPath "C:\\sdk"} \
    Remove-Item -Path C:\\harmony -Recurse; \
    Start-BitsTransfer -Source https://win.rustup.rs -Destination C:\\rustup-init.exe; \
    Start-Process -FilePath C:\\rustup-init.exe -ArgumentList '-y' -Wait; \
    Remove-Item -Path C:\\rustup-init.exe;