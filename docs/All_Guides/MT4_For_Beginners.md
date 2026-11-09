# MT4 for Beginners: Your First Steps

Welcome! This guide is for complete beginners who have never used MetaTrader 4. We'll set up the terminal and a free demo account, then point RustMT4 at it.

---

## What is MetaTrader 4 (MT4)?

**MetaTrader 4** is a widely-used trading platform. It lets you:

- Trade forex, metals, commodities and more
- Analyze markets with charts and indicators
- Run automated strategies
- Monitor positions in real time

Think of MT4 as the "cockpit" for the markets. **RustMT4** lets you control that cockpit programmatically from Rust, over gRPC.

---

## Demo vs Live: start safely

### Why start with a demo account?

A **demo account** is a risk-free practice account with virtual money. We **strongly recommend** it while you learn:

- ✅ **Zero risk** — virtual funds (typically $10,000–$100,000)
- ✅ **Real market conditions** — live prices and behavior
- ✅ **Learn the platform** before risking real money
- ✅ **Test your code** — perfect for developing RustMT4 bots

### Live account

A **live account** uses real money. Only move to live when you understand MT4, have tested your strategy thoroughly on demo, and accept the risks.

---

## Quick start: creating a demo account

The fastest path is to create a demo account directly in the MT4 terminal.

### Step 1️⃣ — Download and install the MT4 terminal

1. Visit the official MetaQuotes site: [https://www.metatrader4.com/en/download](https://www.metatrader4.com/en/download)
2. Download the build for your OS.
3. Install and launch it.

---

### Step 2️⃣ — Launch MT4 and find the Navigator panel

On first launch MT4 usually shows a popup to open a new account. If it doesn't, find the **Navigator** panel.

![Navigator Panel Location](../Guide_Images/1.%20NAV.png)

The **Navigator** is on the **left**, below **Market Watch**. It has sections: Accounts, Indicators, Expert Advisors, and others.

---

### Step 3️⃣ — Open a new demo account

1. In **Navigator**, find **Accounts**.
2. **Right-click** "Accounts".
3. Choose **"Open an account"**.

This opens the broker server search.

---

### Step 4️⃣ — Select the MetaQuotes demo server

In the broker search, for your first account use the simplest option:

- Look for **"MetaQuotes-Demo"** / **"MetaQuotes Ltd"** (use the search field if needed).
- Select it and click **Next**.

![MetaQuotes Server Selection](../Guide_Images/2.%20META.png)

---

### Step 5️⃣ — Choose "Open a new demo account"

![Open New Demo Account](../Guide_Images/3.%20OPEN.png)

Select **"Open a new demo account"** and click **Next**.

---

### Step 6️⃣ — Fill in your details

- **Name / Last Name / Email**
- **Phone** — optional for demo
- **Deposit** — virtual amount (e.g. $10,000)
- **Leverage** — 1:100 is a good start
- **Currency** — USD, EUR, …

Accept the terms and click **Next**.

![Personal Information Form](../Guide_Images/4.%20creator.png)

---

### Step 7️⃣ — Save your credentials

MT4 creates the account and shows:

- **Login** — account number (e.g. 591129415)
- **Password** — master (trading) password
- **Investor Password** — read-only
- **Server** — e.g. "MetaQuotes-Demo"

**⚠️ Save these now.** You'll need **Login**, **Password** and **Server** for RustMT4.

![Account Credentials](../Guide_Images/5.%20OKK.png)

Click **Finish**.

---

### Step 8️⃣ — Configure RustMT4 with your credentials

Open **`examples/0_common/settings.json`** and fill in what you saved:

```json
{
  "user": 591129415,
  "password": "YourPasswordHere",
  "server_name": "MetaQuotes-Demo",
  "name": "rustmt4-examples",
  "api_key": null,
  "test_symbol": "EURUSD"
}
```

| Key | Meaning |
|-----|---------|
| `user` | Your login number |
| `password` | Your master password |
| `server_name` | Exact broker server name from MT4 (e.g. `MetaQuotes-Demo`) |
| `name` | Any label for this terminal session |
| `api_key` | Your MetaRPC key, or `null` to use the `MRPC_API_KEY` env var |
| `test_symbol` | Default symbol the examples trade |

> Prefer env vars? `MT4_LOGIN`, `MT4_PASSWORD`, `MT4_SERVER`, `MT4_SYMBOL`, `MT4_NAME`, `MRPC_API_KEY` override the file (env wins). Never commit real credentials.

The gRPC endpoint (`mt4.mrpc.pro:443`) is built into the SDK — you don't set a host/port in the file.

---

### Step 9️⃣ — Verify the MT4 connection

Back in the terminal:

1. MT4 logs you into the new demo account automatically.
2. Bottom-right corner should show a **green connection indicator** and your **balance** (e.g. $10,000).

You're ready to test.

---

### Step 🔟 — Run RustMT4

With `settings.json` filled in, run an example:

```bash
# from the repo root
cargo run --example quickstart            # the 10-minute flow

cargo run --example menu                  # interactive menu of all examples
```

Trades RustMT4 sends appear in your MT4 terminal in real time, and the demo balance moves accordingly. **Experiment freely — it's virtual money.**

---

## 🧩 MT4 password types

### Master password
- **Full access** — open/close trades, change settings.
- **This is the one RustMT4 uses** (`password` in `settings.json`).

### Investor password
- **View-only** — see trades/balance/history, **cannot trade**.
- Good for sharing performance or monitoring tools.
- Set it in **Tools → Options → Server → Change** (Investor Password).

---

## Choosing a broker (optional)

The MetaQuotes demo is fine for practice. Later you might pick a broker for live trading, better spreads, or regional support.

**Not financial advice — do your own research.** Popular MT4 brokers include IC Markets, Pepperstone, OANDA, XM, RoboForex, FxPro.

What to look for:

- ✅ **Regulation** (FCA, ASIC, CySEC, …)
- ✅ **MT4 support** (specifically MT4, not only MT5)
- ✅ **Free demo** (note: demos often expire after ~30 days of inactivity)
- ✅ **Good spreads** and responsive **support**

---

## What's next?

### 1. Connect RustMT4
👉 **[Getting Started with RustMT4](GETTING_STARTED.md)** · **[Your First Project](Your_First_Project.md)**

### 2. Understand the architecture
RustMT4 has three layers — start wherever fits:

- **[MT4Account](../MT4Account/MT4Account.Master.Overview.md)** — raw gRPC, maximum control.
- **[MT4Service](../MT4Service/MT4Service.Overview.md)** — friendly wrappers, most common path.
- **[MT4Sugar](../MT4Sugar/MT4Sugar.Master.Overview.md)** — one-liners, most beginner-friendly.

---

## Important reminders

### 🛡️ Security
- Never share your master password. Use the investor password for monitoring tools. Keep credentials in a password manager.

### 📊 Risk management
Even on demo, build good habits: always use a stop loss, risk no more than 1–2% per trade, understand position sizing ([Risk Sizing](../MT4Sugar/4.%20Risk_Sizing.md)).

### 🧪 Testing
Test all code on demo first. Verify strategies over weeks, not days. Demo results don't guarantee live results.

---

## Useful commands

```bash
cargo build                               # build the library + examples
cargo run --example quickstart            # run the quick start
cargo run --example menu                  # interactive example launcher
rustc --version                           # check your Rust toolchain
```

> On Windows, RustMT4 builds with the **WinLibs GNU toolchain** — see the repo README for toolchain setup.

---

**Ready to code?** Head to [Getting Started](GETTING_STARTED.md) and connect RustMT4 to your demo account.

**Good luck! 🚀**
