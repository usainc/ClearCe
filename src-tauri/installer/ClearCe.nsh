; ClearCe-specific native pages. Included after MUI language definitions.
LangString CCWelcome ${LANG_ENGLISH} "Welcome to ClearCe Setup"
LangString CCWelcome ${LANG_TURKISH} "ClearCe Kurulumuna Hoş Geldiniz"
LangString CCIntro ${LANG_ENGLISH} "ClearCe is a local AI image enhancer for Windows.$\r$\n$\r$\nEnhance, upscale, and process images using your own computer.$\r$\n$\r$\nLocal AI Processing$\r$\nYour images are processed on your computer."
LangString CCIntro ${LANG_TURKISH} "ClearCe, Windows için yerel çalışan bir AI görsel iyileştirme uygulamasıdır.$\r$\n$\r$\nGörsellerinizi kendi bilgisayarınızda iyileştirin ve ölçeklendirin.$\r$\n$\r$\nYerel AI İşleme$\r$\nGörselleriniz kendi bilgisayarınızda işlenir."
LangString CCOptions ${LANG_ENGLISH} "Shortcut options"
LangString CCOptions ${LANG_TURKISH} "Kısayol seçenekleri"
LangString CCOptionsHint ${LANG_ENGLISH} "Choose shortcuts for your Windows account."
LangString CCOptionsHint ${LANG_TURKISH} "Windows hesabınız için kısayolları seçin."
LangString CCStart ${LANG_ENGLISH} "Start Menu shortcut"
LangString CCStart ${LANG_TURKISH} "Başlat Menüsü kısayolu"
LangString CCDesktop ${LANG_ENGLISH} "Desktop shortcut"
LangString CCDesktop ${LANG_TURKISH} "Masaüstü kısayolu"
LangString CCSummary ${LANG_ENGLISH} "Ready to install ClearCe"
LangString CCSummary ${LANG_TURKISH} "ClearCe kurulmaya hazır"
LangString CCSummaryHint ${LANG_ENGLISH} "Review your choices, then select Install."
LangString CCSummaryHint ${LANG_TURKISH} "Seçimlerinizi kontrol edin ve Kur düğmesine basın."
LangString CCScope ${LANG_ENGLISH} "Current user only — no administrator permission required."
LangString CCScope ${LANG_TURKISH} "Yalnızca geçerli kullanıcı — yönetici izni gerekmez."
LangString CCEngine ${LANG_ENGLISH} "AI Engine: not bundled. After installation, ClearCe guides you through selecting a trusted local Real-ESRGAN installation."
LangString CCEngine ${LANG_TURKISH} "AI Motoru: pakete dahil değildir. Kurulumdan sonra ClearCe, güvenilir bir yerel Real-ESRGAN kurulumunu seçmeniz için sizi yönlendirir."
LangString CCYes ${LANG_ENGLISH} "Yes"
LangString CCYes ${LANG_TURKISH} "Evet"
LangString CCNo ${LANG_ENGLISH} "No"
LangString CCNo ${LANG_TURKISH} "Hayır"
LangString CCFinish ${LANG_ENGLISH} "ClearCe has been installed successfully."
LangString CCFinish ${LANG_TURKISH} "ClearCe başarıyla kuruldu."
LangString CCLaunch ${LANG_ENGLISH} "Launch ClearCe"
LangString CCLaunch ${LANG_TURKISH} "ClearCe'yi çalıştır"
LangString CCWebView ${LANG_ENGLISH} "ClearCe requires Microsoft Edge WebView2 Runtime. Install it from Microsoft's official WebView2 website, then run this installer again. ClearCe Setup does not download or install it."
LangString CCWebView ${LANG_TURKISH} "ClearCe, Microsoft Edge WebView2 Runtime gerektirir. Microsoft'un resmi WebView2 sitesinden kurup bu kurulumu yeniden başlatın. ClearCe kurulumu bu bileşeni indirmez veya kurmaz."

Var CCStartChoice
Var CCDesktopChoice
Var CCStartControl
Var CCDesktopControl

Function CCOptionsPage
  Call SkipIfPassive
  !insertmacro MUI_HEADER_TEXT "$(CCOptions)" "$(CCOptionsHint)"
  nsDialogs::Create 1018
  Pop $0
  ${NSD_CreateLabel} 0 0 100% 24u "$(CCScope)"
  Pop $0
  ${NSD_CreateCheckbox} 0 35u 100% 18u "$(CCStart)"
  Pop $CCStartControl
  ${NSD_SetState} $CCStartControl $CCStartChoice
  ${NSD_CreateCheckbox} 0 62u 100% 18u "$(CCDesktop)"
  Pop $CCDesktopControl
  ${NSD_SetState} $CCDesktopControl $CCDesktopChoice
  ${NSD_CreateLabel} 0 98u 100% 48u "$(CCEngine)"
  Pop $0
  nsDialogs::Show
FunctionEnd

Function CCOptionsLeave
  ${NSD_GetState} $CCStartControl $CCStartChoice
  ${NSD_GetState} $CCDesktopControl $CCDesktopChoice
FunctionEnd

Function CCSummaryPage
  Call SkipIfPassive
  !insertmacro MUI_HEADER_TEXT "$(CCSummary)" "$(CCSummaryHint)"
  nsDialogs::Create 1018
  Pop $0
  StrCpy $1 "$(CCNo)"
  StrCpy $2 "$(CCNo)"
  ${If} $CCStartChoice = 1
    StrCpy $1 "$(CCYes)"
  ${EndIf}
  ${If} $CCDesktopChoice = 1
    StrCpy $2 "$(CCYes)"
  ${EndIf}
  ${NSD_CreateLabel} 0 0 100% 24u "ClearCe ${VERSION} — $(CCScope)"
  Pop $0
  ${NSD_CreateLabel} 0 30u 100% 35u "$INSTDIR"
  Pop $0
  ${NSD_CreateLabel} 0 72u 100% 36u "$(CCStart): $1$\r$\n$(CCDesktop): $2"
  Pop $0
  ${NSD_CreateLabel} 0 115u 100% 45u "$(CCEngine)"
  Pop $0
  nsDialogs::Show
FunctionEnd

Function CCSeedLanguage
  ; Only a versioned language token. Never read/write the settings store here.
  CreateDirectory "$APPDATA\${BUNDLEID}"
  FileOpen $0 "$APPDATA\${BUNDLEID}\installer-language.json" w
  ${IfNot} ${Errors}
    ${If} $LANGUAGE = ${LANG_TURKISH}
      FileWrite $0 '{"version":1,"language":"tr"}'
    ${Else}
      FileWrite $0 '{"version":1,"language":"en"}'
    ${EndIf}
    FileClose $0
  ${EndIf}
FunctionEnd
