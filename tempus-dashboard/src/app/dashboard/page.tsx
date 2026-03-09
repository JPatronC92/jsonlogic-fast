"use client";

import { useState } from "react";

export default function DashboardPage() {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<any>(null);

  const runSimulation = () => {
    setLoading(true);
    // Simular un delay de cálculo
    setTimeout(() => {
      setResult({
        baseline: {
          volume: 50000000,
          revenue: 1800000,
          margin: "3.6%"
        },
        scenario: {
          volume: 50000000,
          revenue: 1950000,
          margin: "3.9%"
        },
        impact: {
          absolute: 150000,
          percentage: 8.33
        }
      });
      setLoading(false);
    }, 800);
  };

  return (
    <main className="container">
      <header className="header">
        <h1>Simulador de Escenarios</h1>
        <p>Proyecte el impacto financiero comparando esquemas de precios.</p>
      </header>

      <div className="grid-2">
        {/* Panel de Configuración */}
        <div className="card">
          <h2>Parámetros de Simulación</h2>
          
          <div className="input-group">
            <label>Dataset de Transacciones</label>
            <select>
              <option>Histórico Q4 2023 (50M de volumen)</option>
              <option>Proyección Q1 2024 (Crecimiento 15%)</option>
            </select>
          </div>

          <div className="input-group">
            <label>Esquema Base (Actual)</label>
            <select disabled>
              <option>Comisión Standard (v1.0)</option>
            </select>
          </div>

          <div className="input-group">
            <label>Esquema a Simular</label>
            <select>
              <option>Nuevo Modelo Escalonado (v2.0 Draft)</option>
              <option>Tarifa Plana + 1%</option>
            </select>
          </div>

          <button 
            className="btn-primary mt-4" 
            onClick={runSimulation}
            disabled={loading}
          >
            {loading ? "Ejecutando Simulación..." : "Ejecutar Comparativa"}
          </button>
        </div>

        {/* Panel de Resultados */}
        <div className="card">
          <h2>Impacto Proyectado</h2>
          
          {!result ? (
            <div style={{ height: "100%", display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-muted)" }}>
              Ejecute la simulación para ver la comparativa.
            </div>
          ) : (
            <div>
              <div className="stats-grid" style={{ marginTop: "1rem", marginBottom: "2rem" }}>
                <div className="stat-box">
                  <span className="stat-label">Ingreso Actual (Base)</span>
                  <span className="stat-value">${(result.baseline.revenue / 1000).toFixed(0)}k</span>
                </div>
                <div className="stat-box">
                  <span className="stat-label">Ingreso Simulado</span>
                  <span className="stat-value text-success">${(result.scenario.revenue / 1000).toFixed(0)}k</span>
                </div>
                <div className="stat-box">
                  <span className="stat-label">Crecimiento (Δ)</span>
                  <span className="stat-value">+{result.impact.percentage}%</span>
                </div>
              </div>

              <div style={{ padding: "1.5rem", background: "var(--bg-color)", border: "1px solid var(--border-color)", borderRadius: "8px" }}>
                <h3 style={{ fontSize: "1rem", fontWeight: 500, marginBottom: "0.5rem" }}>Resumen Ejecutivo</h3>
                <p style={{ color: "var(--text-muted)", fontSize: "0.95rem" }}>
                  Al aplicar el <strong>Nuevo Modelo Escalonado</strong> sobre el volumen histórico, se proyecta un incremento de <strong>${(result.impact.absolute).toLocaleString()}</strong> en revenue total, mejorando el margen general de {result.baseline.margin} a {result.scenario.margin}.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </main>
  );
}
