# ClearCe Installation Guide / Kurulum Rehberi

ClearCe 0.1.0 is a Windows x64 public beta. This guide covers installation, first-run AI engine setup, verification, reinstall/uninstall behavior and common setup problems.

ClearCe 0.1.0 Windows x64 public beta sürümüdür. Bu rehber kurulum, ilk açılışta AI motoru kurulumu, doğrulama, yeniden kurulum/kaldırma davranışı ve sık karşılaşılan kurulum sorunlarını kapsar.

- [English](#english)
- [Türkçe](#türkçe)

---

## English

### 1. Requirements

Before installing ClearCe, make sure the PC has:

- Windows x64
- Microsoft Edge WebView2 Runtime
- A Vulkan-compatible NVIDIA, AMD or Intel GPU and a current graphics driver
- Internet access only if you want ClearCe to download the recommended managed Real-ESRGAN package for you

ClearCe does **not** require Node.js, Rust, Python or CUDA for normal installed-app use.

### 2. Download ClearCe

Download the installer only from the official release page:

**[ClearCe 0.1.0 — Public Beta](https://github.com/usainc/ClearCe/releases/tag/v0.1.0)**

Installer file:

`ClearCe_0.1.0_x64-setup.exe`

Expected SHA-256:

```text
32c88d6dcce5f4d4774f9b7c59fa2f9222c05a3d594914b41d60947afe0ba116
```

Optional PowerShell verification:

```powershell
Get-FileHash .\ClearCe_0.1.0_x64-setup.exe -Algorithm SHA256
```

The returned value should match the SHA-256 above and the `SHA256SUMS.txt` file attached to the GitHub Release.

### 3. Run the installer

The ClearCe installer is a per-user installer and normally does not require Administrator privileges.

During setup you can:

1. Choose **English** or **Türkçe**.
2. Review the welcome page.
3. Choose the installation directory.
4. Choose shortcut options.
5. Review the installation summary.
6. Install ClearCe.
7. Launch ClearCe when setup finishes.

The default installation location is under the current user's local application data. Spaces and Unicode characters in the selected path are supported.

### 4. Windows SmartScreen

ClearCe 0.1.0 is currently distributed with an **unsigned Windows installer**. Windows SmartScreen may therefore display a warning because the installer does not yet have an Authenticode publisher signature.

For safety:

- download ClearCe only from this repository's official GitHub Release;
- compare the SHA-256 value before running the installer if you want to verify the file;
- do not disable Windows Defender or SmartScreen for ClearCe.

If you are not comfortable running unsigned software, wait for a future signed release.

### 5. First launch: install the AI engine

Real-ESRGAN is **not bundled inside the ClearCe installer**.

On first launch:

1. Open **Models**.
2. Leave model selection on **Auto (Recommended)** unless you have a reason to choose manually.
3. ClearCe detects available Vulkan devices and shows a recommendation for the PC.
4. Select **Download Recommended**.
5. ClearCe downloads the pinned official Real-ESRGAN package after your explicit action.
6. The package SHA-256 and archive layout are verified before activation.
7. ClearCe extracts the approved files into its managed engine directory.
8. The engine and Vulkan device are health-checked.
9. When the status becomes ready, open an image and start processing.

After the managed engine has been installed successfully, normal image inference runs locally and does not require an internet connection.

### 6. Manual AI engine setup

If you already have a compatible Real-ESRGAN NCNN Vulkan runtime:

1. Open **Models**.
2. Choose **Select Local AI Engine**.
3. Select the extracted engine directory containing `realesrgan-ncnn-vulkan.exe` and the required model files.
4. ClearCe validates and imports only the expected runtime files.

See [AI Engine Setup](ENGINE_SETUP.md) and [Engine & Model Policy](ENGINE_MODELS.md) for details.

### 7. Start enhancing

ClearCe 0.1.0 supports:

- 2x, 4x, 8x and 12x enhancement
- PNG, JPG and WebP
- batch processing
- Before / After comparison
- general-photo and anime/illustration model recommendations

For 8x and 12x, use smaller source images first: these modes require substantially more GPU memory, RAM, temporary disk space and processing time.

### 8. Reinstall and uninstall

Reinstalling or updating ClearCe preserves the existing user configuration where applicable, including settings, history and healthy engine configuration.

Uninstalling ClearCe removes installer-owned application files and shortcuts. It does **not** delete source images or completed output images. ClearCe 0.1.0 also preserves its user settings/history/engine data so a later reinstall can reuse them.

### 9. Troubleshooting

**No compatible Vulkan device detected**  
Install or update the graphics driver from the GPU manufacturer's official source and reopen ClearCe.

**WebView2 is missing**  
Install Microsoft Edge WebView2 Runtime from Microsoft's official WebView2 page, then run ClearCe Setup again.

**Managed engine download fails**  
Check the internet connection or proxy configuration and retry. A failed managed installation does not replace a previously working engine.

**Engine is invalid or missing files**  
Use **Reinstall Managed Engine** or select a complete compatible local engine directory again.

**8x/12x is rejected or runs out of resources**  
Try a smaller input image or a lower scale and make sure sufficient temporary disk space is available.

For reproducible application bugs, use [GitHub Issues](https://github.com/usainc/ClearCe/issues). Security vulnerabilities should be reported through [private vulnerability reporting](https://github.com/usainc/ClearCe/security/advisories/new), not as public issues.

---

## Türkçe

### 1. Gereksinimler

ClearCe'yi kurmadan önce bilgisayarda şunların bulunduğundan emin olun:

- Windows x64
- Microsoft Edge WebView2 Runtime
- Vulkan uyumlu NVIDIA, AMD veya Intel ekran kartı ve güncel ekran kartı sürücüsü
- Yalnızca ClearCe'nin önerilen yönetilen Real-ESRGAN paketini indirmesini istiyorsanız internet bağlantısı

Kurulu uygulamayı normal şekilde kullanmak için Node.js, Rust, Python veya CUDA gerekmez.

### 2. ClearCe'yi indirin

Kurulum dosyasını yalnızca resmî sürüm sayfasından indirin:

**[ClearCe 0.1.0 — Public Beta](https://github.com/usainc/ClearCe/releases/tag/v0.1.0)**

Kurulum dosyası:

`ClearCe_0.1.0_x64-setup.exe`

Beklenen SHA-256:

```text
32c88d6dcce5f4d4774f9b7c59fa2f9222c05a3d594914b41d60947afe0ba116
```

İsterseniz PowerShell ile doğrulayabilirsiniz:

```powershell
Get-FileHash .\ClearCe_0.1.0_x64-setup.exe -Algorithm SHA256
```

Çıkan değer yukarıdaki SHA-256 ve GitHub Release'e eklenmiş `SHA256SUMS.txt` dosyasındaki değerle aynı olmalıdır.

### 3. Kurulum sihirbazını çalıştırın

ClearCe kullanıcı bazlı kurulur ve normal durumda Yönetici izni istemez.

Kurulum sırasında:

1. **English** veya **Türkçe** seçin.
2. Karşılama ekranını inceleyin.
3. Kurulum klasörünü seçin.
4. Kısayol seçeneklerini belirleyin.
5. Kurulum özetini kontrol edin.
6. ClearCe'yi kurun.
7. Kurulum tamamlandığında ClearCe'yi çalıştırın.

Varsayılan kurulum konumu mevcut Windows kullanıcısının yerel uygulama verileri altındadır. Boşluk ve Unicode karakter içeren klasör yolları desteklenir.

### 4. Windows SmartScreen

ClearCe 0.1.0 şu anda **dijital imzasız bir Windows installer** ile dağıtılıyor. Bu nedenle Windows SmartScreen, Authenticode yayıncı imzası bulunmadığı için uyarı gösterebilir.

Güvenlik için:

- ClearCe'yi yalnızca bu deponun resmî GitHub Release sayfasından indirin;
- isterseniz çalıştırmadan önce SHA-256 değerini karşılaştırın;
- ClearCe için Windows Defender veya SmartScreen'i kapatmayın.

İmzasız yazılım çalıştırmak istemiyorsanız imzalı bir gelecek sürümü bekleyin.

### 5. İlk açılış: AI motorunu kurun

Real-ESRGAN, **ClearCe installer paketinin içine gömülü değildir**.

İlk açılışta:

1. **Modeller** ekranını açın.
2. Özel bir nedeniniz yoksa seçim modunu **Otomatik (Önerilen)** olarak bırakın.
3. ClearCe kullanılabilir Vulkan cihazlarını algılar ve bilgisayarınıza uygun öneriyi gösterir.
4. **Önerileni İndir** düğmesine basın.
5. ClearCe yalnızca sizin açık işleminizden sonra sabitlenmiş resmî Real-ESRGAN paketini indirir.
6. Paket etkinleştirilmeden önce SHA-256 ve arşiv yapısı doğrulanır.
7. Onaylanan dosyalar ClearCe'nin yönetilen motor klasörüne çıkarılır.
8. Motor ve Vulkan cihazı sağlık kontrolünden geçirilir.
9. Durum hazır olduğunda bir görsel açıp işlemeye başlayabilirsiniz.

Yönetilen motor başarıyla kurulduktan sonra normal görsel işleme yerel olarak çalışır ve internet bağlantısına ihtiyaç duymaz.

### 6. Manuel AI motoru kurulumu

Uyumlu bir Real-ESRGAN NCNN Vulkan motorunuz zaten varsa:

1. **Modeller** ekranını açın.
2. **Yerel AI Motorunu Seç** seçeneğini kullanın.
3. `realesrgan-ncnn-vulkan.exe` ve gerekli model dosyalarını içeren çıkarılmış motor klasörünü seçin.
4. ClearCe yalnızca beklenen çalışma zamanı dosyalarını doğrular ve içe aktarır.

Ayrıntılar için [AI Motoru Kurulumu](ENGINE_SETUP.md) ve [Motor & Model Politikası](ENGINE_MODELS.md) belgelerine bakın.

### 7. Görselleri iyileştirmeye başlayın

ClearCe 0.1.0 şunları destekler:

- 2x, 4x, 8x ve 12x iyileştirme
- PNG, JPG ve WebP
- toplu işlem
- Önce / Sonra karşılaştırması
- genel fotoğraf ve anime/illüstrasyon model önerileri

8x ve 12x için önce daha küçük kaynak görseller kullanın. Bu modlar daha fazla GPU belleği, RAM, geçici disk alanı ve işlem süresi gerektirir.

### 8. Yeniden kurulum ve kaldırma

ClearCe'yi yeniden kurmak veya güncellemek mevcut kullanıcı yapılandırmasını mümkün olduğu ölçüde korur; buna ayarlar, geçmiş ve sağlıklı motor yapılandırması dahildir.

ClearCe kaldırıldığında installer'a ait uygulama dosyaları ve kısayollar kaldırılır. Kaynak görselleriniz veya tamamlanmış çıktı görselleriniz silinmez. ClearCe 0.1.0 ayrıca kullanıcı ayarlarını/geçmişini/motor verilerini korur; böylece daha sonraki bir yeniden kurulum bunları tekrar kullanabilir.

### 9. Sorun giderme

**Uyumlu Vulkan cihazı algılanmadı**  
Ekran kartı sürücünüzü üreticinin resmî kaynağından kurun veya güncelleyin ve ClearCe'yi yeniden açın.

**WebView2 eksik**  
Microsoft'un resmî WebView2 sayfasından Microsoft Edge WebView2 Runtime'ı kurun ve ClearCe kurulumunu tekrar çalıştırın.

**Yönetilen motor indirilemiyor**  
İnternet bağlantısını veya proxy ayarını kontrol edip tekrar deneyin. Başarısız bir yönetilen motor kurulumu daha önce çalışan motoru değiştirmez.

**Motor geçersiz veya gerekli dosyalar eksik**  
**Yönetilen Motoru Yeniden Kur** seçeneğini kullanın veya eksiksiz uyumlu bir yerel motor klasörünü tekrar seçin.

**8x/12x kaynak yetersizliği nedeniyle çalışmıyor**  
Daha küçük bir giriş görseli veya daha düşük ölçek deneyin ve yeterli geçici disk alanı olduğundan emin olun.

Tekrarlanabilir uygulama hataları için [GitHub Issues](https://github.com/usainc/ClearCe/issues) kullanabilirsiniz. Güvenlik açıklarını public issue olarak değil, [private vulnerability reporting](https://github.com/usainc/ClearCe/security/advisories/new) üzerinden bildirin.
