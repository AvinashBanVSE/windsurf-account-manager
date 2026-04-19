# Administrator Permission Setup Guide

## Method 1: Using PowerShell Script (Recommended)

1. First build the project normally:
   ```bash
   npm run tauri build
   ```

2. After successful build, run the PowerShell script:
   ```powershell
   .\set_admin_manifest.ps1 "src-tauri\target\release\windsurf-account-manager.exe"
   ```

   Note: Windows SDK or Visual Studio is required. The script will automatically find the mt.exe tool.

## Method 2: Using Batch File (Simplest)

Simply run:
```batch
build_with_admin.bat
```

This batch file will automatically build and set administrator permissions.

## Method 3: Using Resource Hacker (Manual)

If you don't have Windows SDK, you can use a third-party tool:

1. Download [Resource Hacker](http://www.angusj.com/resourcehacker/)

2. Open the built exe file

3. From the menu, select Action > Add from a blank script

4. Enter the following:
   ```
   1 24 "admin.manifest"
   ```

5. Create an admin.manifest file with the following content:
   ```xml
   <?xml version="1.0" encoding="UTF-8" standalone="yes"?>
   <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
     <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
       <security>
         <requestedPrivileges>
           <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
         </requestedPrivileges>
       </security>
     </trustInfo>
   </assembly>
   ```

6. Save and rebuild the resources

## Method 4: Using Manifest Tool (mt.exe) Command Line

If Visual Studio or Windows SDK is already installed:

```cmd
"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\mt.exe" -manifest admin.manifest -outputresource:windsurf-account-manager.exe;1
```

The path may vary depending on the version.

## Important Notes

- After setting administrator permissions, the UAC dialog will appear every time the program is launched
- Users must confirm to run the program
- Modifying the registry under `HKEY_LOCAL_MACHINE` requires administrator permissions
- It is recommended to back up the original MachineGuid value before making modifications