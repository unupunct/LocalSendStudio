import { NavLink, Navigate, Route, Routes } from "react-router-dom";
import { ArrowLeftRight, FolderOpen, FolderSync, Gauge, Home, Laptop, MoreHorizontal, Send, Settings as Cog, Share2, Waypoints } from "lucide-react";
import { useApp } from "./lib/store";
import { Overlays } from "./components/ui";
import { LocalPicker } from "./components/LocalPicker";
import HomePage from "./pages/Home";
import Devices from "./pages/Devices";
import SendPage from "./pages/Send";
import Browse from "./pages/Browse";
import Transfers from "./pages/Transfers";
import Folders from "./pages/Folders";
import Sync from "./pages/Sync";
import Network from "./pages/Network";
import Settings from "./pages/Settings";
import More from "./pages/More";

const nav = [
  { to: "/", label: "Home", icon: Home },
  { to: "/devices", label: "Devices", icon: Laptop },
  { to: "/send", label: "Send", icon: Send },
  { to: "/browse", label: "Browse", icon: FolderOpen },
  { to: "/transfers", label: "Transfers", icon: ArrowLeftRight },
  { to: "/folders", label: "Shared folders", icon: Share2 },
  { to: "/sync", label: "Sync", icon: FolderSync },
  { to: "/network", label: "Network", icon: Gauge },
  { to: "/settings", label: "Settings", icon: Cog },
];
const tabs = [nav[0], nav[2], nav[3], nav[4], { to: "/more", label: "More", icon: MoreHorizontal }];

export default function App() {
  const { identity, live, devices } = useApp();
  const active = Object.values(live).filter((l) => l.status === "transferring").length;
  const online = devices.filter((d) => d.status === "online").length;

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark"><Waypoints size={18} /></div>
          LocalSendStudio
        </div>
        {nav.map(({ to, label, icon: Icon }) => (
          <NavLink key={to} to={to} end={to === "/"} className={({ isActive }) => (isActive ? "nav active" : "nav")}>
            <Icon size={18} />
            {label}
            {to === "/transfers" && active > 0 && <span className="badge">{active}</span>}
          </NavLink>
        ))}
        <div className="side-foot">
          <b>{identity?.name ?? "…"}</b>
          <span className="muted">{online} nearby · v{identity?.version}</span>
        </div>
      </aside>

      <main className="main">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/devices" element={<Devices />} />
          <Route path="/send" element={<SendPage />} />
          <Route path="/browse" element={<Browse />} />
          <Route path="/transfers" element={<Transfers />} />
          <Route path="/folders" element={<Folders />} />
          <Route path="/sync" element={<Sync />} />
          <Route path="/network" element={<Network />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/more" element={<More />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>

      <nav className="tabbar" aria-label="Main">
        {tabs.map(({ to, label, icon: Icon }) => (
          <NavLink key={to} to={to} end={to === "/"} className={({ isActive }) => (isActive ? "active" : "")}>
            <Icon size={21} />
            {label}
            {to === "/transfers" && active > 0 && <span className="badge">{active}</span>}
          </NavLink>
        ))}
      </nav>
      <Overlays />
      <LocalPicker />
    </div>
  );
}
