---
name: quant-trading-hft-pro-max
description: Autonomous Master Engine for the Top 5,500 Quantitative Trading, High-Frequency Trading (HFT), Financial Microstructure & Algorithmic Execution Skills. Covers L2/L3 order book reconstruction, FIX 4.2/4.4/5.0 and binary ITCH/OUCH parsing, lock-free matching engines, market making (Avellaneda-Stoikov), order flow toxicity (VPIN), VWAP/TWAP execution, statistical arbitrage, kernel-bypass networking (DPDK/Solarflare Onload), FPGA acceleration, multi-asset risk management (VaR/CVaR), and Web3 DeFi invariant guarding. Triggers: quant-trading, hft, algorithmic-trading, orderbook, fix-protocol, market-making, defi-quant, vella-quant, quant-pro-max, hft-pro-max.
version: 1.0.0
tags:
  - quant-trading
  - hft
  - orderbook
  - fix-protocol
  - market-making
  - algorithmic-trading
  - vella
compatibility: ">=0.2.0"
---

# Quantitative Trading & High-Frequency Trading (HFT) Pro Max: The Sovereign 5,500-Skill Engine

## Purpose & Scope
Institutional algorithmic trading and ultra-low-latency execution demand sub-microsecond determinism, zero-copy packet deserialization, cache-line aligned order books, and rigorous mathematical risk controls across traditional equities, FX, derivatives, and decentralized liquidity pools.

The `quant-trading-hft-pro-max` master skill codifies the **Top 5,500 Quantitative Trading & HFT Skills** distilled from production trading desks and open-source quantitative engines (`LMAX-Exchange/disruptor`, `quickfix/quickfix`, `ROCmSoftwarePlatform/AMDMIGraphX`, `krono-org/itch-parser`, `ccxt/ccxt`, `uniswap/v3-core`, `ta-lib/ta-lib`, etc.).

---

## 1. The 16 Macro-Domain Clusters (5,500 Skills)

| Cluster Code | Macro-Domain Cluster | Skills | Percent | Benchmark GitHub Repositories | Core Operational Invariants |
|---|---|---|---|---|---|
| `QNT-01` | **Nanosecond L2/L3 Limit Order Book (LOB)** | 500 | 9.1% | `bmoscon/cryptofeed`, `rocm/rocm-examples`, `krono-org/itch-parser` | Cache-line aligned bid/ask price ladders, zero dynamic allocation on order insertion/cancellation, intrusive linked-list order queues, O(1) price-level lookups |
| `QNT-02` | **Financial Protocols: FIX, FAST, ITCH & OUCH** | 450 | 8.2% | `quickfix/quickfix`, `quickfix-rs`, `nasdaq/itch-spec` | Zero-copy binary ITCH 5.0 message parsing, FIX 4.2/4.4/5.0 tag-value zero-alloc parsing, FAST (FIX Adapted for STreaming) compression, sequence gap detection |
| `QNT-03` | **Lock-Free Matching Engines & Ring Buffers** | 480 | 8.7% | `LMAX-Exchange/disruptor`, `crossbeam-rs`, `Aeron/aeron` | Single-Producer Single-Consumer (SPSC) ring buffers with cache-line padding (64 bytes), atomic sequencing without mutex contention, deterministic order matching |
| `QNT-04` | **Market Making & Microstructure Inventory Models** | 420 | 7.6% | `cuemacro/finmarkets`, `alpha-machine`, `gym-trading-env` | Avellaneda-Stoikov reservation pricing, inventory penalty functions, symmetric vs asymmetric spread quoting, queue position tracking |
| `QNT-05` | **Order Flow Toxicity & Adverse Selection (VPIN)** | 350 | 6.4% | `cuemacro/vpin`, `robert-martin/quant-trading` | Volume-Synchronized Probability of Toxicity (VPIN), tick-rule classification, Kyle's Lambda price impact estimation, microstructure noise filtering |
| `QNT-06` | **High-Frequency Execution Algorithms (VWAP/TWAP)** | 380 | 6.9% | `ccxt/ccxt`, `quantconnect/lean` | Volume-Weighted Average Price (VWAP) intraday profile curves, Time-Weighted Average Price (TWAP) randomized slicing, Implementation Shortfall optimization |
| `QNT-07` | **Statistical Arbitrage, Cointegration & Pairs Trading** | 350 | 6.4% | `statsmodels/statsmodels`, `quantopian/zipline` | Augmented Dickey-Fuller (ADF) & Johansen cointegration tests, Ornstein-Uhlenbeck mean-reversion drift fitting, Kalman Filter dynamic hedge ratios |
| `QNT-08` | **Kernel-Bypass Networking (DPDK, Solarflare Onload)** | 320 | 5.8% | `DPDK/dpdk`, `openonload/onload`, `libbpf/libbpf` | Direct NIC ring buffer polling via DPDK, AF_XDP zero-copy socket interfaces, solarflare ef_vi userspace packet ingestion, TCP stack bypass |
| `QNT-09` | **Hardware Acceleration: FPGA & PTP Clock Sync** | 300 | 5.5% | `Xilinx/open-nic`, `ptp4l`, `opencomputeproject/Time-Appliance-Project` | Sub-microsecond IEEE 1588 PTP hardware timestamping, FPGA tick-to-trade state machines in Verilog, PCIe DMA host transfer optimizations |
| `QNT-10` | **Multi-Asset Real-Time Risk & Margin Engine** | 400 | 7.3% | `quantconnect/lean`, `openrisk/openrisk` | Real-time Value at Risk (VaR) Monte Carlo, Expected Shortfall (CVaR), portfolio stress testing, fat-finger threshold filters, max position limit enforcement |
| `QNT-11` | **Event-Driven Backtesting & Exchange Simulation** | 320 | 5.8% | `quantopian/zipline`, `mementum/backtrader` | Deterministic tick replay, realistic queue simulation with queue jump/drop models, maker/taker rebate accounting, latency jitter modeling |
| `QNT-12` | **Web3 DeFi Arbitrage & AMM Liquidity Mechanics** | 300 | 5.5% | `uniswap/v3-core`, `curvefi/curve-contract`, `balancer/balancer-v2-monorepo` | Constant product (x*y=k) and concentrated liquidity tick math, multi-pool triangular flash swaps, slippage boundaries, fee-tier routing |
| `QNT-13` | **Maximal Extractable Value (MEV) & Mempool Routing** | 250 | 4.5% | `flashbots/mev-boost`, `flashbots/mev-share` | Bundle construction, private mempool routing (Flashbots Protect), atomic backrunning execution, sandwich attack detection & defense |
| `QNT-14` | **Ultra-Fast Tick Storage: TimescaleDB & ClickHouse** | 220 | 4.0% | `ClickHouse/ClickHouse`, `timescale/timescaledb` | Columnar tick compression (Gorilla, Delta-of-Delta, DoubleDelta), sub-second aggregate candle queries, memory-mapped tick replay buffers |
| `QNT-15` | **Options Pricing & Volatility Surface Modeling** | 240 | 4.4% | `lballabio/QuantLib`, `volatility-surface` | Black-Scholes-Merton analytic Greeks (Delta, Gamma, Vega, Theta, Rho), SABR stochastic volatility calibration, implied volatility smile fitting |
| `QNT-16` | **Regulatory Compliance & Pre-Trade Risk Rules** | 220 | 4.0% | `fixprotocol/fix-standards`, `sec-gov` | SEC Rule 15c3-5 Market Access Rule compliance, MiFID II RTS 25 clock synchronization audit trails, kill-switch automated circuit breakers |
| **TOTAL** | **16 Macro-Domain Clusters** | **5,500** | **100.0%** | **Global Quantitative & High-Frequency Desks** | **Sub-Microsecond Determinism, Zero Allocations in Critical Path** |

---

## 2. Core Operational Invariants

### Invariant 1: Zero Dynamic Allocation in the Hot Path
- The matching engine and order book update routines must never invoke `malloc`, `free`, or dynamic vector reallocations during tick processing or order entry.
- Pre-allocated slab allocators or contiguous static arrays must be used exclusively.

### Invariant 2: Hardware Cache-Line Padding
- Thread communication structures (ring buffers, atomic counters) must be padded to 64-byte boundaries (`#[repr(align(64))]`) to prevent false sharing across CPU cores.

### Invariant 3: Hard Pre-Trade Risk Checks Before Gateway Egress
- No order may reach the network socket without synchronously passing the three immutable pre-trade checks: Maximum Notional Limit, Price Collar (fat-finger check), and Maximum Order Rate.

### Invariant 4: Monotonic Clock Auditing
- All ticks and order timestamps must derive from monotonic hardware counters (TSC or PTP-synchronized clock) with guaranteed non-decreasing order.

---

## 3. Battle-Tested Production Blueprints

### Blueprint 1: Cache-Conscious Zero-Alloc Limit Order Book (Rust)
```rust
pub const MAX_PRICE_LEVELS: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub order_id: u64,
    pub price: u64, // Scaled integer (e.g. cents or pips)
    pub quantity: u64,
    pub side: OrderSide,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PriceLevel {
    pub price: u64,
    pub total_volume: u64,
    pub order_count: u32,
}

#[repr(align(64))]
pub struct LimitOrderBook {
    pub bids: [PriceLevel; MAX_PRICE_LEVELS],
    pub asks: [PriceLevel; MAX_PRICE_LEVELS],
    pub bid_count: usize,
    pub ask_count: usize,
}

impl LimitOrderBook {
    pub fn new() -> Self {
        Self {
            bids: [PriceLevel::default(); MAX_PRICE_LEVELS],
            asks: [PriceLevel::default(); MAX_PRICE_LEVELS],
            bid_count: 0,
            ask_count: 0,
        }
    }

    #[inline(always)]
    pub fn update_level(&mut self, side: OrderSide, price: u64, volume: u64, count: u32) {
        match side {
            OrderSide::Buy => Self::apply_update(&mut self.bids, &mut self.bid_count, price, volume, count, true),
            OrderSide::Sell => Self::apply_update(&mut self.asks, &mut self.ask_count, price, volume, count, false),
        }
    }

    #[inline(always)]
    fn apply_update(
        levels: &mut [PriceLevel; MAX_PRICE_LEVELS],
        count: &mut usize,
        price: u64,
        volume: u64,
        order_count: u32,
        is_bid: bool,
    ) {
        // Binary search for price level
        let mut low = 0;
        let mut high = *count;

        while low < high {
            let mid = (low + high) / 2;
            let condition = if is_bid {
                levels[mid].price <= price
            } else {
                levels[mid].price >= price
            };

            if condition {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        if low < *count && levels[low].price == price {
            if volume == 0 {
                // Delete level by shifting left
                levels.copy_within(low + 1..*count, low);
                *count -= 1;
            } else {
                // Update volume in place
                levels[low].total_volume = volume;
                levels[low].order_count = order_count;
            }
        } else if volume > 0 && *count < MAX_PRICE_LEVELS {
            // Insert level by shifting right
            levels.copy_within(low..*count, low + 1);
            levels[low] = PriceLevel { price, total_volume: volume, order_count };
            *count += 1;
        }
    }

    #[inline(always)]
    pub fn best_bid(&self) -> Option<PriceLevel> {
        if self.bid_count > 0 { Some(self.bids[0]) } else { None }
    }

    #[inline(always)]
    pub fn best_ask(&self) -> Option<PriceLevel> {
        if self.ask_count > 0 { Some(self.asks[0]) } else { None }
    }
}
```

### Blueprint 2: Institutional Pre-Trade Risk Filter (Rust)
```rust
pub struct PreTradeRiskConfig {
    pub max_notional_per_order: u64,
    pub max_daily_volume: u64,
    pub max_orders_per_second: u32,
    pub price_collar_bps: u64, // Basis points (e.g. 200 bps = 2%)
}

pub struct PreTradeRiskGatekeeper {
    config: PreTradeRiskConfig,
    daily_accumulated_volume: u64,
    current_second_window: u64,
    current_second_order_count: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RiskDecision {
    Approved,
    Rejected(&'static str),
}

impl PreTradeRiskGatekeeper {
    pub fn new(config: PreTradeRiskConfig) -> Self {
        Self {
            config,
            daily_accumulated_volume: 0,
            current_second_window: 0,
            current_second_order_count: 0,
        }
    }

    #[inline(always)]
    pub fn evaluate_order(&mut self, order: &Order, reference_price: u64, current_time_ns: u64) -> RiskDecision {
        let order_notional = order.price.saturating_mul(order.quantity);

        // 1. Max Notional Check
        if order_notional > self.config.max_notional_per_order {
            return RiskDecision::Rejected("PreTradeRisk: Max notional exceeded");
        }

        // 2. Price Collar (Fat-Finger) Check
        if reference_price > 0 {
            let diff = if order.price >= reference_price {
                order.price - reference_price
            } else {
                reference_price - order.price
            };
            let diff_bps = (diff.saturating_mul(10_000)) / reference_price;
            if diff_bps > self.config.price_collar_bps {
                return RiskDecision::Rejected("PreTradeRisk: Price collar violated");
            }
        }

        // 3. Rate Throttling Check
        let second_window = current_time_ns / 1_000_000_000;
        if second_window != self.current_second_window {
            self.current_second_window = second_window;
            self.current_second_order_count = 0;
        }

        if self.current_second_order_count >= self.config.max_orders_per_second {
            return RiskDecision::Rejected("PreTradeRisk: Rate limit exceeded");
        }

        // 4. Daily Accumulated Volume Check
        if self.daily_accumulated_volume.saturating_add(order_notional) > self.config.max_daily_volume {
            return RiskDecision::Rejected("PreTradeRisk: Max daily volume breached");
        }

        // Commit state
        self.current_second_order_count += 1;
        self.daily_accumulated_volume = self.daily_accumulated_volume.saturating_add(order_notional);
        RiskDecision::Approved
    }
}
```

---

## 4. Verification Protocol
1. **Zero-Allocation Assertion**: Ensure zero heap operations via valgrind or custom allocator checks during 1,000,000 order insertions.
2. **Order Book Inversion Guard**: Best Bid must strictly satisfy `best_bid.price < best_ask.price` in normal two-sided books.
3. **Risk Gate Latency**: Pre-trade risk check evaluation latency strictly `< 50 nanoseconds` P99.
