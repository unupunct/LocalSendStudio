import { NavLink, Routes, Route } from "react-router-dom";
import Dashboard from "./pages/Dashboard";
import NetworkMap from "./pages/NetworkMap";
import Devices from "./pages/Devices";
import GlobalExplorer from "./pages/GlobalExplorer";
import QuickShare from "./pages/QuickShare";
import Transfers from "./pages/Transfers";
import Sync from "./pages/Sync";
import SharedFolders from "./pages/SharedFolders";
import Diagnostics from "./pages/Diagnostics";
import Analytics from "./pages/Analytics";
import Settings from "./pages/Settings";

const nav = [
  ["Dashboard", "/"],
  ["Network Map", "/map"],
  ["Devices", "/devices"],
  ["Global Explorer", "/explorer"],
  ["Quick Share", "/quick"],
  ["Transfers", "/transfers"],
  ["Synchronization", "/sync"],
  ["Shared Folders", "/shared"],
  ["Diagnostics", "/diag"],
  ["Analytics", "/analytics"],
  ["Settings", "/settings"],
];

export default function App() {
  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">LocalSendStudio</div>
        <nav>
          {nav.map(([label, to]) => (
            <NavLink key={to} to={to} end={to === "/"} className={({ isActive }) => isActive ? "nav active" : "nav"}>
              {label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="main">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/map" element={<NetworkMap />} />
          <Route path="/devices" element={<Devices />} />
          <Route path="/explorer" element={<GlobalExplorer />} />
          <Route path="/quick" element={<QuickShare />} />
          <Route path="/transfers" element={<Transfers />} />
          <Route path="/sync" element={<Sync />} />
          <Route path="/shared" element={<SharedFolders />} />
          <Route path="/diag" element={<Diagnostics />} />
          <Route path="/analytics" element={<Analytics />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}
