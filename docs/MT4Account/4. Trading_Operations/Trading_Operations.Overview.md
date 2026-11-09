# 4. Trading Operations — Overview

> The write path: send, modify, and close/delete orders — plus close-by.

These are the **only methods in `MT4Account` that change account state**. They live on `mt4_term_api.TradingHelper`. Each takes the raw protobuf request (MT4's trade requests have several fields); friendlier shapes live in [`MT4Service`](../../MT4Service/MT4Service.Overview.md) and the order-construction helpers in [`MT4Sugar`](../../MT4Sugar/MT4Sugar.Master.Overview.md).

---

## Methods in this group

| Method | Returns | One-liner |
|--------|---------|-----------|
| [`order_send(req)`](./order_send.md) | `OrderSendData` | Open a market or pending order. |
| [`order_modify(req)`](./order_modify.md) | `OrderModifyData` | Change SL/TP (open) or price/SL/TP/expiry (pending). |
| [`order_close_delete(req)`](./order_close_delete.md) | `OrderCloseDeleteData` | Close a market order (full/partial) or delete a pending. |
| [`order_close_by(req)`](./order_close_by.md) | `OrderCloseByData` | Close an order by an opposite one. |

---

## Trade errors

Unlike the read methods, trade calls are **not retried** on transient errors (a timed-out `OrderSend` may have filled). MQL failures (130 invalid stops, 132 market closed, 134 not enough money, 138 requote, …) come back inside the reply's `Error` and surface as `MT4Error::Api { mql_code, … }`. `MT4Sugar` maps a non-zero `mql_code` to `MT4Error::TradeError(code, message)`.

---

## A safe trading flow

1. **Quote** the symbol (`quote` / `ensure_symbol`) so it's live in Market Watch.
2. **Send** (`order_send`) — check the reply.
3. **Manage** (`order_modify`) — move SL to breakeven, trail.
4. **Close** (`order_close_delete`, or `order_close_by` to net two opposite orders).

> This is what the `MT4Sugar` orchestrators automate.

---

**Back to:** [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
