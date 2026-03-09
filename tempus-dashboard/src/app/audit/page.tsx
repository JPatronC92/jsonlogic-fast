"use client";

import { useState } from "react";

export default function AuditPage() {
  const [transactionId, setTransactionId] = useState("");
  const [searching, setSearching] = useState(false);
  const [result, setResult] = useState<any>(null);

  const handleSearch = () => {
    setSearching(true);
    setTimeout(() => {
      setResult({
        client: "TechCorp Inc.",
        date: "2024-03-01T14:30:00Z",
        transactionId: transactionId || "tx_8f93j29d",
        amount: 15000.00,
        currency: "MXN",
        appliedRule: {
          name: "Nuevo Modelo Escalonado",
          version: "v2.0",
          urn: "urn:pricing:stripe:mx:standard",
          logic: "Tiers: >10k = 1.5%"
        },
        calculation: {
          fee: 225.00,
          net: 14775.00
        }
      });
      setSearching(false);
    }, 600);
  };

  return (
    <main className="container">
      <header className="header">
        <h1>Auditoría Determinista</h1>
        <p>Verifique la regla exacta aplicada a cualquier transacción histórica.</p>
      </header>

      <div className="card" style={{ maxWidth: "800px", margin: "0 auto" }}>
        <div style={{ display: "flex", gap: "1rem", alignItems: "flex-end", marginBottom: "2rem", borderBottom: "1px solid var(--border-color)", paddingBottom: "2rem" }}>
          <div className="input-group" style={{ marginBottom: 0, flex: 1 }}>
            <label>ID de Transacción, Cliente o Fecha</label>
            <input 
              type="text" 
              placeholder="Ej. tx_8f93j29d" 
              value={transactionId}
              onChange={(e) => setTransactionId(e.target.value)}
            />
          </div>
          <button 
            className="btn-primary" 
            style={{ width: "auto", minWidth: "120px" }}
            onClick={handleSearch}
            disabled={searching}
          >
            {searching ? "Buscando..." : "Buscar"}
          </button>
        </div>

        {!result ? (
          <div style={{ padding: "3rem 0", textAlign: "center", color: "var(--text-muted)" }}>
            Ingrese un parámetro de búsqueda para visualizar la auditoría.
          </div>
        ) : (
          <div>
            <h2 style={{ fontSize: "1.1rem", marginBottom: "1rem" }}>Resultados de la Auditoría</h2>
            
            <div className="grid-2">
              <div>
                <p style={{ color: "var(--text-muted)", fontSize: "0.85rem", marginBottom: "0.25rem" }}>Cliente</p>
                <p style={{ fontWeight: 500, marginBottom: "1rem" }}>{result.client}</p>

                <p style={{ color: "var(--text-muted)", fontSize: "0.85rem", marginBottom: "0.25rem" }}>Fecha Exacta de Ejecución</p>
                <p style={{ fontWeight: 500, marginBottom: "1rem" }}>{new Date(result.date).toLocaleString()}</p>

                <p style={{ color: "var(--text-muted)", fontSize: "0.85rem", marginBottom: "0.25rem" }}>Monto Procesado</p>
                <p style={{ fontWeight: 500, marginBottom: "1rem" }}>${result.amount.toLocaleString()} {result.currency}</p>
              </div>

              <div style={{ background: "var(--bg-color)", padding: "1.5rem", borderRadius: "8px", border: "1px solid var(--border-color)" }}>
                <p style={{ color: "var(--text-muted)", fontSize: "0.85rem", marginBottom: "0.5rem" }}>Regla Matemáticamente Aplicada</p>
                <p style={{ fontWeight: 500 }}>{result.appliedRule.name}</p>
                <p style={{ fontSize: "0.9rem", fontFamily: "monospace", color: "var(--accent)", marginBottom: "1rem" }}>{result.appliedRule.version} ({result.appliedRule.urn})</p>

                <div style={{ borderTop: "1px dashed var(--border-color)", paddingTop: "1rem", marginTop: "1rem", display: "flex", justifyContent: "space-between" }}>
                  <span style={{ color: "var(--text-muted)" }}>Cálculo (Comisión)</span>
                  <span style={{ fontWeight: 600 }}>${result.calculation.fee.toLocaleString()}</span>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", marginTop: "0.5rem" }}>
                  <span style={{ color: "var(--text-muted)" }}>Liquidación Neta</span>
                  <span style={{ fontWeight: 600, color: "var(--success)" }}>${result.calculation.net.toLocaleString()}</span>
                </div>
              </div>
            </div>

            <div style={{ marginTop: "2rem", padding: "1rem", background: "rgba(16, 185, 129, 0.1)", border: "1px solid rgba(16, 185, 129, 0.2)", borderRadius: "6px", display: "flex", gap: "0.5rem", alignItems: "center", color: "var(--success)", fontSize: "0.9rem", fontWeight: 500 }}>
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
              Certificado de Autenticidad: El cálculo coincide exactamente con la lógica firmada en la fecha de la transacción.
            </div>
          </div>
        )}
      </div>
    </main>
  );
}
