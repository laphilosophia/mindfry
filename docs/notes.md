# Consciousness Model - Tartışma Notları

> Geçici belge: Teorik tartışmadan çıkan kararlar ve feedback analizi
> Tarih: 2026-01-14

---

## 1. Veri Yapısı Analizi

### Problem

Klasik veritabanı yapıları (RDBMS, KV, GraphDB), "durumu" statik bir veri olarak saklar. Atrion Consciousness modelinde ise veri, sürekli enerji kaybeden ve form değiştiren dinamik bir varlıktır.

### Çözüm: In-Memory Phase Graph

Geleneksel bir Graph DB değil, vektör tabanlı bir durum makinesi.

| Bileşen       | Teknoloji / Yaklaşım        | Görev                                        |
| ------------- | --------------------------- | -------------------------------------------- |
| **Nodes**     | In-Memory Structs           | Vektör alanı noktaları (`E`, `ECC`, `Decay`) |
| **Edges**     | Weighted Links              | Enerji kanalları                             |
| **Index**     | HNSW (Custom Metric)        | Faz uzayı yakınlık araması                   |
| **Evolution** | Lazy Evaluation (On-Access) | CPU optimizasyonu                            |

### Edge Model Detayı

Weight ve ActivationCost aynı şey değil:

| Özellik        | Tip       | Kullanım                         |
| -------------- | --------- | -------------------------------- |
| Weight         | float     | Bağlantı gücü (synapse strength) |
| ActivationCost | float     | Traversal maliyeti               |
| Direction      | enum      | unidirectional / bidirectional   |
| LastActivation | timestamp | Decay hesabı                     |
| DecayRate      | float     | Bu edge ne hızda zayıflıyor      |

- **Weight:** "Bu bağlantı ne kadar güçlü?"
- **ActivationCost:** "Bu bağlantıyı kullanmak ne kadar pahalı?"

Yüksek weight, düşük cost olabilir (güçlü ve ucuz bağlantı).

### HNSW Metric Düzeltmesi

```typescript
// Vanilla HNSW (yanlış)
const distance = euclidean(vectorA, vectorB)

// Consciousness Model (doğru)
const distance = computeReachCost(nodeA, nodeB, graph)

function computeReachCost(a: Node, b: Node, graph: PhaseGraph): number {
  // Faz uzayı mesafesi + edge traversal maliyeti
  const phaseDist = weightedEuclidean(a.stateVector, b.stateVector, PHASE_WEIGHTS)
  const pathCost = graph.shortestPathCost(a.id, b.id)

  return phaseDist + pathCost
}
```

### Lazy Evaluation Detayı

- Çoğu node seyrek erişilir
- Erişilmeyen node'un "güncel olması" anlamsız
- Erişim anında `dt` hesaplaması deterministik
- **Kritik:** Cold storage'daki node'lar için `lastAccess` timestamp'ı persist edilmeli

### Persist Stratejisi

```
┌─────────────────────────────────────────────────────────┐
│  RAM (Phase Graph)          │  Disk (Cold Storage)      │
│  ├─ Hot nodes               │  ├─ Payload snapshots     │
│  ├─ Active edges            │  ├─ Identity registry     │
│  ├─ StateVectors            │  └─ Reincarnation hints   │
│  └─ Real-time evolution     │                           │
└─────────────────────────────────────────────────────────┘

Persist edilenler:
- Payload (bilgi içeriği)
- Node identity (reincarnation için)
- Son bilinen StateVector (hint olarak)
- lastAccess timestamp

Persist EDİLMEYENLER:
- Anlık faz konumu (her an hesaplanır)
- Edge activation costs (dinamik)
- Cluster üyeliği (derived)
```

---

## 2. v3 ↔ v4 İletişim Protokolü

### Problem

Neokorteks (v4), Bilinç (v3) üzerinde ağır sorgular çalıştıramaz. Heisenberg ilkesi gereği, bu gözlem sistemi değiştirir ve bütçeyi tüketir.

### Çözüm: Push-Based Projection Model

v3, belirli koşullarda v4'e bir Projection (Durum Özeti) fırlatır.

```
┌─────────────────────────────────────────────────────────┐
│  v3 (Consciousness)         │  v4 (Neocortex)           │
│                              │                           │
│  [Phase Graph]───────────────► [Projection Buffer]      │
│                              │                           │
│  Push koşulları:             │  Neocortex uyanır:        │
│  ├─ Limbik (v2) alarm        │  ├─ Projection geldiğinde │
│  ├─ ClusterEntropy < crit    │  ├─ İdle/Dream modunda    │
│  └─ Anomaly detected         │  └─ Explicit trigger      │
│                              │                           │
│  Pull YASAK                  │  Doğrudan tarama YOK      │
└─────────────────────────────────────────────────────────┘
```

### Projection Interface

```typescript
interface Projection {
  // Tetikleyici sebep
  trigger: 'limbic_alarm' | 'entropy_critical' | 'anomaly' | 'idle_sync'
  timestamp: number

  // Neokorteks için "State of the Union" özeti
  summary: {
    hotNodeCount: number // Aktif bilgi sayısı
    systemPressure: number // 0.0 - 1.0 arası genel stres
    criticalClusters: string[] // Riskli bölgelerin ID'leri
  }

  // Neokorteks'in derinlemesine bakması için ipuçları
  drillDownHints: DrillDownHint[]

  // Eklenen alanlar
  cost: number // Projection'ın maliyeti (v3'ün harcadığı bütçe)
  freshnessMs: number // Bu projection'ın "tazeliği"
}
```

### Gözlem Maliyeti Dağılımı

| Eylem                   | Bütçe Kaynağı | Maliyet                    |
| ----------------------- | ------------- | -------------------------- |
| v3 → Projection üretimi | v3 internal   | Düşük (self-observation)   |
| v4 → Projection okuma   | Sıfır         | Projection pasif buffer'da |
| v4 → DrillDown isteği   | v4 budget     | Yüksek (explicit request)  |

---

## 3. Dream Mode (Rüya Modu)

### Amaç

Sistem yükü (`systemPressure`) belirli bir eşiğin altına düştüğünde, Neokorteks proaktif bakım moduna geçer. Bu biyolojik REM uykusuna benzer.

### Görevler

- Gün içinde "Lazy Evaluation" nedeniyle güncellenmemiş node'ları tarar
- Ölü bilgileri temizler (Pruning)
- Kısa süreli hafızayı uzun süreli hafızaya (Cold Storage) transfer eder

### Atomic Task Garantisi

"Trafik artarsa anında kesilir" tehlikeli. Kesme sırasında yarım kalan işlem node'u corrupt edebilir.

**Çözüm:** Dream Mode işlemleri atomic unit'ler halinde olmalı:

```typescript
interface DreamTask {
  type: 'prune' | 'transfer' | 'consolidate'
  target: NodeId

  // Bu task bölünemez - ya tamamlanır ya da hiç başlamamış gibi
  atomic: true

  // Tahmini süre (kesme kararı için)
  estimatedMs: number
}

class DreamScheduler {
  private queue: DreamTask[] = []
  private currentTask: DreamTask | null = null

  interrupt(): void {
    // Mevcut task'ı tamamla, kuyruğu dondur
    this.queue = []
    // currentTask doğal olarak bitecek
  }

  tick(): void {
    if (systemPressure > DREAM_THRESHOLD) {
      this.interrupt()
      return
    }

    if (!this.currentTask && this.queue.length > 0) {
      this.currentTask = this.queue.shift()!
      this.execute(this.currentTask)
    }
  }
}
```

---

## 4. Neocortex Uyku Döngüsü

```
                    ┌──────────────┐
                    │  DORMANT     │ (Normal operasyon)
                    └──────┬───────┘
                           │ Projection geldi
                           ▼
                    ┌──────────────┐
                    │  ANALYSIS    │ (Strateji belirleme)
                    └──────┬───────┘
                           │ Karar verildi
                           ▼
                    ┌──────────────┐
                    │  DISPATCH    │ (Komut gönder)
                    └──────┬───────┘
                           │ Komut tamamlandı
                           ▼
                    ┌──────────────┐
                    │  DORMANT     │ (Geri uyku)
                    └──────────────┘
```

---

## 5. Feedback Değerlendirme Özeti

| Konu                 | Feedback     | Değerlendirme                      |
| -------------------- | ------------ | ---------------------------------- |
| Phase Graph yapısı   | ✅ Doğru     | Tek bileşen olarak iyi tanımlanmış |
| Lazy Evaluation      | ✅ Mükemmel  | CPU optimizasyonu için kritik      |
| HNSW                 | ⚠️ Kısmen    | Custom metric gerekli              |
| Projection Interface | ✅ İyi temel | cost/freshness eklenebilir         |
| Dream Mode           | ⚠️ Dikkat    | Atomic task garantisi gerekli      |
| Edge model           | ⚠️ Eksik     | Weight ≠ ActivationCost ayrımı     |

---

## 6. Açık Sorular

### v3-v4 İletişimi

> Neokorteks (v4), Bilinç (v3) üzerindeki veriyi okurken "Gözlem Bütçesi"ni (Observation Budget) nasıl tüketmeyecek?

**Yanıt:** Projection bazlı push model. v4 doğrudan taramaz, v3 özet gönderir.

### Veri Yapısı

> Bu modelin veri yapısı ne olacak?

**Yanıt:** In-Memory Phase Graph - vektör tabanlı durum makinesi, HNSW indeksli, lazy evaluation ile.

---

## 7. Belgeye Eklenecek Bölümler

`consciousness.md` için önerilen yeni bölümler:

```markdown
## 16. Infrastructure Perspective

### 16.1 The Phase Graph

[Veri yapısı tanımı]

### 16.2 Edge Model

[Weight vs ActivationCost ayrımı]

### 16.3 Lazy Evolution

[On-access hesaplama + timestamp persistence]

## 17. Inter-Layer Communication

### 17.1 Projection Protocol

[Interface + cost/freshness]

### 17.2 Dream Mode

[Atomic task garantisi ile]
```
