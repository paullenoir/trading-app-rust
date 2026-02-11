export interface Strategy {
  strategy_name: string;
  recommendation: 'BUY' | 'HOLD' | 'SELL';
  metadata?: Record<string, number>;
}

export interface StockInfo {
  symbol: string;
  name: string;
}

export interface OpenTrade {
  symbol: string;
  quantite_totale: number;
  prix_moyen: number;
  entry_date: string;
}

export interface OpenTradeWithRecommendations {
  symbol: string;
  quantite_totale: number;
  prix_moyen: number;
  current_price: number;
  entry_date: string;
  stock: StockInfo;
  strategies: Strategy[];
}

export interface ClosedTrade {
  symbol: string;
  gain_dollars: number;
  pourcentage_gain: number;
  temps_jours: number;
  devise: string;
  quantite_achat: number;
  quantite_vente: number;
  date_achat?: string;
  date_vente?: string;
  prix_achat?: number;
  prix_vente?: number;
}

export type ConsensusType = 'Strong Buy' | 'Buy' | 'Hold' | 'Sell' | 'Strong Sell';

export interface PositionWithGain extends OpenTradeWithRecommendations {
  gainPercent: number;
  gainDollars: number;
  consensus: ConsensusType;
}
