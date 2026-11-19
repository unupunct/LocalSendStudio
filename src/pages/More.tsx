import { Link } from "react-router-dom";
import { ChevronRight, FolderSync, Gauge, Laptop, Settings, Share2 } from "lucide-react";
import { Page } from "../components/ui";

const items = [
  { to: "/devices", label: "Devices", sub: "Nearby devices, add by IP", icon: Laptop },
  { to: "/folders", label: "Shared folders", sub: "Choose what others can see", icon: Share2 },
  { to: "/sync", label: "Sync", sub: "Copy a folder between devices", icon: FolderSync },
  { to: "/network", label: "Network", sub: "Troubleshooting and speed test", icon: Gauge },
  { to: "/settings", label: "Settings", sub: "Name, privacy, theme", icon: Settings },
];

export default function More() {
  return (
    <Page title="More">
      <div className="card" style={{ padding: 6 }}>
        {items.map(({ to, label, sub, icon: Icon }) => (
          <Link key={to} to={to} className="file" style={{ textDecoration: "none", color: "inherit", padding: 14 }}>
            <div className="dev-ico" style={{ width: 38, height: 38 }}><Icon size={18} /></div>
            <div className="grow"><div style={{ fontWeight: 600 }}>{label}</div><div className="muted small">{sub}</div></div>
            <ChevronRight size={18} className="muted" />
          </Link>
        ))}
      </div>
    </Page>
  );
}
