<div align="center">

# 🛡️ ClipVault

### *Fast, Themed & Encrypted Clipboard Manager with Secure Vault for Linux*

[![Release](https://img.shields.io/badge/release-v0.1.0-emerald.svg)](https://github.com)
[![Platform](https://img.shields.io/badge/platform-Linux%20(Ubuntu%20%7C%20Debian%20%7C%20Arch)-blue.svg)](https://github.com)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Built With Tauri v2](https://img.shields.io/badge/built%20with-Tauri%20v2%20%2B%20Rust%20%2B%20Svelte-orange.svg)](https://tauri.app)

<br/>

**ClipVault**, Linux ortamında çalışan, sistem kaynaklarını minimum tüketen (~15-25 MB RAM), akıllı çift içerikli (metin & ekran görüntüsü) pano geçmişini **askeri düzeyde AES-256-GCM şifreli Güvenli Kasa** ile birleştiren yeni nesil bir masaüstü uygulamasıdır.

</div>

---

## ⚡ Neden ClipVault?

Geleneksel pano yöneticileri hassas şifrelerinizi, API anahtarlarınızı veya özel belgelerinizi düz metin (plain-text) olarak saklar ve sistemdeki tüm uygulamaların erişimine açık bırakır. 

**ClipVault**, günlük hız ihtiyacınız için ışık hızında bir pano geçmişi sunarken; parola, kripto anahtarları, kimlik fotokopileri veya özel PDF'leriniz için parolasız açılamayan, donanım/makine anahtarlı, bağımsız bir **Güvenli Kasa (Secure Vault)** sağlar.

---

## ✨ Temel Özellikler

### 📋 1. Akıllı Pano Yöneticisi (Clipboard Manager)
- **Çift İçerik Desteği (Metin & Görsel):** Kopyalanan tüm metinler, kod blokları ve panoya alınan ekran alıntıları/görseller anında yakalanır.
- **Canlı Renk & Veri Önizleyici (Color & URL Preview):**
  - Kopyalanan metin bir renk kodu (`#HEX`, `rgb(...)`, `hsl(...)`) ise kart üzerinde canlı renk kutucuğu gösterilir.
  - Web bağlantıları algılanır ve tek tıkla doğrudan varsayılan tarayıcınızda açma kısayolu sunulur.
- **Akıllı Zirveye Taşıma (Bump to Top):** Bir öğeyi kopyaladığınızda veya listeden seçtiğinizde otomatik olarak listenin en tepesine taşınır; sık kullandığınız öğeler daima elinizin altında kalır.
- **Kategori Filtreleri & Canlı Arama:**
  - `Tümü`, `Metinler`, `Görseller`, `Sabitlenenler` sekmeleri.
  - Yazdığınız anda filtreleyen anlık arama motoru.
- **Pano Geçmişini Dışa Aktarma:** Geçmişinizi dilediğiniz zaman Markdown (`.md`) veya düz metin (`.txt`) olarak dışa aktarabilirsiniz.

### 🔐 2. Şifreli Güvenli Kasa (Secure Vault)
- **Askeri Düzeyde Şifreleme:** Verileriniz **AES-256-GCM** ve **PBKDF2-HMAC-SHA256** (100.000 iterasyon) ile şifrelenir.
- **Kişiselleştirilebilir Güvenlik Seviyeleri:**
  - **PIN Koruması:** Kullanıcı tarafından belirlenen parola/PIN ile kilitleme.
  - **Cihaz Koruması (Device-Bound Key):** Parola sormadan yalnızca o donanıma özel türetilen 256-bit anahtarla şeffaf şifreleme.
- **Güvenli Dosya & Belge Kasası:**
  - PDF, Office belgeleri, görseller ve arşivleri şifreli kasada saklayın.
  - Sistem panosuna dosya kopyalama (`file://` URI protokolü ile GTK clipboard entegrasyonu). Dosyalar geçici klasörlerde iz bırakmadan doğrudan güvenli kasadan servis edilir.
- **Kasa Tam Yedekleme & Geri Yükleme (Backup & Restore):**
  - Kasanızdaki tüm not, parola, ayar ve dosyaları tek bir şifreli yedek paketi (`.backup` / `.zip`) olarak dışa aktarın ve başka bir makinede geri yükleyin.
- **Özelleştirilebilir Sekmeler:** Bilgilerinizi `Kişisel`, `Siteler`, `Yazılım`, `Eğitim` veya kendi oluşturacağınız özel sekmelerde düzenleyin.

### 🎨 3. Tasarım ve Kullanıcı Deneyimi (Ergonomi & Temalar)
- **Dinamik Renk Temaları:** 
  - `Zümrüt Yeşili (Varsayılan)`, `Okyanus Mavisi`, `Gece Yarısı Koyu`, `Pastel Mor`, `Gün Batımı Turuncusu`, `Minimal Gri`.
  - Seçilen tema hem Pano hem Güvenli Kasa hem de tüm modallarda kusursuz bir şekilde senkronize çalışır.
- **Raycast / Spotlight Hissiyatı:** Çerçevesiz, yumuşak gölgeli, modern ve göz yormayan yüzen pencere.
- **Özel Taşıma Kolu (Window Drag Handle):** Pencereyi ekranın dilediğiniz noktasına kolayca sürükleyin; pozisyon ve boyut hafızası korunur.
- **Sistem Başlangıcında Otomatik Başlatma:** Ayarlar ekranından tek tıkla `~/.config/autostart` entegrasyonu.

---

## ⌨️ Klavye Kısayolları

| Kısayol | Açıklama |
|---|---|
| `Alt + Shift + Q` | ClipVault penceresini aç / kapat (Ayarlardan değiştirilebilir) |
| `↑` / `↓` | Listede önceki / sonraki öğeye odaklan |
| `Enter` | Seçili öğeyi kopyala, en üste taşı ve pencereyi gizle |
| `1` - `9` | Listedeki ilk 9 öğeyi tek tuşla anında kopyala |
| `P` | Seçili öğeyi en başa sabitle (Pin) / sabitlemeyi kaldır |
| `Delete` | Seçili öğeyi sil (onay kutusu ile) |
| `Alt + ↑` / `Alt + ↓` | Seçili öğeyi listede manuel yukarı / aşağı kaydır |
| `Esc` | Açık modalı kapat veya ClipVault penceresini gizle |

---

## 📥 Kurulum (Installation)

### Debian / Ubuntu / Linux Mint (.deb ile Kurulum)

Üretilen `.deb` paketini tek komutla kurabilirsiniz:

```bash
# Paketi kurun
sudo dpkg -i clipvault_0.1.0_amd64.deb

# Eksik bağımlılık olursa onarın
sudo apt-get install -f
```

Kurulum tamamlandıktan sonra uygulama menünüzde **ClipVault** simgesi belirecek veya terminalden `clipvault` komutu ile başlatabilirsiniz.

---

## 🛠️ Kaynak Koddan Derleme (Building from Source)

### Ön Gereksinimler
- **Node.js** (v18+ veya v20+)
- **Rust & Cargo** (v1.75+)
- Linux Geliştirme Kütüphaneleri:
  ```bash
  sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf
  ```

### Geliştirici Modunda Çalıştırma
```bash
# Depoyu klonlayın
git clone https://github.com/username/clipvault.git
cd clipvault

# Bağımlılıkları yükleyin
npm install

# Geliştirici modunu başlatın
npm run tauri dev
```

### Üretim Paketi Oluşturma (Production Build)
```bash
# Release derlemesini ve .deb paketini üretir
npm run tauri build
```
Oluşturulan paket `src-tauri/target/release/bundle/deb/clipvault_0.1.0_amd64.deb` konumunda hazır olacaktır.

---

## 🔒 Güvenlik & Gizlilik Mimarisi

1. **Sıfır Bulut / Tamamen Yerel (Zero-Cloud):** ClipVault hiçbir verinizi internete aktarmaz. Tüm veritabanı ve önbellek `~/.local/share/clipvault/` dizininde saklanır.
2. **Kriptografik Standartlar:**
   - **Şifreleme Algoritması:** AES-256-GCM (Kimlik doğrulamalı şifreleme - AEAD).
   - **Anahtar Türetme (KDF):** PBKDF2-HMAC-SHA256, 100.000 döngü ve rastgele 32-bayt tuz (salt).
   - **Dosya İzolasyonu:** Kasaya eklenen hassas dosyalar `0700` dosya izinleriyle kullanıcıya özel izole alanda saklanır.

---

## 📄 Lisans

Bu proje [MIT Lisansı](LICENSE) altında lisanslanmıştır. Dilediğiniz gibi geliştirebilir, fork edebilir ve katkıda bulunabilirsiniz.

