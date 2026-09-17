# ClearCe engine and model policy

Phase 4.5 keeps one controlled Windows runtime: Real-ESRGAN NCNN Vulkan 0.2.5.0. ClearCe can download the official `realesrgan-ncnn-vulkan-20220424-windows.zip` asset from the pinned upstream GitHub release. The exact URL and SHA-256 are compiled into the Rust catalog; arbitrary URLs and mirror discovery are rejected.

## Supported catalog

| ID | Category | State | Runtime use |
| --- | --- | --- | --- |
| `realesrgan-x4plus` | General photo | Available, managed/manual | Default general model; 2x/4x/8x/12x pipeline |
| `realesrgan-x4plus-anime` | Anime / illustration | Available in the managed package | Selected by Auto for Anime / Illustration |
| `realesr-general-x4v3` | Efficient general | Deferred | The approved Windows package does not contain compatible NCNN files |
| GFPGAN portrait | Portrait | Deferred | No compatible runner is implemented |

Portrait and Screenshot / Text do not claim a specialist model. Those processing modes remain unavailable until a real backend exists.

## Deterministic recommendation

ClearCe reads Vulkan physical-device properties. CPU devices are excluded. Dedicated memory is displayed and used only for discrete GPUs; shared integrated memory is not labelled VRAM.

| Detection | Tier | Profile |
| --- | --- | --- |
| No Vulkan device | Unsupported | No model recommendation; driver/GPU action shown |
| Integrated GPU, or discrete with 0–3 GiB | Low | Efficient recommendation with the supported general model |
| Discrete with 4–7 GiB, or discrete memory unavailable | Mid | Balanced |
| Discrete with 8–15 GiB | High | Quality |
| Discrete with 16 GiB or more | Ultra | Quality |

General Photo recommends `realesrgan-x4plus`. Anime / Illustration recommends `realesrgan-x4plus-anime`; when absent the UI offers the approved package. Auto uses a present managed package first and otherwise falls back to a valid manual/legacy engine. An invalid managed package produces a repair state instead of silently switching. Manual mode never switches to the managed engine.

## Managed install

Managed files live under `%LOCALAPPDATA%\dev.usaince.enhancece\engines\managed\realesrgan`. Manual imports live under `engines\manual\realesrgan`; the former `engines\realesrgan` location remains a read-only-compatible fallback.

The managed flow:

1. accepts only the exact catalog URL;
2. follows at most five redirects and only through GitHub/GitHub content hosts;
3. downloads into an owned temporary directory with size and free-space bounds;
4. verifies SHA-256 `abc02804e17982a3be33675e4d471e91ea374e65b70167abc09e31acb412802d`;
5. rejects absolute, parent-traversal, backslash and duplicate archive entries;
6. extracts only the executable, two supported model pairs and required runtime DLLs;
7. creates a hashed manifest and package-source metadata;
8. publishes to the stable managed directory, runs executable/Vulkan health checks, and rolls back on failure.

Temporary download data is removed with the owned temporary directory. A failed managed attempt does not modify the separate manual engine. No image is uploaded; network access occurs only after the user chooses the download action.

## Manual install

Advanced users can choose an extracted local engine directory. ClearCe copies only fixed expected files, writes an integrity manifest, validates model headers, runs the same bounded launch/Vulkan health checks, and publishes atomically. Manual and managed packages coexist.

All recommendation, status, guide, progress and recovery copy exists in English and Turkish. The catalog and recommendation decisions originate in Rust; the UI does not invent installed or supported models.
