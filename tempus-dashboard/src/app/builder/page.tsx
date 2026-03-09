"use client";

import { useState } from "react";

export default function BuilderPage() {
  const [ruleName, setRuleName] = useState("");
  const [feeType, setFeeType] = useState("PERCENTAGE");
  const [startDate, setStartDate] = useState("");
  const [saved, setSaved] = useState(false);

  const handleSave = () => {
    // Simulated save
    setSaved(true);
    setTimeout(() => setSaved(false), 3000);
  };

  return (
    <main className="container">
      <header className="header">
        <h1>Configuración de Reglas</h1>
        <p>Diseñe y versione la lógica de negocio.</p>
      </header>

      <div style={{ maxWidth: "600px", margin: "0 auto" }}>
        <div className="card">
          <h2>Nueva Regla de Facturación</h2>
          
          <div className="input-group">
            <label>Nombre de la Regla</label>
            <input 
              type="text" 
              placeholder="Ej. Comisión Standard Marketplace" 
              value={ruleName}
              onChange={(e) => setRuleName(e.target.value)}
            />
          </div>

          <div className="input-group">
            <label>Modelo de Cargo</label>
            <select value={feeType} onChange={(e) => setFeeType(e.target.value)}>
              <option value="PERCENTAGE">Porcentaje sobre volumen (%)</option>
              <option value="FIXED_FEE">Cargo Fijo por transacción ($)</option>
              <option value="TIERED">Escalonado (Tiers)</option>
            </select>
          </div>

          <div className="input-group">
            <label>Fecha de Activación (Vigencia)</label>
            <input 
              type="date" 
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
            />
          </div>

          <button 
            className="btn-primary" 
            style={{ width: "100%", marginTop: "1rem" }}
            onClick={handleSave}
            disabled={!ruleName || !startDate}
          >
            {saved ? "Guardado Exitosamente" : "Guardar y Versionar Regla"}
          </button>
        </div>

        <div className="mt-8">
          <h3 style={{ fontSize: "1.1rem", fontWeight: 500, marginBottom: "1rem" }}>Timeline de Versiones</h3>
          <div className="card" style={{ padding: "1.5rem" }}>
            <div style={{ display: "flex", justifyContent: "space-between", borderBottom: "1px solid var(--border-color)", paddingBottom: "1rem", marginBottom: "1rem" }}>
              <div>
                <strong>v2.0 (Borrador)</strong>
                <div style={{ color: "var(--text-muted)", fontSize: "0.85rem" }}>{ruleName || "Sin nombre"}</div>
              </div>
              <div style={{ color: "var(--warning)", fontSize: "0.85rem", fontWeight: 500 }}>Pendiente</div>
            </div>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <div>
                <strong>v1.0 (Activa)</strong>
                <div style={{ color: "var(--text-muted)", fontSize: "0.85rem" }}>Comisión Base 3.6%</div>
              </div>
              <div style={{ color: "var(--success)", fontSize: "0.85rem", fontWeight: 500 }}>Desde 01/01/2024</div>
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
