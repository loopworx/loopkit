const https = require("https");
const fs = require("fs");
const path = require("path");
const os = require("os");
const { execSync } = require("child_process");

const PACKAGE_JSON = require("./package.json");
const VERSION = PACKAGE_JSON.version;
const BINARY_DIR = path.join(__dirname, "bin");

const PLATFORM = process.platform;
const ARCH = process.arch;

const TARGET_MAP = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
  "win32-arm64": "aarch64-pc-windows-msvc",
};

const key = `${PLATFORM}-${ARCH}`;
const target = TARGET_MAP[key];

if (!target) {
  console.warn(`loopkit: no prebuilt binary for ${key}, skipping install.`);
  console.warn("Build from source: cargo install loopkit");
  process.exit(0);
}

const ext = PLATFORM === "win32" ? ".zip" : ".tar.gz";
const assetName = `loopkit-${target}${ext}`;
const downloadUrl = `https://github.com/loopworx/loopkit/releases/download/v${VERSION}/${assetName}`;
const binName = PLATFORM === "win32" ? "loopkit.exe" : "loopkit";

function download(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return download(res.headers.location).then(resolve, reject);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
        }
        resolve(res);
      })
      .on("error", reject);
  });
}

async function install() {
  console.log(`loopkit: downloading ${assetName}`);

  if (!fs.existsSync(BINARY_DIR)) {
    fs.mkdirSync(BINARY_DIR, { recursive: true });
  }

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "loopkit-"));
  const archivePath = path.join(tmpDir, assetName);

  // Download archive
  const res = await download(downloadUrl);
  const out = fs.createWriteStream(archivePath);
  res.pipe(out);
  await new Promise((resolve, reject) => {
    out.on("finish", resolve);
    out.on("error", reject);
  });

  // Extract
  if (ext === ".tar.gz") {
    execSync(`tar xzf "${archivePath}" -C "${BINARY_DIR}"`);
  } else {
    execSync(`7z x "${archivePath}" -o"${BINARY_DIR}" -y`);
  }

  const binPath = path.join(BINARY_DIR, binName);
  if (PLATFORM !== "win32") {
    fs.chmodSync(binPath, 0o755);
  }

  // Clean up temp
  fs.rmSync(tmpDir, { recursive: true, force: true });

  console.log(`loopkit: installed v${VERSION} (${target})`);
}

install().catch((err) => {
  console.error("loopkit: install failed:", err.message);
  console.error("Build from source: cargo install loopkit");
  process.exit(1);
});
