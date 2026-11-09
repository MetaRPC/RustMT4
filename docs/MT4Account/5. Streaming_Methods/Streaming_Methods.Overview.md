# 5. Streaming Methods — Overview

> Real-time server-push streams: ticks, trade events, open-order ticket lists, per-order profit, and connection state.

Each method returns a raw `tonic::Streaming<…Reply>`; drop it to unsubscribe. Read frames with `stream.message().await`, matching the reply's `oneof { Data, Error }`.

```rust
while let Ok(Some(reply)) = stream.message().await {
    if let Some(pb::on_symbol_tick_reply::Response::Data(d)) = reply.response {
        // handle d
    }
}
```

---

## Methods in this group

| Method | Item | One-liner |
|--------|------|-----------|
| [`on_symbol_tick(req)`](./on_symbol_tick.md) | `OnSymbolTickReply` | Live ticks for the requested symbols. |
| [`on_trade(req)`](./on_trade.md) | `OnTradeReply` | Order/trade events (new / changed / closed). |
| [`on_opened_orders_tickets(req)`](./on_opened_orders_tickets.md) | `OnOpenedOrdersTicketsReply` | Open-order ticket list on a timer. |
| [`on_opened_orders_profit(req)`](./on_opened_orders_profit.md) | `OnOpenedOrdersProfitReply` | Account equity + per-order profit on a timer. |
| [`on_connect_state(req)`](./on_connect_state.md) | `OnConnectStateReply` | Terminal connection-state events. |

---

## When to use which

* **Live prices** → `on_symbol_tick` (vs polling `quote`).
* **React to fills/closes** → `on_trade`.
* **Live P/L dashboard** → `on_opened_orders_profit`.
* **Track what's open without re-fetching details** → `on_opened_orders_tickets`.

> `on_connect_state` is on the `Connection` service (not `SubscriptionService`); the rest are on `SubscriptionService`. `MT4Sugar` wraps the tick / profit / tickets streams as mpsc channels of friendly values.

---

**Back to:** [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
