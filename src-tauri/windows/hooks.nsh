; NSIS installer hooks, included by Tauri's installer template.
;
; Orbsmith keeps its data in %APPDATA%\Orbsmith rather than in the bundle
; identifier folder that the template's "Delete the application data" option
; removes, so that folder is removed here under the same conditions.

!include LogicLib.nsh

!define /ifndef FILE_ATTRIBUTE_DIRECTORY 0x10
!define /ifndef FILE_ATTRIBUTE_REPARSE_POINT 0x400

; Deletes a directory tree like `RMDir /r`, except that junctions and
; symbolic links are removed themselves and never followed: `RMDir /r`
; recurses into them and would delete whatever they point to.
;
; Usage: Push <directory>, then Call un.OrbsmithRemoveTree.
Function un.OrbsmithRemoveTree
  Exch $0 ; directory
  Push $1 ; find handle
  Push $2 ; entry name
  Push $3 ; entry attributes
  Push $4 ; scratch

  System::Call 'kernel32::GetFileAttributesW(w r0) i .r3'
  ${If} $3 = -1
    Goto done
  ${EndIf}
  IntOp $4 $3 & ${FILE_ATTRIBUTE_REPARSE_POINT}
  ${If} $4 <> 0
    RMDir "$0" ; without /r: removes the link only
    Goto done
  ${EndIf}

  FindFirst $1 $2 "$0\*.*"
  ${DoWhile} $2 != ""
    ${If} $2 != "."
    ${AndIf} $2 != ".."
      System::Call 'kernel32::GetFileAttributesW(w "$0\$2") i .r3'
      ${If} $3 <> -1
        IntOp $4 $3 & ${FILE_ATTRIBUTE_REPARSE_POINT}
        ${If} $4 <> 0
          IntOp $4 $3 & ${FILE_ATTRIBUTE_DIRECTORY}
          ${If} $4 <> 0
            RMDir "$0\$2"
          ${Else}
            Delete "$0\$2"
          ${EndIf}
        ${Else}
          IntOp $4 $3 & ${FILE_ATTRIBUTE_DIRECTORY}
          ${If} $4 <> 0
            Push "$0\$2"
            Call un.OrbsmithRemoveTree
          ${Else}
            Delete "$0\$2"
          ${EndIf}
        ${EndIf}
      ${EndIf}
    ${EndIf}
    FindNext $1 $2
  ${Loop}
  FindClose $1
  RMDir "$0"

  done:
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    ; An empty $APPDATA would turn the path into \Orbsmith on the current drive.
    ${If} $APPDATA != ""
      Push "$APPDATA\Orbsmith"
      Call un.OrbsmithRemoveTree
    ${EndIf}
  ${EndIf}
!macroend
