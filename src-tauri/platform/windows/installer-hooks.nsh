; Keep PRODUCTNAME and registry identities stable across installer languages.
LangString NativeDisplayName 1033 "PopRaKo Native"
LangString NativeDisplayName 2052 "白杨子 N"

; The executable name follows Cargo's default-run; no custom startMenuFolder.
!macro LocalizeShortcut directory sourceName targetName
  !insertmacro IsShortcutTarget "${directory}\${sourceName}.lnk" "$INSTDIR\poprako-native.exe"
  Pop $0
  ${If} $0 = 1
    !insertmacro IsShortcutTarget "${directory}\${targetName}.lnk" "$INSTDIR\poprako-native.exe"
    Pop $0
    ${If} $0 = 1
      Delete "${directory}\${sourceName}.lnk"
    ${Else}
      Rename "${directory}\${sourceName}.lnk" "${directory}\${targetName}.lnk"
    ${EndIf}
  ${EndIf}
!macroend

Function LocalizeApplicationShortcuts
  ; Only touch shortcuts owned by this installation. Finish-page desktop
  ; shortcuts are created after POSTINSTALL, so repeat on GUI completion.
  ReadRegStr $0 SHCTX "Software\Microsoft\Windows\CurrentVersion\Uninstall\PopRaKo Native" "DisplayName"
  ${If} $0 != "$(NativeDisplayName)"
    Return
  ${EndIf}
  ${If} $LANGUAGE = 2052
    !insertmacro LocalizeShortcut "$SMPROGRAMS" "PopRaKo Native" "白杨子 N"
    !insertmacro LocalizeShortcut "$DESKTOP" "PopRaKo Native" "白杨子 N"
  ${Else}
    !insertmacro LocalizeShortcut "$SMPROGRAMS" "白杨子 N" "PopRaKo Native"
    !insertmacro LocalizeShortcut "$DESKTOP" "白杨子 N" "PopRaKo Native"
  ${EndIf}
FunctionEnd

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "$(NativeDisplayName)"
  Call LocalizeApplicationShortcuts
!macroend

Function .onGUIEnd
  Call LocalizeApplicationShortcuts
FunctionEnd

!macro RemoveLocalizedShortcut directory
  !insertmacro IsShortcutTarget "${directory}\白杨子 N.lnk" "$INSTDIR\poprako-native.exe"
  Pop $0
  ${If} $0 = 1
    !insertmacro UnpinShortcut "${directory}\白杨子 N.lnk"
    Delete "${directory}\白杨子 N.lnk"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    !insertmacro RemoveLocalizedShortcut "$SMPROGRAMS"
    !insertmacro RemoveLocalizedShortcut "$DESKTOP"
  ${EndIf}
!macroend
