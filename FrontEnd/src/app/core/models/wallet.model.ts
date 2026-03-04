export interface WalletBalance {
  portfolio_total: number;
  tresorerie: number;
  balances: Record<string, number>;
}

export interface WalletTransaction {
  id?: number;
  date: string;
  action: 'ajout' | 'retrait';
  amount: number;
  currency: string;
}

export interface TransactionRequest {
  date: string;
  action: 'ajout' | 'retrait';
  amount: number;
  currency: string;
}

export interface TransactionResponse {
  success: boolean;
  new_balance: number;
}

export interface WalletHistoryItem {
  id: number;
  date: string;
  action: string;
  amount: number;
  currency: string;
}
