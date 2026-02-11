import { Strategy, StockInfo } from './trade.model';

export interface StockWithStrategies {
  stock: StockInfo;
  strategies: Strategy[];
}

export interface HistoricalDataPoint {
  timestamp: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export interface HistoricalResponse {
  symbol: string;
  timeframe: string;
  data: HistoricalDataPoint[];
}

export interface EmaHistoryPoint {
  date: string;
  value: number;
}

export interface IndicatorsResponse {
  symbol: string;
  timeframe: string;
  latest_date: string;
  indicators: {
    rsi: number;
    stochastic_k: number;
    ema_20: number;
    ema_50: number;
    ema_200: number;
    pivot_point: number;
    resistance_1: number;
    support_1: number;
  };
  historical_ema_50: EmaHistoryPoint[];
}
