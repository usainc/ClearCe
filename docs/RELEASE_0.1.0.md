# ClearCe 0.1.0 — Windows x64

ClearCe branding release, Phase 4.2; includes Phase 4.1 localization. Per-user NSIS installer; unsigned.

Installer: `src-tauri/target/release/bundle/nsis/ClearCe_0.1.0_x64-setup.exe`.

## English

ClearCe processes images locally using a trusted Real-ESRGAN NCNN Vulkan engine. English and Turkish are available; first launch follows Windows language and later respects your saved choice. Switch languages immediately in Settings.

The AI engine is not included. Open Models and choose **Select Local AI Engine**. Select an extracted trusted folder containing `realesrgan-ncnn-vulkan.exe` and `models/realesrgan-x4plus.param` / `.bin`. Verification checks files, launchability and Vulkan readiness. Invalid selections preserve an existing installation. If the engine disappears, select it again; your history and settings remain available.

Import an image, select 2x/4x/8x/12x and an output folder, then enhance. 8x/12x use multiple AI passes and require more resources. Existing output files are kept; EXIF metadata is removed. Batch processing and recorded-result comparison are available.

## Türkçe

ClearCe, güvenilir bir Real-ESRGAN NCNN Vulkan motoruyla görselleri bilgisayarınızda işler. İngilizce ve Türkçe desteklenir. İlk açılışta Windows dili kullanılır; sonraki açılışlarda seçiminiz korunur. Dili Ayarlar üzerinden anında değiştirebilirsiniz.

AI motoru kurulum paketine dahil değildir. Modeller ekranındaki **Yerel AI Motorunu Seç** düğmesiyle `realesrgan-ncnn-vulkan.exe` ve `models/realesrgan-x4plus.param` / `.bin` dosyalarını içeren güvenilir klasörü seçin. Dosyalar, motorun başlatılabilmesi ve Vulkan hazırlığı doğrulanır. Geçersiz seçim çalışan kurulumu bozmaz. Motor kaybolursa tekrar seçebilirsiniz; geçmişiniz ve ayarlarınız korunur.

Bir görsel açın, 2x/4x/8x/12x ölçeğini ve çıktı klasörünü seçip iyileştirmeyi başlatın. 8x/12x birden fazla AI geçişi kullanır ve daha fazla kaynak gerektirir. Var olan dosyaların üzerine yazılmaz; EXIF bilgileri kaldırılır. Toplu işlem ve gerçek sonuç karşılaştırması kullanılabilir.

## Requirements and limits

Windows x64, WebView2 and a compatible Vulkan driver. Engine redistribution permission has not been verified, so it is not bundled or automatically downloaded. Installer is not digitally signed. WebView2 installation can require internet when absent. No auto updater, specialist AI models, metadata preservation, overwrite, cloud service or video support. See `PHASE4_1.md` for validation and `MVP_FEATURE_STATUS.md` for exact feature truth.
