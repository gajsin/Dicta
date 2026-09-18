Var DictaInstalled

!macro NSIS_HOOK_POSTINSTALL
  StrCpy $DictaInstalled 1
  ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --sync-shell-icons'
!macroend

; The interactive finish page creates the desktop shortcut after POSTINSTALL.
Function .onGUIEnd
  ${If} $DictaInstalled == 1
    ExecWait '"$INSTDIR\dicta.exe" --sync-shell-icons'
  ${EndIf}
FunctionEnd
