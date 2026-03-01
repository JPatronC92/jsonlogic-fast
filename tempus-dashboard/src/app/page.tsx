"use client";

import { useState } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

export default function Home() {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<any>(null);
  const [chartData, setChartData] = useState<any[]>([]);
  const [count, setCount] = useState(100000);

  const runSimulation = async () => {
    setLoading(true);
    setResult(null);
    setChartData([]);

    // Generate random transactions for the simulation
    const transactions = Array.from({ length: count }).map((_, i) => ({
      amount: Math.floor(Math.random() * 5000) + 100,
      currency: "MXN",
      country: i % 2 === 0 ? "MX" : "US",
      payment_method: "CREDIT_CARD",
    }));

    try {
      const res = await fetch("/api/simulate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          scheme_urn: "urn:pricing:stripe:mx:standard",
          execution_date: new Date().toISOString(),
          transactions,
        }),
      });
      const data = await res.json();
      setResult(data);

      if (!data.error) {
        // Generate a simulated 30-day timeline based on the total results for the dashboard
        const days = 30;
        const timeline = [];
        let remainingVol = data.total_processed_volume;
        let remainingFees = data.total_fees_collected;
        let remainingNet = data.total_net_settlement;

        for (let i = days; i > 0; i--) {
          const isLast = i === 1;
          // Add some random noise to the daily distribution
          const weight = isLast ? 1 : (Math.random() * 0.5 + 0.5) / i;
          
          const dailyVol = isLast ? remainingVol : data.total_processed_volume * weight;
          const dailyFees = isLast ? remainingFees : data.total_fees_collected * weight;
          const dailyNet = isLast ? remainingNet : data.total_net_settlement * weight;

          remainingVol -= dailyVol;
          remainingFees -= dailyFees;
          remainingNet -= dailyNet;

          const date = new Date();
          date.setDate(date.getDate() - i + 1);

          timeline.push({
            date: date.toLocaleDateString("en-US", { month: "short", day: "numeric" }),
            Volume: Math.round(dailyVol),
            Fees: Math.round(dailyFees),
            Net: Math.round(dailyNet),
          });
        }
        setChartData(timeline);
      }
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="container">
      <header className="header">
        <h1>Tempus Time-Travel Engine</h1>
        <p>Deterministic Billing & Commission Infrastructure</p>
      </header>

      <div className="dashboard-layout">
        <div className="grid-2">
          {/* Controls Panel */}
          <div className="card">
            <h2>Batch Simulation Setup</h2>
            <p className="desc">
              Test the financial impact of a pricing scheme against millions of historical transactions instantly using our Rust-powered core.
            </p>
            
            <div className="input-group">
              <label>Number of Transactions</label>
              <input 
                type="number" 
                value={count} 
                onChange={(e) => setCount(Number(e.target.value))} 
                step="10000"
                min="1000"
              />
            </div>

            <button onClick={runSimulation} disabled={loading} className="btn-primary">
              {loading ? "Running Fast-Path Batch..." : "Run Time-Travel Audit"}
            </button>

            {result && !result.error && (
              <div className="stats-grid" style={{ marginTop: "2rem" }}>
                <div className="stat-box">
                  <span className="stat-label">Processed Tx</span>
                  <span className="stat-value">{result.transactions_count?.toLocaleString() ?? 0}</span>
                </div>
                <div className="stat-box">
                  <span className="stat-label">Failed Tx</span>
                  <span className="stat-value warning">{result.failed_transactions?.toLocaleString() ?? 0}</span>
                </div>
              </div>
            )}
          </div>

          {/* Results & Chart Panel */}
          <div className="card">
            <h2>Financial Impact Analysis</h2>
            {!result ? (
              <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#64748b' }}>
                Run a simulation to see the projected P&L breakdown and timeline.
              </div>
            ) : result.error ? (
              <p className="error">{result.error}</p>
            ) : (
              <>
                <div className="stats-grid">
                  <div className="stat-box">
                    <span className="stat-label">Total Volume (MXN)</span>
                    <span className="stat-value">${result.total_processed_volume?.toLocaleString() ?? 0}</span>
                  </div>
                  <div className="stat-box">
                    <span className="stat-label">Net Settlement</span>
                    <span className="stat-value success">${result.total_net_settlement?.toLocaleString() ?? 0}</span>
                  </div>
                  <div className="stat-box" style={{ gridColumn: "span 2" }}>
                    <span className="stat-label">Total Fees Collected (Platform Revenue)</span>
                    <span className="stat-value accent">${result.total_fees_collected?.toLocaleString() ?? 0}</span>
                  </div>
                </div>

                <div className="chart-container">
                  <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={chartData} margin={{ top: 20, right: 0, left: 0, bottom: 0 }}>
                      <defs>
                        <linearGradient id="colorNet" x1="0" y1="0" x2="0" y2="1">
                          <stop offset="5%" stopColor="#10b981" stopOpacity={0.3}/>
                          <stop offset="95%" stopColor="#10b981" stopOpacity={0}/>
                        </linearGradient>
                        <linearGradient id="colorFees" x1="0" y1="0" x2="0" y2="1">
                          <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3}/>
                          <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                        </linearGradient>
                      </defs>
                      <CartesianGrid strokeDasharray="3 3" stroke="#334155" vertical={false} />
                      <XAxis dataKey="date" stroke="#94a3b8" fontSize={12} tickLine={false} axisLine={false} />
                      <YAxis stroke="#94a3b8" fontSize={12} tickLine={false} axisLine={false} tickFormatter={(value) => `$${value / 1000}k`} />
                      <Tooltip 
                        contentStyle={{ backgroundColor: '#1e293b', borderColor: '#334155', borderRadius: '8px', color: '#f8fafc' }}
                        itemStyle={{ color: '#f8fafc' }}
                        formatter={(value: any) => {
                          const val = Array.isArray(value) ? value[0] : value;
                          return [`$${Number(val).toLocaleString()}`, undefined];
                        }}
                      />
                      <Area type="monotone" dataKey="Net" stroke="#10b981" strokeWidth={2} fillOpacity={1} fill="url(#colorNet)" />
                      <Area type="monotone" dataKey="Fees" stroke="#3b82f6" strokeWidth={2} fillOpacity={1} fill="url(#colorFees)" />
                    </AreaChart>
                  </ResponsiveContainer>
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    </main>
  );
}
