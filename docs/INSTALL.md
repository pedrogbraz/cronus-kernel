# Installing CRONUS

`cronus` is a single binary, and it is the whole toolchain. Pick one of the options below, then check it works:

```bash
cronus --version     # cronus 0.1.0 (1a2b3c4d5e6f)
```

Prebuilt binaries are published on
[GitHub Releases](https://github.com/pedrogbraz/cronus-kernel/releases) for:

| Platform              | Target                       | Asset                                        |
|-----------------------|------------------------------|----------------------------------------------|
| macOS, Apple silicon  | `aarch64-apple-darwin`       | `cronus-<version>-aarch64-apple-darwin.tar.gz`      |
| macOS, Intel          | `x86_64-apple-darwin`        | `cronus-<version>-x86_64-apple-darwin.tar.gz`       |
| Linux x86_64 (static) | `x86_64-unknown-linux-musl`  | `cronus-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux x86_64 (glibc)  | `x86_64-unknown-linux-gnu`   | `cronus-<version>-x86_64-unknown-linux-gnu.tar.gz`  |
| Windows x86_64        | `x86_64-pc-windows-msvc`     | `cronus-<version>-x86_64-pc-windows-msvc.zip`       |

Each release also includes a `SHA256SUMS` file. Every archive holds one folder,
`cronus-<version>-<target>/`, containing the binary, `LICENSE` and `README.md`.
Linux ARM64 has no prebuilt binary yet; use `cargo install` (see below).

## 1. Install script (macOS, Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/pedrogbraz/cronus-kernel/main/scripts/install.sh | sh
```

The script works out your OS and CPU, downloads the matching asset, checks it
against `SHA256SUMS` and installs it to `~/.cronus/bin/cronus`. It never uses sudo.
If that folder isn't on your `PATH`, the script prints the line to add.

On Linux it installs the static musl build, which runs on any distribution.

| Variable                   | Default                     | Purpose                                   |
|----------------------------|-----------------------------|-------------------------------------------|
| `CRONUS_VERSION`           | latest release              | Pin a version (`0.2.0` or `v0.2.0`)       |
| `CRONUS_INSTALL_DIR`       | `~/.cronus/bin`             | Where to put the binary                   |
| `CRONUS_REPO`              | `pedrogbraz/cronus-kernel`  | Download releases from another fork       |
| `CRONUS_TARGET`            | detected                    | Force a target, e.g. `x86_64-unknown-linux-gnu` |
| `CRONUS_INSTALL_DRY_RUN=1` | off                         | Print the URLs and install path only; downloads nothing |

```bash
curl -fsSL https://raw.githubusercontent.com/pedrogbraz/cronus-kernel/main/scripts/install.sh \
  | CRONUS_VERSION=0.2.0 CRONUS_INSTALL_DIR="$HOME/bin" sh
```

To upgrade, run the script again. To uninstall, run `rm -rf ~/.cronus/bin/cronus`.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/pedrogbraz/cronus-kernel/main/scripts/install.ps1 | iex
```

This installs `cronus.exe` to `%USERPROFILE%\.cronus\bin`. It reads the same
`CRONUS_VERSION`, `CRONUS_REPO`, `CRONUS_INSTALL_DIR` and `CRONUS_INSTALL_DRY_RUN`
variables. It prints the command to add that folder to your `PATH`, but doesn't
change `PATH` itself.

## 2. Manual download

```bash
version=0.2.0
target=x86_64-unknown-linux-musl   # see the table above
base=https://github.com/pedrogbraz/cronus-kernel/releases/download/v$version

curl -fsSLO "$base/cronus-$version-$target.tar.gz"
curl -fsSLO "$base/SHA256SUMS"
sha256sum --ignore-missing -c SHA256SUMS      # macOS: shasum -a 256 --ignore-missing -c SHA256SUMS
tar -xzf "cronus-$version-$target.tar.gz"
install -m 0755 "cronus-$version-$target/cronus" ~/.local/bin/cronus
```

On macOS, a binary downloaded in a web browser gets quarantined by Gatekeeper.
Clear the flag with `xattr -d com.apple.quarantine ./cronus`. Downloads made with
`curl` and the install script are not quarantined.

## 3. From source with Cargo

You need a stable Rust toolchain and a C compiler (SQLite is compiled in).

```bash
# a tagged release
cargo install --git https://github.com/pedrogbraz/cronus-kernel --tag v0.2.0 --locked

# the latest main
cargo install --git https://github.com/pedrogbraz/cronus-kernel --locked

# a local checkout
cargo install --path . --locked
```

Binaries built from source report `(unknown)` as the commit in `cronus --version`
unless `CRONUS_GIT_SHA` is set at build time.

## 4. Deploy with Docker, Fly.io or Railway

Run `cronus deploy` from your project folder (the one with your `*.cronus` files):

```bash
cronus deploy            # Dockerfile + docker-compose.yml + .dockerignore
cronus deploy --fly      # Dockerfile + fly.toml + .dockerignore
cronus deploy --railway  # Dockerfile + railway.json + .dockerignore
```

The generated `Dockerfile` does **not** compile CRONUS. Instead, it:

- downloads the Linux release binary (`x86_64-unknown-linux-musl`) matching the
  version of the `cronus` that generated it, and verifies its SHA256.
  Override with `--build-arg CRONUS_VERSION=x.y.z`, or `--build-arg CRONUS_REPO=owner/repo`
  for a fork;
- copies your project folder (sources and static assets) into `/app`.
  `.dockerignore` leaves out `data/`, `*.db`, `.cronus/`, `.env` and `.git/`;
- runs `cronus run --prod --host 0.0.0.0 $PORT` as the unprivileged `cronus` user
  (uid 10001). If the container starts as root, the entrypoint first fixes the
  ownership of `/app/data` and then drops privileges;
- declares `/app/data` as the persistent volume. `.cronus/` (JWT key, runtime
  state) is symlinked there.

Images are `linux/amd64` only. On Apple silicon, build with
`docker build --platform linux/amd64 .`.

**Put the database on the volume.** The SQLite path is relative to `/app`, so set it
under `data/` in your app block (`cronus deploy` warns if you don't):

```cronus
app "Shop" {
  port 8080
  database sqlite "./data/app.db"
}
```

For local runs, `mkdir -p data` first; the container creates the folder on its own.

**Set `JWT_SECRET`** in production (at least 32 bytes, e.g. `openssl rand -base64 48`).
Without it, a key is generated and kept at `/app/data/.cronus/jwt.key`.

### Docker Compose

```bash
cronus deploy
docker compose up --build
```

Data persists in the named volume `cronus-data`. To use your own secret,
uncomment the `JWT_SECRET` line in `docker-compose.yml` and start with
`JWT_SECRET="$(openssl rand -base64 48)" docker compose up --build`.

### Fly.io

```bash
cronus deploy --fly
fly auth login
fly launch --copy-config --no-deploy
fly volumes create cronus_data --size 1
fly secrets set JWT_SECRET="$(openssl rand -base64 48)"
fly deploy
```

`fly.toml` mounts `cronus_data` at `/app/data`, sets `PORT`, and health-checks
`GET /api/health`. SQLite lives on one volume, so run a single machine per volume.

### Railway

```bash
cronus deploy --railway
railway login
railway up
```

In the service settings, attach a volume mounted at `/app/data` and set
`JWT_SECRET`. Railway injects `PORT`, and the image reads it.

## Cutting a release (maintainers)

1. Bump `version` in `Cargo.toml`, run `cargo build` so `Cargo.lock` updates, and commit.
2. Optionally rehearse: in Actions, run **release** with `dry_run = true`, then
   download the `release-<version>` artifact.
3. `git tag v<version> && git push origin v<version>`. The workflow fails if the
   tag doesn't match `Cargo.toml`.
4. Check the release has 5 archives and `SHA256SUMS`, then test the install script:
   `curl -fsSL .../scripts/install.sh | CRONUS_VERSION=<version> sh`.
