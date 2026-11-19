// Demo backend used ONLY when building with VITE_DEMO=1 (for screenshots and UI work in a plain browser).
// Never included in release builds: main.tsx imports it behind `import.meta.env.VITE_DEMO`.
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";

const now = Date.now();
const MIN = 60_000;

const devices = [
  { id: "d1", name: "Maya's MacBook Air", os: "macOS", version: "0.2.0", ip: "192.168.1.24", port: 47821, shared_folders: ["Inbox", "Design assets"], status: "online", last_seen: now, trusted: true },
  { id: "d2", name: "Pixel 8", os: "Android", version: "0.2.0", ip: "192.168.1.31", port: 47821, shared_folders: ["Inbox"], status: "online", last_seen: now, trusted: true },
  { id: "d3", name: "Living-room PC", os: "Windows", version: "0.2.0", ip: "192.168.1.12", port: 47821, shared_folders: ["Inbox", "Movies"], status: "online", last_seen: now, trusted: false },
  { id: "d4", name: "Ubuntu workstation", os: "Linux", version: "0.2.0", ip: "192.168.1.40", port: 47821, shared_folders: ["Inbox", "Projects"], status: "online", last_seen: now, trusted: true },
  { id: "d5", name: "Old laptop", os: "Windows", version: "0.1.0", ip: "192.168.1.19", port: 47821, shared_folders: [], status: "offline", last_seen: now - 25 * MIN, trusted: false },
];

const folders = [
  { id: "f1", name: "Inbox", path: "C:\\Users\\you\\Downloads\\LocalSendStudio", enabled: true, read: false, upload: true, manage: false },
  { id: "f2", name: "Photos", path: "D:\\Pictures\\Family", enabled: true, read: true, upload: false, manage: false },
  { id: "f3", name: "Work", path: "D:\\Work\\Shared", enabled: true, read: true, upload: true, manage: false },
];

const hist = [
  ["Trip-2026-final.mp4", "Maya's MacBook Air", "this PC", 1_240_000_000, "completed", 38_000_000, 4 * MIN, "download"],
  ["Budget Q4.xlsx", "this PC", "Pixel 8", 482_000, "completed", 6_100_000, 22 * MIN, "upload"],
  ["IMG_4821.heic", "Pixel 8", "this PC", 3_900_000, "completed", 21_000_000, 41 * MIN, "download"],
  ["design-system.fig", "this PC", "Ubuntu workstation", 58_000_000, "completed", 44_000_000, 3 * 60 * MIN, "upload"],
  ["backup.zip", "this PC", "Living-room PC", 920_000_000, "failed", 0, 5 * 60 * MIN, "upload"],
  ["slides.pdf", "Maya's MacBook Air", "this PC", 12_400_000, "completed", 33_000_000, 26 * 60 * MIN, "download"],
  ["notes.md", "Ubuntu workstation", "this PC", 4_200, "completed", 900_000, 49 * 60 * MIN, "download"],
].map((r, i) => ({ id: `h${i}`, filename: r[0], source: r[1], destination: r[2], size: r[3], started: now - (r[6] as number), ended: now - (r[6] as number) + 20_000, speed: r[5], status: r[4], direction: r[7] }));

const listing = {
  can_upload: true,
  can_manage: false,
  entries: [
    { name: "Brand", is_dir: true, size: 0, modified: 1_760_000_000 },
    { name: "Icons", is_dir: true, size: 0, modified: 1_759_000_000 },
    { name: "Mockups", is_dir: true, size: 0, modified: 1_758_000_000 },
    { name: "hero-banner.png", is_dir: false, size: 2_480_000, modified: 1_761_000_000 },
    { name: "launch-teaser.mp4", is_dir: false, size: 184_000_000, modified: 1_760_500_000 },
    { name: "style-guide.pdf", is_dir: false, size: 6_900_000, modified: 1_760_200_000 },
    { name: "tokens.json", is_dir: false, size: 18_400, modified: 1_760_100_000 },
  ],
};

export async function installDemo() {
  mockWindows("main");
  mockIPC(
    (cmd) => {
      switch (cmd) {
        case "get_identity": return { id: "me", name: "Studio-PC", os: "Windows", version: "0.2.0", port: 47821, trust_mode: "ask" };
        case "get_devices": return devices;
        case "get_shared_folders": return folders;
        case "get_transfer_history": return hist;
        case "get_groups": return [];
        case "list_trusted": return [{ id: "d1", name: "Maya's MacBook Air", added: now - 3 * 86_400_000 }, { id: "d4", name: "Ubuntu workstation", added: now - 9 * 86_400_000 }];
        case "get_download_dir": return "C:\\Users\\you\\Downloads\\LocalSendStudio";
        case "get_settings": return null;
        case "get_local_storage": return { path: "C:\\", available: 312_000_000_000, total: 953_000_000_000 };
        case "get_interfaces": return [{ name: "Wi-Fi", ip: "192.168.1.50", kind: "wifi" }];
        case "list_remote_dir": return listing;
        case "network_diagnostics":
          return { port: 47821, self_reachable: true, devices_online: 4, usable_interfaces: 1, shared_folders: 3, trust_mode: "ask", discovery_port: 47822, interfaces: [{ name: "Wi-Fi", ip: "192.168.1.50", kind: "wifi" }] };
        case "speed_test": return { bytes_per_sec: 41_500_000, rtt_ms: 4 };
        default: return null;
      }
    },
    { shouldMockEvents: true }
  );

  // Show a couple of live transfers and optionally an approval prompt, driven by the URL hash.
  const q = new URLSearchParams(location.search);
  setTimeout(() => {
    if (q.has("live")) {
      emit("transfer-started", { id: "t1", filename: "Trip-2026-final.mp4", direction: "download", device: "Maya's MacBook Air" });
      emit("transfer-started", { id: "t2", filename: "Budget Q4.xlsx", direction: "upload", device: "Pixel 8" });
      emit("transfer-progress", { id: "t1", transferred: 702_000_000, total: 1_240_000_000, speed: 38_400_000, eta: 14 });
      emit("transfer-progress", { id: "t2", transferred: 190_000, total: 482_000, speed: 2_100_000, eta: 1 });
    }
    if (q.has("approve")) emit("approval-request", { id: "d3", name: "Living-room PC", ip: "192.168.1.12" });
  }, 400);
}
