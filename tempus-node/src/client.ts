import axios, { AxiosInstance } from "axios";
import {
  CalculateFeeResponse,
  BatchSimulateResponse,
  CalculateRequest,
  BatchSimulateRequest,
} from "./types";

export interface TempusClientOptions {
  apiKey: string;
  baseURL?: string; // e.g. "http://localhost:8000/api/v1" or production URL
  timeout?: number;
}

export class TempusClient {
  private api: AxiosInstance;

  constructor(options: TempusClientOptions) {
    if (!options.apiKey) {
      throw new Error("Tempus API Key is required.");
    }

    this.api = axios.create({
      baseURL: options.baseURL || "http://localhost:8000/api/v1",
      timeout: options.timeout || 10000,
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": options.apiKey,
        "X-Tempus-SDK": "node/1.0.0",
      },
    });
  }

  /**
   * Ejecuta el motor matemático para calcular tarifas de una sola transacción.
   * Proporciona trazabilidad y sellado criptográfico (Time-Travel).
   * 
   * @param request Datos de la transacción y el scheme_id
   * @returns Desglose de fees y monto neto
   */
  async calculate(request: CalculateRequest): Promise<CalculateFeeResponse> {
    try {
      const response = await this.api.post<CalculateFeeResponse>(
        "/billing/calculate",
        request
      );
      return response.data;
    } catch (error: any) {
      this.handleError(error);
    }
  }

  /**
   * Simula el impacto de un esquema de precios sobre un lote masivo de transacciones.
   * Útil para backtesting, proyecciones de P&L y auditorías.
   * 
   * @param request Lista de transacciones y el scheme_id
   * @returns Agregado de volumen, fees y settlement
   */
  async simulateBatch(request: BatchSimulateRequest): Promise<BatchSimulateResponse> {
    try {
      const response = await this.api.post<BatchSimulateResponse>(
        "/billing/simulate-batch",
        request
      );
      return response.data;
    } catch (error: any) {
      this.handleError(error);
    }
  }

  private handleError(error: any): never {
    if (error.response) {
      // El servidor respondió con un status code fuera del rango 2xx
      throw new Error(
        `Tempus API Error [${error.response.status}]: ${error.response.data?.detail || JSON.stringify(error.response.data) || error.message
        }`
      );
    } else if (error.request) {
      // La petición fue hecha pero no hubo respuesta
      throw new Error(`Tempus API Error: No response received. ${error.message}`);
    } else {
      // Algo pasó al configurar la petición
      throw new Error(`Tempus SDK Error: ${error.message}`);
    }
  }
}
