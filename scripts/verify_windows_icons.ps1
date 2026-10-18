param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [string]$Installer,
    [string]$Report = ".agents/analysis/windows-icons.json",
    [string]$ReadingIcon,
    [string]$PlaywrightModule = $env:PLAYWRIGHT_MODULE,
    [int]$DebugPort = 9345
)

$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$iconBytes = [IO.File]::ReadAllBytes((Join-Path $workspace "src-tauri/icons/icon.ico"))
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public class ReadUXIconCheck {
  public delegate bool ResourceName(IntPtr module, IntPtr type, IntPtr name, IntPtr parameter);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr LoadLibraryEx(string file, IntPtr reserved, uint flags);
  [DllImport("kernel32.dll")] public static extern bool FreeLibrary(IntPtr module);
  [DllImport("kernel32.dll", EntryPoint="EnumResourceNamesW")] public static extern bool EnumResourceNames(IntPtr module, IntPtr type, ResourceName callback, IntPtr parameter);
  [DllImport("kernel32.dll", EntryPoint="FindResourceW")] public static extern IntPtr FindResource(IntPtr module, IntPtr name, IntPtr type);
  [DllImport("kernel32.dll")] public static extern IntPtr LoadResource(IntPtr module, IntPtr resource);
  [DllImport("kernel32.dll")] public static extern IntPtr LockResource(IntPtr resource);
  [DllImport("kernel32.dll")] public static extern uint SizeofResource(IntPtr module, IntPtr resource);
  [DllImport("user32.dll", EntryPoint="SendMessageW")] public static extern IntPtr SendMessage(IntPtr window, uint message, IntPtr wparam, IntPtr lparam);
  public static byte[] Read(IntPtr module, IntPtr name, int type) {
    var resource = FindResource(module, name, (IntPtr)type);
    if(resource == IntPtr.Zero) throw new Exception("图标资源缺失");
    var bytes = new byte[SizeofResource(module, resource)];
    Marshal.Copy(LockResource(LoadResource(module, resource)), bytes, 0, bytes.Length);
    return bytes;
  }
  public static IntPtr FirstGroup(IntPtr module) {
    IntPtr group = IntPtr.Zero;
    EnumResourceNames(module, (IntPtr)14, (m,t,n,p) => { group=n; return false; }, IntPtr.Zero);
    return group;
  }
}
'@

function Assert-ExecutableIcon([string]$Path) {
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    $module = [ReadUXIconCheck]::LoadLibraryEx($resolved, [IntPtr]::Zero, 2)
    if ($module -eq [IntPtr]::Zero) { throw "无法读取可执行文件：$resolved" }
    try {
        $group = [ReadUXIconCheck]::Read($module, [ReadUXIconCheck]::FirstGroup($module), 14)
        $count = [BitConverter]::ToUInt16($group, 4)
        if ($count -ne [BitConverter]::ToUInt16($iconBytes, 4)) { throw "图标帧数不一致：$resolved" }
        for ($i = 0; $i -lt $count; $i++) {
            $id = [BitConverter]::ToUInt16($group, 6 + 14 * $i + 12)
            $actual = [ReadUXIconCheck]::Read($module, [IntPtr][int]$id, 3)
            $offset = [BitConverter]::ToUInt32($iconBytes, 6 + 16 * $i + 12)
            $length = [BitConverter]::ToUInt32($iconBytes, 6 + 16 * $i + 8)
            $expected = $iconBytes[$offset..($offset + $length - 1)]
            if ([Convert]::ToBase64String($actual) -ne [Convert]::ToBase64String($expected)) {
                throw "图标内容不一致：$resolved，第 $i 帧"
            }
        }
        return @{path=$resolved; frames=$count; matches=$true}
    } finally { $null = [ReadUXIconCheck]::FreeLibrary($module) }
}

$result = @{executable=(Assert-ExecutableIcon $Executable)}
if ($Installer) { $result.installer = Assert-ExecutableIcon $Installer }
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
$oldData = $env:READUX_DATA_DIR
$oldBrowserArguments = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$probeData = Join-Path ([IO.Path]::GetTempPath()) ("readux-icon-" + [Guid]::NewGuid().ToString("N"))
if (-not [IO.Path]::GetFullPath($probeData).StartsWith([IO.Path]::GetFullPath([IO.Path]::GetTempPath()), [StringComparison]::OrdinalIgnoreCase)) {
    throw "图标测试的数据目录超出临时目录范围"
}
try {
    $env:READUX_DATA_DIR = $probeData
    if ($ReadingIcon) {
        if (Get-NetTCPConnection -State Listen -LocalPort $DebugPort -ErrorAction SilentlyContinue) {
            throw "图标测试端口已占用：$DebugPort"
        }
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$DebugPort"
    }
    $process = Start-Process -FilePath $executablePath -WindowStyle Hidden -PassThru
    $small = [IntPtr]::Zero
    $big = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 80; $attempt++) {
        Start-Sleep -Milliseconds 250
        $process.Refresh()
        if ($process.HasExited) { throw "图标验证时主程序退出：$($process.ExitCode)" }
        if ($process.MainWindowHandle -ne [IntPtr]::Zero) {
            $small = [ReadUXIconCheck]::SendMessage($process.MainWindowHandle, 0x7F, [IntPtr]::Zero, [IntPtr]::Zero)
            $big = [ReadUXIconCheck]::SendMessage($process.MainWindowHandle, 0x7F, [IntPtr]1, [IntPtr]::Zero)
            if ($small -ne [IntPtr]::Zero -and $big -ne [IntPtr]::Zero) { break }
        }
    }
    if ($small -eq [IntPtr]::Zero -or $big -eq [IntPtr]::Zero) { throw "主窗口缺少标题栏或任务栏图标" }
    $result.window = @{small=$small.ToInt64(); big=$big.ToInt64(); title=$process.MainWindowTitle}
    if ($ReadingIcon) {
        $endpoint = "http://127.0.0.1:$DebugPort"
        $ready = $false
        for ($attempt = 0; $attempt -lt 80; $attempt++) {
            try { $null = Invoke-RestMethod "$endpoint/json/version"; $ready = $true; break } catch { Start-Sleep -Milliseconds 250 }
        }
        if (-not $ready) { throw "图标测试无法连接 WebView2" }
        $readingPath = (Resolve-Path -LiteralPath $ReadingIcon).Path
        & node (Join-Path $PSScriptRoot "verify_taskbar_ipc.mjs") $endpoint $readingPath $PlaywrightModule "reading"
        if ($LASTEXITCODE -ne 0) { throw "阅读图标 IPC 验证失败" }
        $readingBig = [ReadUXIconCheck]::SendMessage($process.MainWindowHandle, 0x7F, [IntPtr]1, [IntPtr]::Zero)
        $readingSmall = [ReadUXIconCheck]::SendMessage($process.MainWindowHandle, 0x7F, [IntPtr]::Zero, [IntPtr]::Zero)
        if ($readingBig -eq [IntPtr]::Zero -or $readingBig -eq $big -or $readingSmall -ne $readingBig) {
            throw "阅读图标未正确替换窗口的大、小图标"
        }
        Add-Type -AssemblyName System.Drawing
        $bitmap = [System.Drawing.Icon]::FromHandle($readingBig).ToBitmap()
        try { $bitmap.Save((Join-Path (Split-Path $readingPath) "native-reading-taskbar.png"), [System.Drawing.Imaging.ImageFormat]::Png) }
        finally { $bitmap.Dispose() }
        & node (Join-Path $PSScriptRoot "verify_taskbar_ipc.mjs") $endpoint $readingPath $PlaywrightModule "restore"
        if ($LASTEXITCODE -ne 0) { throw "默认图标恢复 IPC 验证失败" }
        $restored = [ReadUXIconCheck]::SendMessage($process.MainWindowHandle, 0x7F, [IntPtr]1, [IntPtr]::Zero)
        $restoredSmall = [ReadUXIconCheck]::SendMessage($process.MainWindowHandle, 0x7F, [IntPtr]::Zero, [IntPtr]::Zero)
        if ($restored -ne $big -or $restoredSmall -ne $small) { throw "默认窗口图标未恢复" }
        $result.reading = @{big=$readingBig.ToInt64(); small=$readingSmall.ToInt64(); restored=$restored.ToInt64(); restoredSmall=$restoredSmall.ToInt64()}
    }
    $reportPath = [IO.Path]::GetFullPath((Join-Path $workspace $Report))
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($reportPath)) | Out-Null
    [IO.File]::WriteAllText($reportPath, ($result | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))
    $result | ConvertTo-Json -Depth 5
} finally {
    if ($process -and -not $process.HasExited) {
        $null = $process.CloseMainWindow()
        if (-not $process.WaitForExit(5000)) { Stop-Process -Id $process.Id -Force }
    }
    $env:READUX_DATA_DIR = $oldData
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $oldBrowserArguments
    if (Test-Path -LiteralPath $probeData) { Remove-Item -LiteralPath $probeData -Recurse -Force }
}
