# 🐾 Dog Tracker

A simple, fast web app for tracking your dog's meals and treats. Built as a single binary — no external services required.

## Features

- **Today View** — See today's scoop count at a glance, add feedings (+½ or +1 scoop), and log treats
- **Treat Tracking** — Quick buttons for Bone, Pup Cup, and Scraps, plus a custom treat input
- **Calendar View** — Monthly overview with daily scoop totals; tap any day for details
- **Historical Editing** — Add feedings and treats to past days (clearly marked as "edited")
- **Mobile-First** — Designed for iPhone Safari with proper safe areas and large touch targets

## Tech Stack

| Layer    | Technology                      |
|----------|---------------------------------|
| Backend  | Rust, Axum, SeaORM, SQLite      |
| Frontend | Svelte 5, Vite                  |
| Bundling | rust-embed (single binary)      |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v18+)

## Getting Started

```bash
# Build (compiles frontend + backend)
cargo build --release

# Run
./target/release/dog-tracker
```

The app starts at **http://127.0.0.1:3000**.

## Configuration

| Environment Variable | Default                            | Description                              |
|----------------------|------------------------------------|------------------------------------------|
| `PORT`               | `3000`                             | Port to listen on                        |
| `DATABASE_URL`       | `sqlite:./dog_tracker.db?mode=rwc` | SQLite connection string                 |
| `BIND_ALL`           | _(unset)_                          | Set to bind `0.0.0.0` instead of `127.0.0.1` |

## Deployment

This app is designed to run behind [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/) with Cloudflare Access for authentication. No certs or passwords to manage.

```bash
# On your VPS
./dog-tracker                          # listens on 127.0.0.1:3000
cloudflared tunnel run dog-tracker     # exposes via Cloudflare
```

Add an Access Application in the Cloudflare Zero Trust dashboard to restrict access by email.

## API

| Method   | Endpoint                     | Description                        |
|----------|------------------------------|------------------------------------|
| `GET`    | `/api/today`                 | Today's feedings and treats        |
| `GET`    | `/api/day/:date`             | Feedings and treats for a date     |
| `GET`    | `/api/calendar/:year/:month` | Daily totals for a month           |
| `POST`   | `/api/feeding`               | Add a feeding (body: `{ amount_half_scoops: 1\|2, date?: "YYYY-MM-DD" }`) |
| `DELETE` | `/api/feeding/:id`           | Remove a feeding                   |
| `POST`   | `/api/treat`                 | Add a treat (body: `{ name: "...", date?: "YYYY-MM-DD" }`) |
| `DELETE` | `/api/treat/:id`             | Remove a treat                     |

## Project Structure

```
src/
  main.rs        — Server bootstrap, routing
  handlers.rs    — API handlers + static file serving
  feeding.rs     — Feeding entity (SeaORM)
  treat.rs       — Treat entity (SeaORM)
  migration.rs   — Schema creation + SQLite pragmas
frontend/
  src/
    App.svelte           — Shell + tab navigation
    lib/Today.svelte     — Today view
    lib/Calendar.svelte  — Calendar view
    lib/DayDetail.svelte — Historical day view
    lib/api.js           — API client
build.rs         — Compiles frontend during cargo build
```

## License

MIT
