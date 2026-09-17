# ClearCe 0.1.0 Release Candidate — Windows x64

ClearCe Windows MVP release candidate with the localized per-user NSIS wizard and Phase 4.5 managed-engine workflow. This remains an **UNSIGNED RELEASE CANDIDATE**; this phase does not upload or publish it.

Installer: `src-tauri/target/release/bundle/nsis/ClearCe_0.1.0_x64-setup.exe`.

## English

ClearCe processes images locally using a trusted Real-ESRGAN NCNN Vulkan engine. English and Turkish are available; a fresh installation uses the installer language, otherwise Windows language is the fallback. An existing saved choice always wins. Switch languages immediately in Settings.

The AI engine is not included in the installer. Auto recommends a model from the detected GPU/Vulkan profile. Open Models and choose **Download Recommended** to fetch the pinned official package, verify its SHA-256 and activate its general/anime models. Advanced users can still choose **Select Local AI Engine**. Managed and manual installs are separate; invalid attempts preserve the working setup.

Import an image, select 2x/4x/8x/12x and an output folder, then enhance. 8x/12x use multiple AI passes and require more resources. Existing output files are kept; EXIF metadata is removed. Batch processing and recorded-result comparison are available.

## Türkçe

ClearCe, güvenilir bir Real-ESRGAN NCNN Vulkan motoruyla görselleri bilgisayarınızda işler. İngilizce ve Türkçe desteklenir. Yeni kurulumda kurulum dili kullanılır; kayıtlı dil tercihiniz varsa korunur. Kurulum dili aktarılmamışsa Windows dili kullanılır. Dili Ayarlar üzerinden anında değiştirebilirsiniz.

AI motoru kurulum paketine dahil değildir. Auto, algılanan GPU/Vulkan profiline göre model önerir. Modeller ekranındaki **Önerileni İndir** düğmesi sabitlenmiş resmî paketi indirir, SHA-256 değerini doğrular ve genel/anime modellerini etkinleştirir. İleri düzey kullanıcılar **Yerel AI Motorunu Seç** seçeneğini kullanabilir. Yönetilen ve manuel kurulumlar ayrıdır; geçersiz denemeler çalışan kurulumu bozmaz.

Bir görsel açın, 2x/4x/8x/12x ölçeğini ve çıktı klasörünü seçip iyileştirmeyi başlatın. 8x/12x birden fazla AI geçişi kullanır ve daha fazla kaynak gerektirir. Var olan dosyaların üzerine yazılmaz; EXIF bilgileri kaldırılır. Toplu işlem ve gerçek sonuç karşılaştırması kullanılabilir.

## Requirements and limits

Windows x64, preinstalled WebView2 and a compatible Vulkan driver. No engine is bundled; internet is needed only for the optional managed-engine download after explicit user action. Image processing itself remains local and works offline after a healthy engine is installed. The installer is not digitally signed, so SmartScreen may warn. No auto updater, portrait/text/restoration specialist models, metadata preservation, overwrite, cloud service or video support. See [Phase 4.5](PHASE4_5.md), [Windows trust](WINDOWS_TRUST.md) and `MVP_FEATURE_STATUS.md` for precise validation and limitations.

Release integrity files are generated under `src-tauri/target/release/release-integrity/`. The published SHA-256 values must match the exact final executable and installer. Publish the matching manifest/hashes together with a reviewed installer on the official GitHub Releases channel; never label an unsigned artifact as signed.
