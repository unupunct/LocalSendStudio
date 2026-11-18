import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from "recharts";

export default function Analytics() {
  const [history, setHistory] = useState<any[]>([]);
  useEffect(() => { (async () => setHistory(await api.transferHistory()))(); }, []);

  const total = history.reduce((a, t) => a + (t.size || 0), 0);
  const uploads = history.filter(t => t.direction === "upload").reduce((a, t) => a + t.size, 0);
  const downloads = history.filter(t => t.direction === "download").reduce((a, t) => a + t.size, 0);

  const byDay: Record<string, number> = {};
  history.forEach(t => {
    const d = new Date(t.started).toLocaleDateString();
    byDay[d] = (byDay[d] || 0) + t.size;
  });
  const chartData = Object.entries(byDay).map(([name, value]) => ({ name, value: Math.round(value / 1024 / 1024) }));

  const pie = [
    { name: "Upload", value: uploads },
    { name: "Download", value: downloads },
  ];

  return (
    <div>
      <h1>Analytics</h1>
      <div className="grid grid-3">
        <div className="card"><h2>Total Data</h2><div>{(total / 1024 / 1024).toFixed(1)} MB</div></div>
        <div className="card"><h2>Transfers</h2><div>{history.length}</div></div>
        <div className="card"><h2>Devices Active</h2><div>{new Set(history.map(t => t.source)).size}</div></div>
      </div>
      <div className="card" style={{ marginTop: 16, height: 300 }}>
        <h2>Daily Volume (MB)</h2>
        <ResponsiveContainer width="100%" height="90%">
          <BarChart data={chartData}>
            <XAxis dataKey="name" stroke="#8b949e" />
            <YAxis stroke="#8b949e" />
            <Tooltip />
            <Bar dataKey="value" fill="#22c55e" />
          </BarChart>
        </ResponsiveContainer>
      </div>
      <div className="card" style={{ marginTop: 16, height: 260 }}>
        <h2>Upload vs Download</h2>
        <ResponsiveContainer width="100%" height="90%">
          <PieChart>
            <Pie data={pie} dataKey="value" nameKey="name" outerRadius={80}>
              <Cell fill="#22c55e" /><Cell fill="#06b6d4" />
            </Pie>
            <Tooltip />
          </PieChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
