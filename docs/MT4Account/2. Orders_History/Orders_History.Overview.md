# 2. Orders & History — Overview

> What's open now (market **and** pending orders, in one list), their ticket numbers, and the closed-order history.

In MT4 there are no separate "positions": a filled market order **is** the position, and pending orders sit in the same opened-orders list (distinguished by `order_type`). These methods are **read-only** and live on `mt4_term_api.AccountHelper`.

---

## Methods in this group

| Method | Returns | One-liner |
|--------|---------|-----------|
| [`opened_orders(req)`](./opened_orders.md) | `OpenedOrdersData` | All open market + pending orders, with full detail. |
| [`opened_orders_tickets()`](./opened_orders_tickets.md) | `OpenedOrdersTicketsData` | Just the ticket numbers (lightweight). |
| [`orders_history(req)`](./orders_history.md) | `OrdersHistoryData` | Closed / cancelled orders over a time window (paged). |

---

## When to use which

* **"Show me everything open, with prices and P/L"** → `opened_orders()` → `order_infos`.
* **"I just need the tickets to iterate"** → `opened_orders_tickets()`.
* **"What happened last week?"** → `orders_history()` (time window + sort + paging).

> Market orders vs pendings: filter `order_infos` by `order_type` — `0` Buy / `1` Sell are the market "positions"; `2..5` are the pending Limit/Stop types.

---

**Back to:** [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
