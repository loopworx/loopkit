const https = require("https");
const fs = require("fs");
const path = require("path");
const { createGunzip } = require("zlib");
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

  const res = await download(downloadUrl);
  const binPath = path.join(BINARY_DIR, binName);
  const out = fs.createWriteStream(binPath);

  if (ext === ".tar.gz") {
    res.pipe(createGunzip()).pipe(out);
  } else {
    res.pipe(out);
  }

  await new Promise((resolve, reject) => {
    out.on("finish", resolve);
    out.on("error", reject);
  });

  if (PLATFORM !== "win32") {
    fs.chmodSync(binPath, 0o755);
  }

  console.log(`loopkit: installed v${VERSION} (${target})`);
}

install().catch((err) => {
  console.error("loopkit: install failed:", err.message);
  console.error("Build from source: cargo install loopkit");
  process.exit(1);
});
