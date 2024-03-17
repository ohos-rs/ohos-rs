FROM mcr.microsoft.com/windows/servercore:ltsc2019

SHELL ["powershell", "-Command", "$ErrorActionPreference = 'Stop';"]

ENV OHOS_NDK_HOME "C:\sdk"

ADD ./config C:\\Users\\ContainerAdministrator\\.cargo\\config

RUN New-Item -ItemType Directory -Path C:\\harmony, C:\\sdk, C:\\tmp; \
    Start-BitsTransfer -Source https://aka.ms/vs/16/release/vs_buildtools.exe -Destination C:\\tmp\\vs_buildtools.exe; \
    Start-Process -FilePath C:\\tmp\\vs_buildtools.exe -ArgumentList '--quiet --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools  --add Microsoft.VisualStudio.Workload.MSBuildTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows10SDK.17763' -Wait; \
    Start-BitsTransfer -Source https://repo.huaweicloud.com/openharmony/os/4.0-Release/ohos-sdk-windows_linux-public.tar.gz -Destination C:\\harmony\\ohos-sdk-windows_linux-public.tar.gz; \
    New-Item -ItemType Directory -Path C:\\harmony\\ohos-sdk-windows_linux-public; \
    tar -xzf C:\\harmony\\ohos-sdk-windows_linux-public.tar.gz -C C:\\harmony\\ohos-sdk-windows_linux-public; \
    Get-ChildItem -Path "C:\\harmony\\ohos-sdk-windows_linux-public\\ohos-sdk\\windows\\*.zip" | ForEach-Object {Expand-Archive -Path $_.FullName -DestinationPath "C:\\sdk"}; \
    Remove-Item -Path C:\\harmony -Recurse; \
    Start-BitsTransfer -Source https://win.rustup.rs -Destination C:\\tmp\\rustup-init.exe; \
    Start-Process -FilePath C:\\tmp\\rustup-init.exe -ArgumentList '-y' -Wait; \
    Remove-Item -Path C:\\tmp -Recurse; 