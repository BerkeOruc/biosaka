# BioSaka Hedefler — Day-by-Day Roadmap

Her gun 1 feature. Basit, odakli, teslim edilebilir.

---

## Day 1 — Interactive Neuron Stimulation (Poke the Worm)

Klavyeden belirli sensory noronlari uyar. Solucan canli canli tepki versin.

- [j] → ASEL (head, left)
- [k] → ASER (head, right)
- [u] → AWAL (olfaction, left)
- [o] → AWAR (olfaction, right)
- [p] → toggle periodic auto-stim on/off
- Header'da "Stimulated: ASEL" feedback'i
- Stimulasyon siddeti gosterimi

**Durum:** tamamlandi ✓
- [j] [k] [u] [o] → klavyeden noron uyarma
- [p] → auto-stim toggle
- Header'da sari "Stim: ASEL" feedback (15 frame sonra kaybolur)
- Help overlay'de stimulation bolumu
- Footer'da "[j]stim" gostergesi

---

## Day 2 — Simulation Speed Control

Hiz kontrolu. Yavaslat/hizlandir.

- `[` / `]` → simulation step multiplier (1x / 3x / 10x / 30x / 100x)
- Header'da hiz gostergesi (mor renkli `|3x`)
- Yuksek hizda worm body guncellemesini atla (GPU degilsin)

**Durum:** tamamlandi ✓
- `[` / `]` tuslari ile hiz degistirme
- Header'da `|3x` gostergesi
- Help overlay'de speed kontrol satiri eklendi

---

## Day 3 — Neurotransmitter Diversity

GABA, glutamate, acetylcholine farklari. Inhibitor noronlar gercekten inhibe etsin.

- Veriye norotransmitter tipi ekle (CSV'ye sutun veya noron adina gore inference)
- GABA sinapslari → postsinaptik potansiyeli DUSUR
- Glutamat → eksitasyon
- Asetilkolin → eksitasyon (motor)
- Devlog'da "artık gerçek inhibisyon var" anı

---

## Day 4 — Neuron Search

Graph uzerinde istedigin noronu bul.

**Durum:** tamamlandi ✓
- `/` tusu ile search modu aktiflesir
- Harf/girinti/tire karakterleri ile arama yapilir, buyuk/kucuk harf duyarsiz `contains` eslesme
- Backspace ile duzeltme, Esc ile iptal, Enter ile onayla
- Eslesen noronlar graph'ta **Magenta BOLD** olarak highlight edilir; eslesmeyenler gizlenir
- Header'da mor renkli ` Search: ASEL█ ` gostergesi ve `[5]` sonuc sayisi
- Graph title'inda `search: 5` sayisi
- Help overlay'e `[/]` satiri eklendi

---

## Day 5 — Simulation Parameter Tuning

Zar gecirgenligi, threshold, noise canli canli degissin.

- `[p]` ile parameter paneli toggle
- Leak constant (0.90 - 0.99)
- Threshold (0.5 - 2.0)
- Noise strength (0.0 - 0.1)
- Synaptic weight multiplier (0.5 - 3.0)
- Her parametre su anki degerini gostersin

---

## Day 6 — Network Metrics Panel

Stats tabina yeni metrikler.

**Durum:** tamamlandi ✓
- Stats panelinde `Sync idx` — es zamanli atesleme orani (aktif oranin karesi)
- `Rate dist` — L/M/H kategorilerinde noron sayisi (0-2%, 2-8%, >8%)
- `Hubs:` en cok baglantili 5 noron, isim ve baglanti sayisiyla
- `self.hubness` App'te onceden hesaplanmis veri kullaniliyor

---

## Day 7 — Force Layout Refinement + Edge Coloring

Grafik goruntuleme iyilestirmeleri.

- Chemical edges (type=0): sari/beyaz cizgi
- Gap junctions (type=1): mavi/yesil cizgi
- Neuron gruplarina gore renk: sensory=yesil, motor=kirmizi, inter=mavi
- Layout animasyonu (peanut → force arasi interpolasyon)
- Hub neuronlari daha buyuk goster

**Durum:** tamamlandi ✓
- Chemical edge'ler sari, gap junction'lar cyan — eski DarkGray/Gray'den cok daha okunabilir
- Force layout modunda inaktif noronlar grup bazli renk aliyor: motor=red, inter/sensory=DarkGray
- Hub noronlar (>20 baglanti) her zaman BOLD goruntuleniyor
- `hubness` ve `neuron_groups` App'de onceden hesaplaniyor

---

## Day 8 — Worm Physics v2

20 segment otesi.

- Her segmente bagimsiz kitle/atalet
- Segmentler arasi sertlik/sogutma
- Yer surtunmesi (sag sol asimetrisi ile donus)
- Engel olarak dikdortgenler, carpinCA sek sek
- `[e]` ile engel ekle (fare imleci olmadigi icin random)

---

## Day 9 — Record & Playback

Simulasyonu kaydet, geri sar.

- Ring buffer: son 10 saniye (30000 adim × 307 bit = ~1.2MB)
- `[r]` → record toggle
- `[<]` `[>]` → playback scroll
- Playback modunda simulasyon durur, kayit oynar
- Header'da REC/PLAY gostergesi

---

## Day 10 — Cook 2019 Male Connectome

Ikinci connectome. Erkek solucan.

- `data/connectome_male.csv` (Cook 2019'dan)
- `--sex herm|male` komut satiri argumani
- Farkli baglanti sayilari, farkli davranis
- Stats tabinda "Sex: hermaphrodite/male"

---

## Notlar

- Her gunun sonunda `cargo check` temiz olmali
- Her gun kendi branch'inde gelistirilebilir
- Oncelik sirasi: etki/efor orani en yuksek olan once
- Hedef: 10 gunde biosaka'yi bir ust seviyeye tasimak

---

*berke, 2026*
