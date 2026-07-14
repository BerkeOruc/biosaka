# devlog v0.2 — the worm woke up

selam millet, berke here. biosaka v0.2 cikti. terminaldeki solucan biraz daha bilinclendi.

## bu surumde ne var

**beyin loblara ayrildi.** 307 noron eskiden cemberin etrafinda diziliyodu, simdi iki yarim kure seklinde. sag hemisfer, sol hemisfer, ortada corpus callosum gibi daralma. gozle gorulur sekilde beyne benziyor artik.

**noronlar geri geldi.** ilk surumdeki gibi nokta karakterleri (`●`) kullaniyoruz. atesleyen noronlar `◉` fisheye oluyo, sari yaniyo. inaktifler gri nokta. renk gecisi yellow > green > cyan > blue > gray, 5 seviye. aktif noronler bold gorunuyo.

**gruplar eklendi.** sag panelde sensory/motor/interneuron aktivitesi canli canli gosteriliyor. her grubun kac noronu aktif, ortalama firing rate'i ne, bar olarak gosteriliyor.

```
 SEN 30/55  55.8% ██████
 MOT 57/68  76.6% ████████
 INT 109/184 58.1% ██████
```

**worm view duzeldi.** govde gradientli, bas `█` kirmizi, govde `●` acik kirmizi, kuyruk `◐` gri. yanda motor aktivite barlari, speed, segment bilgisi.

**o panic gitti.** terminal kucuk olunca cizim tasiyordu, buffer overflow atiyordu. simdi gercek terminal boyutuna gore sinirlaniyor. hic panic yok.

**AUR'da.** `yay -S biosaka` kurulum yapabilirsin. arch kullanicilari hic ugrasmiyor.

## roadmap / dusunceler

sirada ne var diye dusunuyorum:

- **norotransmitter cesitliligi.** simdi her sinaps ayni agirlikta. gaba, glutamat, asetilkolin farklari girince davranis da degisecek. inhibitor noronlar gercekten inhibe edecek.
- **force-directed layout.** noronlari elle yerlestirmek yerine gercek baglantilarina gore fizik simulasyonuyla dizmek. aktif bolgeler birbirine yakin duracak.
- **neuron label.** aktif noronlarin isimlerini yaninda gostermek. ozellikle "o hangi noron atesleyince solucan saga donuyo" merak edenler icin.
- **gercek solucan fizigi.** 20 segment biraz yetersiz. muscle physics, toprakta hareket, engel algilama.
- **drosophila.** 100k noron. simdiki 307'den sonraki adim. terminalde beyin patlamasi yasamak istiyosan...
- **mutasyon.** baglantilari rastgele degistir, farkli solucanlar gor. evrimi terminalde izle.
- **record/playback.** simulation'u kaydet, sonra izle. "bu spike nereden geldi" diye geri sar.

uzun lafin kisasi: solucan emekliyor, beyin yaniyor, AUR'da. daha iyi olacak.

wait ready for my arrival worm 🪱

— berke, 2026
